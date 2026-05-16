use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

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
    let body: String = request_webpage(url.clone(), &client).await?;
    let links: Vec<Url> = parse_webpage_links(&body, &url)?.into_iter().collect();

    println!("Found:");
    for link in links {
        println!("  -> {}", link);
    }

    Ok(())
}

async fn request_webpage(url: Url, client: &Client) -> Result<String, reqwest::Error> {
    let response = client
        .get(url)
        .send() //
        .await?;

    let body = response
        .text() //
        .await?;

    Ok(body)
}

fn parse_webpage_links(body: &String, base_url: &Url) -> Result<HashSet<Url>, Box<dyn Error>> {
    let document = Html::parse_document(body); // builds a DOM from the raw text
    let selector = Selector::parse("a[href]")?;
    let mut extracted_urls = HashSet::new();

    for element in document.select(&selector) {
        // extracts the actual text inside the href attribute
        if let Some(href) = element.value().attr("href") {
            //
            if let Ok(mut absolute_url) = base_url.join(href) {
                // remove headers (page.com/article#header -> page.com/article)
                absolute_url.set_fragment(None);
                extracted_urls.insert(absolute_url);
            }
        }
    }

    Ok(extracted_urls)
}

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
// 1. create a reqwest client and pass it for tasks to use
// 2. The User-Agent Header (Avoiding Blocks)
// 3. timeouts
// 4. handle gttp 404 or 500, as they will simply return the html error page
