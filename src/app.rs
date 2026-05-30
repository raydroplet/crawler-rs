use crate::crawler::WebCrawler;
use crate::gui::ViewEgui;
use crossbeam_channel as crossbeam;
use std::collections::HashSet;
use std::thread;
use std::error::Error;
use tokio::sync::mpsc;

struct ParserResult;

pub struct App {
    pages: HashSet<ParserResult>,
}

impl App {
    pub fn new() -> Self {
        Self {
            pages: HashSet::new(),
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        //// channels
        let (crawler_response_tx, mut crawler_response_rx) = mpsc::channel(1024);
        let (crawler_command_tx, mut crawler_command_rx) = mpsc::channel(8);
        let (app_response_tx, mut app_response_rx) = crossbeam::bounded(1024);
        let (app_command_tx, mut app_command_rx) = crossbeam::bounded(1024);

        //// actors
        let view = ViewEgui::new(app_response_rx, app_command_tx);
        let crawler = WebCrawler::new()?;

        //// threads + tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        thread::spawn(move || {
            runtime.block_on(async {
                crawler.run(crawler_command_rx, crawler_response_tx).await;
            });
        });

        // run the view on the main thread
        let _ = ViewEgui::run(view);

        Ok(())
    }
}
