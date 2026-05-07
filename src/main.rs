mod app;
mod gui;
mod crawler;

use app::{App};

fn main() {
    let app = App::new();
    app.run();
}
