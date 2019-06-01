#![feature(async_await)]
#![feature(duration_float)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate tera;

mod metrics;
mod posts;

fn main() -> std::io::Result<()> {
    let mut app = tide::App::new(());

    app.middleware(tide::middleware::RootLogger::new());
    app.middleware(metrics::PromMetrics::default());
    app.at("/metrics").get(metrics::report);

    app.at("/_version").get(async move |_| "Version: 0.2\n");

    app.at("/posts").nest(posts::routes);

    app.serve("0.0.0.0:8000")
}
