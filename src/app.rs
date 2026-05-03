use crate::gui::{EguiView};

pub struct App {
    view: EguiView,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: EguiView::new()
            //
        }
    }

    pub fn run(&self) {
        let _result = self.view.run();
        println!("Hello!");
    }
}
