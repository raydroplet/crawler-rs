use reqwest::{Client, Url, StatusCode};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Semaphore, mpsc, oneshot};

//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//
// what the external world sees

#[derive(Clone)]
pub enum CrawlCommand {
    RequestCrawl(CrawlRequest), // starts a crawl of a defined depth
    Terminate,
}

pub enum CrawlResponse {
    Page(ParserResult), // occasionally returns the result of a single page crawl
    Queued(Url),
}

////////

#[derive(Clone)]
pub struct CrawlRequest {
    pub source: Url,
    pub depth: i32,
}

pub struct ParserResult {
    pub domain: Url,
    pub path: Option<Url>,
    pub depth: i32,
    pub status: StatusCode,
    //
    pub timestamp_start: SystemTime,
    pub timestamp_end: SystemTime,
    //
    pub page_content: String,
    pub discovered_links: HashSet<Url>,
}

pub struct WebCrawler {
    client: Client,
}

///////

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

struct RequesterResult {
    source: Url,
    depth: i32,
    html_body: String,
}

enum ManagerEvent {
    Parsed(ParserResult),
    Error(),
    // NOTE: alike command and events, errors should be sent from it's own channel, but I will avoid this for now
    // RequesterError(CrawlRequest, reqwest::Error),
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
        let (manager_event_tx, manager_event_rx) = mpsc::channel(1024);
        let (parser_tx, parser_rx) = mpsc::channel(1024);

        // spawns the parser
        tokio::spawn({
            let manager_tx = manager_event_tx.clone();
            async move {
                Self::parser_actor(parser_rx, manager_tx).await;
            }
        });

        Self::manager_actor(
            crawler_command_rx,
            crawler_response_tx,
            manager_event_rx,
            parser_tx,
            self.client.clone(),
        )
        .await;
    }

    // pub fn send_blocking(
    //     &self,
    //     command: CrawlCommand,
    // ) -> Result<(), mpsc::error::SendError<CrawlCommand>> {
    //     self.crawler_command_tx.blocking_send(command)
    // }

    async fn manager_actor(
        mut command_rx: flume::Receiver<CrawlCommand>,
        response_tx: flume::Sender<CrawlResponse>,
        //
        mut event_rx: mpsc::Receiver<ManagerEvent>,
        parser_tx: mpsc::Sender<RequesterResult>,
        client: reqwest::Client,
    ) {
        //--- handle commands ---//
        let mut visited = HashSet::new();
        // we allow limited simultaneous network requests.
        let max_requesters = Arc::new(Semaphore::new(4));

        // TODO: add a per-website request delay

        loop {
            tokio::select! {
                cmd_opt = command_rx.recv_async() => {
                    let Ok(command) = cmd_opt else {
                        break; // channel closed. break the loop.
                    };

                    match command {
                        CrawlCommand::RequestCrawl(request) => {
                            // 1. depth check
                            if request.depth < 0 {
                                continue;
                            }

                            // 2. tracking crawls
                            if !visited.insert(request.source.clone()) {
                                // we already visited this page
                                continue;
                            }

                            // 3. spawns a page requester
                            tokio::spawn({
                                let permit = max_requesters.clone();
                                let sender = parser_tx.clone();
                                let client = client.clone();
                                async move {
                                    Self::requester_worker(permit, request, sender, client).await;
                                }
                            });
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
                        ManagerEvent::Parsed(parser_result) => {
                            // 1. depth check + spawn new crawls for the discovered links
                            let new_depth = parser_result.depth - 1;
                            if new_depth >= 0 {
                                for link in &parser_result.discovered_links {
                                    if !visited.insert(link.clone()) {
                                        // we already visited this page
                                        continue;
                                    }

                                    // notify a new task is being queued
                                    let response = CrawlResponse::Queued(link.clone());
                                    if response_tx.send_async(response).await.is_err() {
                                        break; // external client disconnected
                                    };

                                    tokio::spawn({
                                        let permit = max_requesters.clone();
                                        let sender = parser_tx.clone();
                                        let client = client.clone();
                                        let request  = CrawlRequest {
                                            source: link.clone(),
                                            depth: new_depth,
                                        };
                                        async move {
                                            Self::requester_worker(permit, request, sender, client).await;
                                        }
                                    });
                                }
                            }

                            // 2. sends back the result of a crawled page using the 'sender'
                            let response = CrawlResponse::Page(parser_result);
                            if response_tx.send_async(response).await.is_err() {
                                // the external client disconnected without sendind a terminate command.
                                break;
                            };
                        }
                        ManagerEvent::Error() => {
                            //
                        }
                    }
                }
            }
        }
    }

    async fn requester_worker(
        permit: Arc<Semaphore>,
        request: CrawlRequest,
        sender: mpsc::Sender<RequesterResult>,
        client: reqwest::Client,
    ) {
        println!("Spawn request worker: {}", request.source);

        let Ok(_) = permit.acquire_owned().await else {
            // the semaphore is closed
            return;
        };

        let CrawlRequest { source, depth } = request;

        match Self::request_webpage_html(source.clone(), client).await {
            Ok(Some(body)) => {
                println!("crawled: {}", source);
                // happy path: sends the page body to the parser
                let result = RequesterResult {
                    source: source,
                    depth: depth,
                    html_body: body,
                };
                if sender.send(result).await.is_err() {
                    // no parser to hear us, silently return.
                    return;
                }
            }
            Ok(None) => {
                println!("not html: {}", source);
                // not html, we can safely ignore this page
                return;
            }
            Err(err) => {
                println!("Network error for {}: {:?}", source, err);
                // NOTE: for now it's just fire and forget
                return;
            }
        };
    }

    // WARN: is there a better name for this method?
    async fn parser_actor(
        mut parser_rx: mpsc::Receiver<RequesterResult>,
        manager_tx: mpsc::Sender<ManagerEvent>,
    ) {
        while let Some(request_result) = parser_rx.recv().await {
            println!("Parsing: {}", request_result.source);

            let manager_tx = manager_tx.clone();
            let (tx, rx) = oneshot::channel();

            // spawns the rayon worker and goes back to listening request_result's;
            rayon::spawn(move || {
                println!("Spawn parser acdor: {}", request_result.source);
                let urls: HashSet<Url> = Self::parse_webpage_html(&request_result);
                let result = ParserResult {
                    domain: request_result.source,
                    path: None,
                    depth: request_result.depth,
                    status: StatusCode::IM_A_TEAPOT,
                    //
                    timestamp_start: SystemTime::now(), // WARN: undefined
                    timestamp_end: SystemTime::now(),   // WARN: undefined
                    //
                    page_content: request_result.html_body,
                    discovered_links: urls,
                };
                let _ = tx.send(result);
            });

            tokio::spawn(async move {
                if let Ok(result) = rx.await {
                    if manager_tx
                        .send(ManagerEvent::Parsed(result))
                        .await //
                        .is_err()
                    {
                        // no manager to hear our pleas, end the task
                        return;
                    };
                }
            });
        }
    }

    async fn request_webpage_html(
        url: Url,
        client: Client,
    ) -> Result<Option<String>, reqwest::Error> {
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

        let body = response
            .text() //
            .await?;

        Ok(Some(body))
    }

    fn parse_webpage_html(request: &RequesterResult) -> HashSet<Url> {
        let RequesterResult {
            source,
            depth: _,
            html_body,
        } = request;

        let document = Html::parse_document(&html_body); // builds a DOM from the raw text
        let Ok(selector) = Selector::parse("a[href]") else {
            // NOTE: for now it's just fire and forget
            return HashSet::new();
        };
        let mut extracted_urls = HashSet::new();

        for element in document.select(&selector) {
            // extracts the actual text inside the href attribute
            if let Some(href) = element.value().attr("href") {
                //
                if let Ok(mut absolute_url) = source.join(href) {
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

        extracted_urls
    }
}
