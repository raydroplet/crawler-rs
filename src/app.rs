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
                self.event_loop(view_command_rx, crawler_response_rx);
            });

            let _ = ViewEgui::run(view);
        });

        Ok(())
    }

    fn event_loop(&self,
        // NOTE: &self here
        view_command_rx: flume::Receiver<CrawlCommand>,
        crawler_response_rx: flume::Receiver<CrawlResponse>,
    ) {
        loop {
            let to_break = false;
            flume::Selector::new()
                .recv(&crawler_response_rx, |message| {
                    match message {
                        Ok(response) => match response {
                            CrawlResponse::Page(page) => {
                                //
                            }
                            CrawlResponse::Queued(url) => {
                                //
                            }
                        },
                        Err(err) => {}
                    }

                    to_break
                })
                .recv(&view_command_rx, |message| {
                    match message {
                        Ok(command) => match command {
                            CrawlCommand::RequestCrawl(request) => {
                                println!(
                                    "view_command request: {} ({})",
                                    request.source, request.depth
                                );
                            }
                            CrawlCommand::Terminate => {
                                println!("view_command terminate");
                                // TODO: notify crawler
                            }
                        },
                        Err(err) => {
                            println!("view_command err: {}", err);
                        }
                    }

                    to_break
                })
                .wait();

            if to_break {
                break;
            }
        }
    }
}
