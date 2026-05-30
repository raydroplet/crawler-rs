use crawler_rs::app::{App};

fn main() {
    let app = App::new();

    match app.run() {
        Ok(_) => {
            //
        }
        Err(err) => {
            eprintln!("App error: {}", err);
        }
    }
}
