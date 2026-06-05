pub use crate::crawler::{CrawlCommand, CrawlError, CrawlRequest, PageMetadata};
use crate::crawler::{CrawlResponse, Url, WebCrawler};
use crate::gui::ViewEgui;
use std::collections::HashMap;
use std::error::Error;
use std::thread;

pub struct App {
    pages: HashMap<Url, String>,
}

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
        Self {
            pages: HashMap::new(),
            //
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        //// channels
        let (crawler_response_tx, crawler_response_rx) = flume::bounded(1024);
        let (crawler_command_tx, crawler_command_rx) = flume::bounded(8);
        let (view_response_tx, view_response_rx) = flume::bounded(1024);
        let (view_command_tx, view_command_rx) = flume::bounded(1024);

        //// actors
        let view = ViewEgui::new(view_response_rx, view_command_tx);
        let crawler = WebCrawler::new()?;

        //// threads + tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        thread::scope(|scope| {
            scope.spawn(move || {
                runtime.block_on(async {
                    crawler.run(crawler_command_rx, crawler_response_tx).await;
                });
            });

            scope.spawn(|| {
                self.event_loop(
                    view_command_rx,
                    view_response_tx,
                    crawler_command_tx.clone(),
                    crawler_response_rx,
                );
            });

            let _ = ViewEgui::run(view);
        });

        println!("exiting App::run()");
        Ok(())
    }

    fn event_loop(
        &mut self,
        view_command_rx: flume::Receiver<AppRequest>,
        view_response_tx: flume::Sender<AppResponse>,
        crawler_command_tx: flume::Sender<CrawlCommand>,
        crawler_response_rx: flume::Receiver<CrawlResponse>,
    ) {
        loop {
            let to_break = flume::Selector::new()
                .recv(&crawler_response_rx, |message| {
                    match message {
                        Ok(response) => match response {
                            CrawlResponse::Page(page) => {
                                println!(
                                    "received page: {} ({})",
                                    page.metadata.url,
                                    page.metadata.discovered_links.len()
                                );
                                // caches the page in case the gui asks for its contents
                                self.pages.insert(page.metadata.url.clone(), page.content);
                                let event = CrawlEvent::Page(page.metadata);
                                if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                    return true;
                                };
                            }
                            CrawlResponse::Skipped(url) => {
                                println!("skipped page: {}", url);
                                //
                                let event = CrawlEvent::Skipped(url);
                                if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                    return true;
                                };
                            }
                            CrawlResponse::Queued(url, count) => {
                                println!("queued page: {} ({})", url, count);
                                //
                                let event = CrawlEvent::Queued(url, count);
                                if view_response_tx.send(AppResponse::Crawler(event)).is_err() {
                                    return true;
                                };
                            }
                            CrawlResponse::Error(url, err) => {
                                println!("error: {} -> {}", url, err);
                            }
                        },
                        Err(err) => {
                            println!("channel closed?: {}", err);
                            return true;
                        }
                    }

                    false
                })
                .recv(&view_command_rx, |message| {
                    match message {
                        Ok(command) => {
                            match command {
                                AppRequest::Crawler(command) => {
                                    if crawler_command_tx.send(command.clone()).is_err() {
                                        return true;
                                    };

                                    // debug info
                                    if true {
                                        match command {
                                            CrawlCommand::Request(request) => {
                                                println!(
                                                    "view_command request: {} ({})",
                                                    request.source, request.depth
                                                );
                                            }
                                            CrawlCommand::Terminate => {
                                                println!("view_command terminate");
                                                return true;
                                                // TODO: notify crawler
                                            }
                                        }
                                    }
                                }
                                AppRequest::Markdown(url) => {
                                    // TODO:
                                }
                            }
                        }
                        Err(err) => {
                            println!("view_command err: {}", err);
                            return true;
                        }
                    }

                    false
                })
                .wait();

            if to_break {
                let _ = crawler_command_tx.send(CrawlCommand::Terminate);
                println!("breaking free of the App event_loop");
                break;
            }
        }
    }
}
