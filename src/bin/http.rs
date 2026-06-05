use crawler_rs::crawler::{CrawlCommand, CrawlRequest, CrawlResponse, ParserResult, WebCrawler};
use reqwest::Url;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

async fn crawling_test() {
    // let (tx, mut rx) = mpsc::channel(1024);
    //
    // // 1. Wrap the crawler in an Arc so we can share ownership
    // let crawler = Arc::new(WebCrawler::new(tx).expect("Failed to initialize crawler"));
    //
    // let request = CrawlRequest {
    //     source: Url::parse("https://example.com").expect("Invalid URL"),
    //     depth: 2,
    // };
    //
    // // 2. Clone the Arc pointer (this does not clone the crawler itself)
    // let crawler_clone = crawler.clone();
    //
    // // 3. Move the clone into the blocking task
    // tokio::task::spawn_blocking(move || {
    //     if crawler_clone
    //         .send_blocking(CrawlCommand::RequestCrawl(request))
    //         .is_err()
    //     {
    //         println!("Failed to send initial request.");
    //     }
    // });
    //
    // println!("Crawler started. Waiting for results...\n");
    //
    // let timeout_duration = Duration::from_secs(10);
    //
    // loop {
    //     match tokio::time::timeout(timeout_duration, rx.recv()).await {
    //         Ok(Some(response)) => match response {
    //             CrawlResponse::Page(parser_result) => {
    //                 println!("========================================");
    //                 println!("Source URL  : {}", parser_result.domain);
    //                 println!("Depth Left  : {}", parser_result.depth);
    //                 println!("HTML Size   : {} bytes", parser_result.page_content.len());
    //                 println!("Links Found : {}", parser_result.discovered_links.len());
    //                 println!("--- Links ---");
    //
    //                 for link in parser_result.discovered_links {
    //                     println!(" -> {}", link);
    //                 }
    //             }
    //             CrawlResponse::Queued(url) => {
    //                 // ignore.
    //             }
    //         },
    //         Ok(None) => {
    //             println!("Crawler finished processing all links.");
    //             break;
    //         }
    //         Err(_) => {
    //             println!(
    //                 "\nNo new pages crawled for {} seconds. Timing out and shutting down.",
    //                 timeout_duration.as_secs()
    //             );
    //             break;
    //         }
    //     }
    // }
    //
    // // 4. Ensure the original crawler lives until the end of main
    // drop(crawler);
    // println!("Ending.\n");
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
