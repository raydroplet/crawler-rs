mod app;
mod crawler;
mod gui;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}
