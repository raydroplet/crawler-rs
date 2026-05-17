use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

struct CrawlRequest {
    source: Url,
    depth: u32,
}

struct CrawlResult {
    source: Url,
    depth: u32,
    body: String,
    discovered_links: Vec<Url>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent(format!(
            "Crawler-rs/0.1 (https://github.com/{}/{}",
            SIGNATURE, REPOSITORY
        ))
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = Url::parse("https://en.wikipedia.org/wiki/Rust_(programming_language)")?;

    let (requester_tx, mut requester_rx) = mpsc::channel(32); // tasks -> parser
    let (parser_tx, mut parser_rx) = mpsc::channel(32); // parser -> manager
    let (manager_tx, mut manager_rx) = mpsc::channel(32); // main -> manager
    // let (main_tx, mut main_rx) = mpsc::channel(32); // manager -> main

    // defines our first crawl request beforehand
    let request = CrawlRequest {
        source: url,
        depth: 1,
    };
    manager_tx.send(request);

    // create the links manager
    let manager_task = tokio::spawn(async move {
        // receives links from the parser and spawns new webpage requests
        crawling_manager(client, parser_rx, requester_tx).await;
    });

    // create the content parser
    let parser_task = tokio::spawn(async move {
        // receives webpages, parses them and sends the results to the manager
        crawling_parser(requester_rx, parser_tx).await;
    });

    tokio::join!(manager_task, parser_task);

    Ok(())
}

fn crawl_request() {}

async fn crawling_manager(
    client: Client,
    parser_rx: mpsc::Receiver<u8>,
    requester_tx: mpsc::Sender<u8>,
) {
}

async fn crawling_parser(requester_rx: mpsc::Receiver<u8>, tx: mpsc::Sender<u8>) {}

//---//---//---//---//

async fn a() {}

async fn request_webpage_html(url: Url, client: &Client) -> Result<Option<String>, reqwest::Error> {
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

//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//---//

// async fn _crawling_manager(client: Client) {
//     // TODO: check for deadlocks
//     let manager = tokio::spawn(async move {
//         let root_url: Url = Url::parse("https://wikipedia.com").expect("");
//         let mut crawled_pages: HashSet<Url> = HashSet::new();
//         let (transmitter, mut receiver) = mpsc::channel(32);
//
//         // sends the root url
//         let _ = transmitter.send(HashSet::from([root_url.clone()])).await;
//
//         let task_client = client.clone();
//         let task_trasmitter = transmitter.clone();
//         let task_url = root_url.clone();
//         let task_worker = tokio::spawn(async move {
//             let Ok(request_response) = request_webpage_html(task_url, &task_client).await else {
//                 return; // in case of error we just end the task
//             };
//
//             let Some(body) = request_response else {
//                 return; // likely not html
//             };
//
//             // NOTE: the only possible error is for the receiver to be closed.
//             // let _ = task_trasmitter.send(links);
//         });
//
//         while let Some(message) = receiver.recv().await {
//
//             //
//         }
//     });
//
//     // TODO: clean this up
//     if let Err(join_err) = manager.await {
//         // task panicked or was canceled
//         if join_err.is_panic() {
//             println!("The task panicked!");
//         } else {
//             println!("The task was canceled!");
//         }
//     }
// }
//
// fn parse_webpage_links(body: &String, base_url: &Url) -> Result<HashSet<Url>, Box<dyn Error>> {
//     let document = Html::parse_document(body); // builds a DOM from the raw text
//     let selector = Selector::parse("a[href]")?;
//     let mut extracted_urls = HashSet::new();
//
//     for element in document.select(&selector) {
//         // extracts the actual text inside the href attribute
//         if let Some(href) = element.value().attr("href") {
//             //
//             if let Ok(mut absolute_url) = base_url.join(href) {
//                 // remove headers (page.com/article#header -> page.com/article)
//                 absolute_url.set_fragment(None);
//                 // remove query parameters (?action=edit)
//                 // NOTE: this filters some valid links (like youtube.com/watch?v=video_id)
//                 absolute_url.set_query(None);
//                 //
//                 extracted_urls.insert(absolute_url);
//             }
//         }
//     }
//
//     Ok(extracted_urls)
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let client = Client::builder()
//         .user_agent(format!(
//             "Crawler-rs/0.1 (https://github.com/{}/{}",
//             SIGNATURE, REPOSITORY
//         ))
//         .connect_timeout(Duration::from_secs(5))
//         .timeout(Duration::from_secs(30))
//         .build()?;
//
//     let url = Url::parse("https://en.wikipedia.org/wiki/Rust_(programming_language)")?;
//     let mut links: HashSet<Url> = HashSet::new();
//
//     // workers:
//     // 1. spawn a tokio task with a root link to execute request_webpage
//     // 2. send all the found links into a channel
//     // 3. end the task
//
//     // manager:
//     // 1. awaits for any incoming links in the channel
//     // 3. keeps track of invalid websites that return 505 or some other error
//     // 4. filters the undesired links
//     // 5. adds the remaining ones to it's "database"
//     // 6. fire new workers for every of those newly found links, respecting the crawl depth
//
//     if let Some(body) = request_webpage_html(url.clone(), &client).await? {
//         links.extend(parse_webpage_links(&body, &url)?);
//     }
//
//     println!("Found:");
//     for link in links {
//         println!("  -> {}", link);
//     }
//
//     Ok(())
// }

// NOTE: realized i'm overcomplicating things for now
//
// // helper to safely extract the true root domain using the PSL
// fn get_root_domain(host: &str) -> Option<String> {
//     addr::parse_domain_name(host)
//         .ok()
//         .and_then(|domain| domain.root().map(String::from))
// }
//
// fn parse_webpage_links(body: &String, base_url: &Url) -> Result<HashSet<Url>, Box<dyn Error>> {
//     let filter_subdirectories = true;
//     let filter_subdomains = true;
//     let filter_external_links = true;
//
//     let document = Html::parse_document(body); // builds a DOM from the raw text
//     let selector = Selector::parse("a[href]")?;
//     let mut extracted_urls = HashSet::new();
//
//     let base_host = base_url.host_str().ok_or("Base URL does not have a host")?;
//     let base_root = get_root_domain(base_host);
//
//     for element in document.select(&selector) {
//         // extracts the actual text inside the href attribute
//         if let Some(href) = element.value().attr("href") {
//             //
//             if let Ok(mut absolute_url) = base_url.join(href) {
//                 // remove headers (page.com/article#header -> page.com/article)
//                 absolute_url.set_fragment(None);
//
//                 let absolute_host = match absolute_url.host_str() {
//                     Some(host) => host,
//                     // if the page contains a non conforming link, we simply keep going
//                     _ => continue,
//                     // .ok_or("Absolute URL does not have a host")?;
//                 };
//
//                 // TODO test
//                 let is_subdirectory = {
//                     if absolute_url.origin() != base_url.origin() {
//                         false
//                     } else {
//                         let mut base_path = base_url.path().to_string();
//                         if !base_path.ends_with('/') {
//                             base_path.push('/');
//                         }
//
//                         let mut abs_path = absolute_url.path().to_string();
//                         if !abs_path.ends_with('/') {
//                             abs_path.push('/');
//                         }
//
//                         abs_path.starts_with(&base_path)
//                     }
//                 };
//
//                 // TODO test
//                 let is_subdomain = {
//                     (absolute_host == base_host)
//                         || absolute_host.ends_with(&format!(".{}", base_host))
//                 };
//
//                 // TODO test
//                 let is_external_link = { false };
//
//                 // filtering
//                 if (filter_subdirectories && is_subdirectory)
//                     || (filter_subdomains && is_subdomain)
//                     || (filter_external_links && is_external_link)
//                 {
//                     continue;
//                 }
//
//                 extracted_urls.insert(absolute_url);
//             }
//         }
//     }
//
//     Ok(extracted_urls)
// }

// TODO:
// 1. [-] create a reqwest client and pass it for tasks to use
// 2. [x] The User-Agent Header (Avoiding Blocks)
// 3. [x] timeouts
// 4. [ ] handle gttp 404 or 500, as they will simply return the html error page
// 5. [ ] respect robots.txt
// 6. [ ] implement delays for each individual website
