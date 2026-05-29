use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, mpsc, oneshot};
use std::time::{SystemTime, UNIX_EPOCH};

//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//
// what the external world sees

struct CrawlRequest {
    source: Url,
    depth: i32,
}

enum CrawlCommand {
    RequestCrawl(CrawlRequest), // starts a crawl of a defined depth
    Terminate,
}

enum CrawlResponse {
    Page(ParserResult), // occasionally returns the result of a single page crawl
}

//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//
// (mostly) interal implementation

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

struct RequesterResult {
    source: Url,
    depth: i32,
    html_body: String,
}

struct ParserResult {
    domain: Url,
    path: Option<Url>,
    depth: i32,
    //
    timestamp_start: SystemTime,
    timestamp_end: SystemTime,
    //
    page_content: String,
    discovered_links: HashSet<Url>,
}

enum ManagerEvent {
    Parsed(ParserResult),
    // NOTE: alike command and events, errors should be sent from it's own channel, but I will avoid this for now
    // RequesterError(CrawlRequest, reqwest::Error),
}

struct WebCrawler {
    manager_command_tx: mpsc::Sender<CrawlCommand>,
}

impl WebCrawler {
    pub fn new(sender: mpsc::Sender<CrawlResponse>) -> Result<Self, reqwest::Error> {
        //--- client initialization ---//
        let client = Client::builder()
            .user_agent(format!(
                "Crawler-rs/0.1 (https://github.com/{}/{}",
                SIGNATURE, REPOSITORY
            ))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?; // may fail early

        //--- tasks configuration and wiring ---//
        let (manager_event_tx, manager_event_rx) = mpsc::channel(1024);
        let (manager_command_tx, manager_command_rx) = mpsc::channel(1024);
        let (parser_tx, parser_rx) = mpsc::channel(1024);

        // spawns the parser
        tokio::spawn({
            let manager_tx = manager_event_tx.clone();
            async move {
                Self::parser_actor(parser_rx, manager_tx).await;
            }
        });

        // spawns the command/event manager
        tokio::spawn(async move {
            Self::manager_actor(
                manager_command_rx,
                manager_event_rx,
                parser_tx,
                sender,
                client,
            )
            .await;
        });

        //--- instantiation ---//
        Ok(Self {
            manager_command_tx: manager_command_tx,
        })
    }

    pub fn send_blocking(
        &self,
        command: CrawlCommand,
    ) -> Result<(), mpsc::error::SendError<CrawlCommand>> {
        self.manager_command_tx.blocking_send(command)
    }

    async fn manager_actor(
        mut command_rx: mpsc::Receiver<CrawlCommand>,
        mut event_rx: mpsc::Receiver<ManagerEvent>,
        parser_tx: mpsc::Sender<RequesterResult>,
        sender: mpsc::Sender<CrawlResponse>,
        client: reqwest::Client,
    ) {
        //--- handle commands ---//
        let mut visited = HashSet::new();
        // we allow limited simultaneous network requests.
        let max_requesters = Arc::new(Semaphore::new(4));

        // TODO: add a per-website request delay

        loop {
            tokio::select! {
                cmd_opt = command_rx.recv() => {
                    let Some(command) = cmd_opt else {
                        // channel closed. break the loop.
                        break;
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
                            if sender.send(response).await.is_err() {
                                // the external client disconnected without sendind a terminate command.
                                break;
                            };
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
                    depth: request_result.depth,
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

async fn crawling_test() {
    let (tx, mut rx) = mpsc::channel(1024);

    // 1. Wrap the crawler in an Arc so we can share ownership
    let crawler = Arc::new(WebCrawler::new(tx).expect("Failed to initialize crawler"));

    let request = CrawlRequest {
        source: Url::parse("https://example.com").expect("Invalid URL"),
        depth: 2,
    };

    // 2. Clone the Arc pointer (this does not clone the crawler itself)
    let crawler_clone = crawler.clone();

    // 3. Move the clone into the blocking task
    tokio::task::spawn_blocking(move || {
        if crawler_clone
            .send_blocking(CrawlCommand::RequestCrawl(request))
            .is_err()
        {
            println!("Failed to send initial request.");
        }
    });

    println!("Crawler started. Waiting for results...\n");

    let timeout_duration = Duration::from_secs(10);

    loop {
        match tokio::time::timeout(timeout_duration, rx.recv()).await {
            Ok(Some(response)) => match response {
                CrawlResponse::Page(parser_result) => {
                    println!("========================================");
                    println!("Source URL  : {}", parser_result.domain);
                    println!("Depth Left  : {}", parser_result.depth);
                    println!("HTML Size   : {} bytes", parser_result.page_content.len());
                    println!("Links Found : {}", parser_result.discovered_links.len());
                    println!("--- Links ---");

                    for link in parser_result.discovered_links {
                        println!(" -> {}", link);
                    }
                }
            },
            Ok(None) => {
                println!("Crawler finished processing all links.");
                break;
            }
            Err(_) => {
                println!(
                    "\nNo new pages crawled for {} seconds. Timing out and shutting down.",
                    timeout_duration.as_secs()
                );
                break;
            }
        }
    }

    // 4. Ensure the original crawler lives until the end of main
    drop(crawler);
    println!("Ending.\n");
}

#[tokio::main]
async fn main() {
    crawling_test().await;
}

// TODO:
// 1. [-] create a reqwest client and pass it for tasks to use
// 2. [x] The User-Agent Header (Avoiding Blocks)
// 3. [x] timeouts
// 4. [ ] handle gttp 404 or 500, as they will simply return the html error page
// 5. [ ] respect robots.txt
// 6. [ ] implement delays for each individual website
