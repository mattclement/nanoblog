#![feature(async_await)]
#![feature(duration_float)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate prometheus;
#[macro_use] extern crate tera;

use std::thread;

mod api;
mod db;
mod middleware;
mod posts;


fn main() -> Result<(), std::io::Error> {
    let db = db::Database::new();
    let metrics = middleware::PromMetrics::default();
    let bearer_protection = middleware::BearerAuth::default();
    let mut app = tide::App::new(db);

    // Expose the metrics infornation on a different port (hopefully internal!).
    thread::spawn(move || {
        let mut app = tide::App::new(());
        app.at("/metrics").get(middleware::report);
        let _ = app.serve("0.0.0.0:8000");
    });

    app.middleware(tide::middleware::RootLogger::new());
    app.middleware(metrics);
    app.middleware(bearer_protection);

    app.at("/api").nest(|router| {
        router.at("/ping").get(async move |_| "OK\n");
        router.at("/posts").get(api::list_posts);
        router.at("/posts/:post").get(api::get_raw_post);
        router.at("/posts/:post").post(api::upsert_post);
    });

    app.at("/_health")
        .get(async move |_| format!("{}\n", env!("CARGO_PKG_VERSION")));

    app.at("/").get(posts::list_posts);
    app.at("/:post").get(posts::get_post);

    app.serve("0.0.0.0:80")
}
