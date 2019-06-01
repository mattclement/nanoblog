use prometheus::{Encoder, HistogramVec, TextEncoder};

use futures::future::BoxFuture;
use futures::prelude::*;
use tide::{
    middleware::{Middleware, Next},
    Context, Response,
};

lazy_static! {
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["handler"]
    )
    .unwrap();
}

pub struct PromMetrics;

impl Default for PromMetrics {
    fn default() -> Self {
        PromMetrics {}
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for PromMetrics {
    fn handle<'a>(&'a self, cx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let t = std::time::Instant::now();
            let path = cx.uri().path().to_owned();
            let res = next.run(cx).await;
            // Only store the time if it was a recognized route to prevent metrics DoS
            if res.status() != 404 {
                HTTP_REQ_HISTOGRAM
                    .with_label_values(&[&path])
                    .observe(t.elapsed().as_secs_f64());
            }
            res
        })
    }
}

pub async fn report<T>(_: Context<T>) -> String {
    let mut buf = vec![];
    let encoder = TextEncoder::new();
    let metrics_families = prometheus::gather();
    encoder.encode(&metrics_families, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}
