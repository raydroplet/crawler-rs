use reqwest::Client;
pub use reqwest::{StatusCode, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinError;

//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//
// what the external world sees

#[derive(Clone)]
pub enum CrawlCommand {
    Request(CrawlRequest), // starts a crawl of a defined depth
    Terminate,             // WARN: do we need an explict Terminate command?
}

// TODO: only inform a queued page if sure you gonna crawl it
// (silently skip duplicates) avoid overuse of skipped
pub enum CrawlResponse {
    Page(ParserResult), // occasionally returns the result of a single page crawl
    Queued(Url, usize), // url, number of queued links in it
    Skipped(Url),
    Error(Url, CrawlError),
}

////

// TODO: define the possible error types
#[derive(Debug, Clone)]
pub enum CrawlError {
    Network(String),
    Timeout,
    ParseHtml(String),
    TaskPanic(String),
    Other(String),
}

// TODO: test
impl fmt::Display for CrawlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrawlError::Network(msg) => write!(f, "Network error: {}", msg),
            CrawlError::Timeout => write!(f, "Connection timed out"),
            CrawlError::ParseHtml(msg) => write!(f, "HTML parsing failed: {}", msg),
            CrawlError::TaskPanic(msg) => write!(f, "Tokio task panic: {}", msg),
            CrawlError::Other(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl CrawlError {
    pub fn name(&self) -> &'static str {
        match self {
            CrawlError::Network(_) => "Network",
            CrawlError::Timeout => "Timeout",
            CrawlError::ParseHtml(_) => "Parse",
            CrawlError::TaskPanic(_) => "Task",
            CrawlError::Other(_) => "Other",
        }
    }
}

impl Error for CrawlError {
    // default method implementations.
}

////

#[derive(Clone)]
pub struct CrawlRequest {
    pub source: Url,
    pub depth: i8,
}

#[derive(Clone)]
pub struct PageMetadata {
    pub url: Url,
    // pub depth: i8,
    pub status: StatusCode,
    pub timestamp_start: SystemTime,
    pub timestamp_end: SystemTime,
    pub discovered_links: Vec<Url>,
}

pub struct ParserResult {
    pub metadata: PageMetadata,
    pub content: String,
}

pub struct WebCrawler {
    client: Client,
}

////////

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

struct RequesterResult {
    source: Url,
    depth: i8,
    html_body: String,
}

enum ManagerEvent {
    Request(CrawlRequest),
    Branch(Url, i8, HashSet<Url>),
    Skipped(Url),
    Error(Url, CrawlError),
    TaskPanic(JoinError),
    Terminate,
}

///////

impl WebCrawler {
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .user_agent(format!(
                "Crawler-rs/0.1 (https://github.com/{}/{}",
                SIGNATURE, REPOSITORY
            ))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?; // may fail early

        Ok(Self { client: client })
    }

    pub async fn run(
        &self,
        crawler_command_rx: flume::Receiver<CrawlCommand>,
        crawler_response_tx: flume::Sender<CrawlResponse>,
    ) {
        Self::event_loop(crawler_command_rx, crawler_response_tx, self.client.clone()).await;
    }

    // TODO: add a per-website request delay
    async fn event_loop(
        command_rx: flume::Receiver<CrawlCommand>,
        response_tx: flume::Sender<CrawlResponse>,
        client: reqwest::Client,
    ) {
        let mut visited = HashSet::new();
        let max_requesters = Arc::new(Semaphore::new(4));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        loop {
            tokio::select! {
                cmd_opt = command_rx.recv_async() => {
                    let Ok(command) = cmd_opt else {
                        break; // channel closed. break the loop.
                    };

                    match command {
                        CrawlCommand::Request(request) => {
                            let _ = event_tx.send(ManagerEvent::Request(request));
                        }
                        CrawlCommand::Terminate => {
                            // all channels will be dropped; consequently,
                            // all tasks using them will terminate.
                            break;
                        }
                    }
                }
                event_opt = event_rx.recv() => {
                    let Some(event) = event_opt else {
                        // channel closed. break the loop.
                        break;
                    };

                    match event {
                        ManagerEvent::Request(request) => {
                            // depth check
                            if request.depth < 0 {
                                continue; // invalid request
                            }

                            // tracking crawls
                            // WARN: is there any scenario this check may not be sufficient?
                            if !visited.insert(request.source.clone()) {
                                let _ = event_tx.send(ManagerEvent::Skipped(request.source));
                                continue; // we already visited this page
                            }

                            // TODO: we currently avoid crawling visited pages again completely.
                            // implement a small state machine alike Enum(Pending, Parsed) to track
                            // the current state of a page request.
                            //
                            // here's a previous implementation idea that only relies on depth (i8)
                            //
                            // match visited.entry(request.source.clone()) {
                            //     Entry::Occupied(mut entry) => {
                            //         // in case we are crawling this website again, only do so if
                            //         // the requested depth is bigger than the currently crawled one
                            //         if *entry.get() >= request.depth {
                            //             continue;
                            //         }
                            //         entry.insert(request.depth);
                            //     },
                            //     Entry::Vacant(entry) => {
                            //         entry.insert(request.depth);
                            //     }
                            // }

                            // spawns the task
                            tokio::spawn({
                                let timestamp_start = SystemTime::now();
                                let permit = max_requesters.clone();
                                let event_tx = event_tx.clone();
                                let response_tx = response_tx.clone();
                                let client = client.clone();

                                async move {
                                    let Some((status, body, urls)) = WebCrawler::process_page(permit, request.clone(), client, event_tx.clone()).await else {
                                        let _ = event_tx.send(ManagerEvent::Skipped(request.source));
                                        return
                                    };

                                    // branch off new requests, if applicable
                                    if  request.depth > 0  {
                                        let _ = event_tx.send(ManagerEvent::Branch(request.source.clone(), request.depth, urls.clone()));
                                    }

                                    let metadata = PageMetadata {
                                        url: request.source.clone(),
                                        // depth: request.depth,
                                        status: status,
                                        timestamp_start: timestamp_start,
                                        timestamp_end: SystemTime::now(),
                                        discovered_links: urls.into_iter().collect(),
                                    };
                                    let message = ParserResult {
                                        metadata: metadata,
                                        content: body,
                                    };

                                    // sends the payload to the listener
                                    if response_tx.send_async(CrawlResponse::Page(message)).await.is_err() {
                                        let _ = event_tx.send(ManagerEvent::Terminate);
                                    }
                                }
                            });
                        }
                        ManagerEvent::Branch(url, depth, links) => {
                            // notify a new crawl is being queued
                            let url = url.clone();
                            let ammount = links.len();
                            let _ = response_tx.send_async(CrawlResponse::Queued(url, ammount)).await;

                            for link in &links {
                                let event_tx = event_tx.clone();
                                let request = CrawlRequest {
                                    source: link.clone(),
                                    depth: depth - 1,
                                };
                                let _ = event_tx.send(ManagerEvent::Request(request));
                            }
                        }
                        ManagerEvent::Error(url, crawl_error) => {
                            if response_tx.send_async(CrawlResponse::Error(url, crawl_error)).await.is_err() {
                              let _ = event_tx.send(ManagerEvent::Terminate);
                            }
                        }
                        ManagerEvent::Skipped(url) => {
                            if response_tx.send_async(CrawlResponse::Skipped(url)).await.is_err(){
                                let _ = event_tx.send(ManagerEvent::Terminate);
                            };
                        }
                        ManagerEvent::TaskPanic(_err) => {
                            // we ignore those for now, but it's here if necessary.
                        }
                        ManagerEvent::Terminate => {
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn process_page(
        permit: Arc<Semaphore>,
        request: CrawlRequest,
        client: Client,
        event_tx: mpsc::UnboundedSender<ManagerEvent>,
    ) -> Option<(StatusCode, String, HashSet<Url>)> {
        //
        let _owned = permit.acquire_owned().await.ok()?;

        let (body, status) = match Self::process_page_request(request.clone(), client.clone()).await
        {
            Ok(Some(res)) => res,
            Ok(None) => {
                let _ = event_tx.send(ManagerEvent::Skipped(request.source)); // not hml
                return None;
            }
            Err(crawl_err) => {
                let _ = event_tx.send(ManagerEvent::Error(request.source, crawl_err));
                return None;
            }
        };

        let urls = match Self::process_page_parse(request.source, body.clone()).await {
            Ok(res) => res,
            Err(err) => {
                let _ = event_tx.send(err);
                return None;
            }
        };

        Some((status, body, urls))
    }

    // TODO: consider moving the body of the function to the caller
    async fn process_page_request(
        request: CrawlRequest,
        client: Client,
    ) -> Result<Option<(String, StatusCode)>, CrawlError> {
        match Self::request_webpage_html(request.source.clone(), client).await {
            Ok(body_opt) => Ok(body_opt), // NOTE: returs NONE if the page is not html
            Err(err) => Err(CrawlError::Network(err.to_string())),
        }
    }

    // TODO: consider moving the body of the function to the caller
    async fn process_page_parse(
        url: Url,
        body: String,
        // response_tx: flume::Sender<CrawlResponse>,
    ) -> Result<HashSet<Url>, ManagerEvent> {
        let task_url = url.clone();
        let result = tokio::task::spawn_blocking(move || {
            let urls: HashSet<Url> = Self::parse_webpage_html(task_url, body);
            urls
        })
        .await;

        return match result {
            Ok(res) => Ok(res),
            Err(err) => Err(ManagerEvent::TaskPanic(err)),
        };
    }

    ////

    async fn request_webpage_html(
        url: Url,
        client: Client,
    ) -> Result<Option<(String, StatusCode)>, reqwest::Error> {
        let response = client
            .get(url)
            .send() //
            .await?;

        // extract the content-type header
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
            .unwrap_or(""); // default to empty if the server didn't send a header

        // if it is not html, return none
        if !content_type.starts_with("text/html") {
            return Ok(None);
        }

        let status = response.status();
        let body = response.text().await?;
        Ok(Some((body, status)))
    }

    fn parse_webpage_html(url: Url, html_body: String) -> HashSet<Url> {
        let source = url; // TODO:: use the correct url.source();

        let document = Html::parse_document(&html_body); // builds a DOM from the raw text
        let Ok(selector) = Selector::parse("a[href]") else {
            return HashSet::new(); // no links in page
        };
        let mut extracted_urls = HashSet::new();

        for element in document.select(&selector) {
            // extracts the actual text inside the href attribute
            if let Some(href) = element.value().attr("href") {
                //
                if let Ok(mut absolute_url) = source.join(href) {
                    // Only accept standard web protocols
                    let scheme = absolute_url.scheme();
                    if scheme == "http" || scheme == "https" {
                        // remove headers (page.com/article#header -> page.com/article)
                        absolute_url.set_fragment(None);
                        // remove query parameters (?action=edit)
                        // NOTE: this filters some valid links (like youtube.com/watch?v=video_id)
                        absolute_url.set_query(None);
                        //
                        extracted_urls.insert(absolute_url);
                    }
                }
            }
        }

        extracted_urls
    }
}
