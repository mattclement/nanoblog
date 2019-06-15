#![feature(async_await)]
#![feature(duration_float)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate prometheus;
#[macro_use] extern crate tera;

use std::thread;

mod db;
mod metrics;
mod posts;

fn main() -> Result<(), std::io::Error> {
    let db = db::Database::new();
    let metrics = metrics::PromMetrics::default();
    let mut app = tide::App::new(db);

    // Expose the metrics infornation on a different port (hopefully internal!).
    thread::spawn(move || {
        let mut app = tide::App::new(());
        app.at("/metrics").get(metrics::report);
        let _ = app.serve("0.0.0.0:8000");
    });

    app.middleware(tide::middleware::RootLogger::new());
    app.middleware(metrics);

    app.at("/_health")
        .get(async move |_| format!("{}\n", env!("CARGO_PKG_VERSION")));
    app.at("/").get(posts::list_posts);
    app.at("/:post").get(posts::get_post);

    app.serve("0.0.0.0:80")
}