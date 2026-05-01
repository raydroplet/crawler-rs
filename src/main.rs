mod app;
mod gui;

use app::{App};

fn main() {
    let app = App::new();
    app.run();
}
