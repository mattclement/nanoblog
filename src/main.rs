#![feature(async_await)]
#![feature(duration_float)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate tera;


mod db;
mod metrics;
mod posts;


fn main() -> Result<(), std::io::Error> {
    let db = db::Database::new();
    let mut app = tide::App::new(db);

    app.middleware(tide::middleware::RootLogger::new());
    app.middleware(metrics::PromMetrics::default());

    app.at("/_version").get(async move |_| "0.2\n");
    app.at("/_health").get(metrics::report);
    app.at("/metrics").get(metrics::report);
    app.at("/posts").nest(posts::routes);

    app.serve("0.0.0.0:8000")
}
