use crate::crawler::{CrawlCommand, CrawlResponse, ParserResult, WebCrawler};
use crate::gui::ViewEgui;
use std::collections::HashSet;
use std::error::Error;
use std::thread;

pub struct App {
    pages: HashSet<ParserResult>,
}

impl App {
    pub fn new() -> Self {
        Self {
            pages: HashSet::new(),
            //
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        //// channels
        let (crawler_response_tx, mut crawler_response_rx) = flume::bounded(1024);
        let (crawler_command_tx, mut crawler_command_rx) = flume::bounded(8);
        let (view_response_tx, mut view_response_rx) = flume::bounded(1024);
        let (view_command_tx, mut view_command_rx) = flume::bounded(1024);

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
                    crawler_command_tx.clone(),
                    crawler_response_rx,
                );
            });

            let _ = ViewEgui::run(view);
        });

        Ok(())
    }

    fn event_loop(
        &self,
        // NOTE: &self here
        view_command_rx: flume::Receiver<CrawlCommand>,
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
                                    page.domain,
                                    page.discovered_links.len()
                                );
                                //
                            }
                            CrawlResponse::Queued(url) => {
                                println!("queued page: {}", url);
                                //
                            }
                        },
                        Err(err) => {
                            println!("crawler_response err: {}", err);
                            return true;
                        }
                    }

                    false
                })
                .recv(&view_command_rx, |message| {
                    match message {
                        Ok(command) => {
                            let _ = crawler_command_tx.send(command.clone());

                            // debug info
                            if true {
                                match command {
                                    CrawlCommand::RequestCrawl(request) => {
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
                        Err(err) => {
                            println!("view_command err: {}", err);
                            return true;
                        }
                    }

                    false
                })
                .wait();

            if to_break {
                println!("breaking free of the App event_loop");
                break;
            }
        }
    }
}
