use crate::gui::EguiView;
use std::collections::HashSet;
use tokio::sync::{mpsc};

struct ParserResult;

pub struct App {
    view: EguiView,
    // crawler: WebCrawler,
    //
    pages: HashSet<ParserResult>,
}

impl App {
    pub fn new() -> Self {
        //// (CrawlCommand, ParserResult)
        // let (tx, mut rx) = mpsc::channel(1024);
        // let crawler = Arc::new(WebCrawler::new(tx).expect("Failed to initialize crawler"));

        Self {
            view: EguiView::new(), //
            pages: HashSet::new(),
        }
    }

    pub fn run(&self) {
        let _result = self.view.run();
        println!("Hello!");
    }
}
