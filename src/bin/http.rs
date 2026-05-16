use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::error::Error;
use std::time::Duration;

const SIGNATURE: &str = "raydroplet";
const REPOSITORY: &str = "crawler-rs";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder()
        .user_agent(format!("Crawler-rs/0.1 (https://github.com/{}/{}", SIGNATURE, REPOSITORY))
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = Url::parse("https://en.wikipedia.org/wiki/Rust_(programming_language)")?;
    let body: String = request_webpage(url.clone(), &client).await?;
    let links: Vec<Url> = parse_webpage_links(&body, &url)?;

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

fn parse_webpage_links(body: &String, base_url: &Url) -> Result<Vec<Url>, Box<dyn Error>> 
{
    let document = Html::parse_document(body); // builds a DOM from the raw text
    let selector = Selector::parse("a[href]")?;
    let mut extracted_urls = Vec::new();

    for element in document.select(&selector) {
        // extracts the actual text inside the href attribute
        if let Some(href) = element.value().attr("href") {
            //
            if let Ok(absolute_url) = base_url.join(href) {
                extracted_urls.push(absolute_url);
            }
        }
    }

    Ok(extracted_urls)
}

// TODO:
// 1. create a reqwest client and pass it for tasks to use
// 2. The User-Agent Header (Avoiding Blocks)
// 3. timeouts
// 4. handle gttp 404 or 500, as they will simply return the html error page
