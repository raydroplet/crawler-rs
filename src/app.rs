pub use crate::crawler::{CrawlCommand, CrawlError, CrawlRequest, PageMetadata};
use crate::crawler::{CrawlResponse, Url, WebCrawler};
use crate::gui::ViewEgui;
use html_to_markdown_rs::convert;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use tokio::sync::{mpsc};

pub struct App {}

pub enum AppRequest {
    Crawler(CrawlCommand),
    Markdown(Url),
}

pub enum CrawlEvent {
    Page(PageMetadata),
    Queued(Url, usize),
    Skipped(Url),
    Error(Url, CrawlError),
}

pub enum AppResponse {
    Crawler(CrawlEvent),
    Markdown(Url, String),
}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        //// channels
        let (crawler_response_tx, crawler_response_rx) = mpsc::channel(1024);
        let (crawler_command_tx, crawler_command_rx) = mpsc::channel(8);
        let (view_response_tx, view_response_rx) = flume::bounded(1024);
        let (view_command_tx, view_command_rx) = flume::bounded(1024);

        //// actors
        let view = ViewEgui::new(view_response_rx, view_command_tx);
        let crawler = WebCrawler::new()?;

        //// threads + tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        runtime.spawn(async move {
            crawler.run(crawler_command_rx, crawler_response_tx).await;
        });

        runtime.spawn(async move {
            Self::async_event_loop(
                view_command_rx,
                view_response_tx,
                crawler_command_tx.clone(),
                crawler_response_rx,
            )
            .await;
        });

        let _ = ViewEgui::run(view);

        println!("exiting App::run()");
        Ok(())
    }

    async fn async_event_loop(
        view_command_rx: flume::Receiver<AppRequest>,
        view_response_tx: flume::Sender<AppResponse>,
        crawler_command_tx: mpsc::Sender<CrawlCommand>,
        mut crawler_response_rx: mpsc::Receiver<CrawlResponse>,
    ) {
        let pages: RefCell<HashMap<Url, String>> = HashMap::new().into();

        loop {
            tokio::select! {
                view_cmd_opt = view_command_rx.recv_async() => {
                    let Ok(command) = view_cmd_opt else { break; };
                    match command {
                        AppRequest::Crawler(command) => {
                            if crawler_command_tx.send(command.clone()).await.is_err() {
                               break;
                            };

                            // quick debug info
                            if false {
                                match command {
                                    CrawlCommand::Request(request) => {
                                        println!(
                                            "view_command request: {} ({})",
                                            request.source, request.depth
                                        );
                                    }
                                    CrawlCommand::Terminate => {
                                        println!("view_command terminate");
                                        break;
                                        // TODO: notify crawler
                                    }
                                }
                            }
                        }
                        AppRequest::Markdown(url) => {
                            if let Some(content) = pages.borrow_mut().get(&url) {
                                let markdown = match convert(&content, None) {
                                    Ok(result) => result.content.unwrap_or_default(),
                                    Err(err) => {
                                        eprintln!("conversion failed: {err}");
                                        String::from("Failed to parse html into markdown.")
                                    }
                                };
                                if view_response_tx
                                    .send(AppResponse::Markdown(url, markdown))
                                    .is_err()
                                {
                                    break;
                                };
                            }
                        }
                    }
                }

                crawler_res_opt = crawler_response_rx.recv() => {
                    let Some(response) = crawler_res_opt else { break; };
                    match response {
                        CrawlResponse::Page(page) => {
                            println!(
                                "received page: {} ({})",
                                page.metadata.url,
                                page.metadata.discovered_links.len()
                            );
                            // caches the page in case the gui asks for its contents
                            pages
                                .borrow_mut()
                                .insert(page.metadata.url.clone(), page.content);
                            let event = CrawlEvent::Page(page.metadata);
                            if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                break;
                            };
                        }
                        CrawlResponse::Skipped(url) => {
                            println!("skipped page: {}", url);
                            //
                            let event = CrawlEvent::Skipped(url);
                            if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                break;
                            };
                        }
                        CrawlResponse::Queued(url, count) => {
                            println!("queued page: {} ({})", url, count);
                            //
                            let event = CrawlEvent::Queued(url, count);
                            if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                break;
                            };
                        }
                        CrawlResponse::Error(url, err) => {
                            println!("error: {} -> {}", url, err);
                            //
                            let event = CrawlEvent::Error(url, err);
                            if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                break;
                            };
                        }
                    }
                }
            }
        }
    }
}
