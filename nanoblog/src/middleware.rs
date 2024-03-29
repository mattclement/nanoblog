use prometheus::{Encoder, HistogramVec, IntCounterVec, TextEncoder};

use futures::future::BoxFuture;
use futures::prelude::*;
use tide::{
    http, Error,
    EndpointResult,
    middleware::{Middleware, Next},
    Context, Response,
};

use crate::db::Database;

lazy_static! {
    static ref LATENCY: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["handler"]
    )
    .unwrap();
    static ref RES_STATUS: IntCounterVec = register_int_counter_vec!(
        "http_res_status_by_handler",
        "Handler HTTP status codes",
        &["handler", "status_code"]
    )
    .unwrap();

    static ref UNAUTHORIZED: http::Response<String> = http::Response::builder()
        .status(http::StatusCode::UNAUTHORIZED)
        .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body("Invalid bearer token.".into())
        .unwrap();
}

pub struct PromMetrics;

impl Default for PromMetrics {
    fn default() -> Self {
        PromMetrics {}
    }
}

impl<T: Send + Sync + 'static> Middleware<T> for PromMetrics {
    fn handle<'a>(&'a self, cx: Context<T>, next: Next<'a, T>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let t = std::time::Instant::now();
            let path = cx.uri().path().to_owned();

            let res = next.run(cx).await;

            let status = res.status();
            // Only store the info if it was a recognized route to prevent metrics DoS
            if status != 404 {
                RES_STATUS
                    .with_label_values(&[&path, &status.as_u16().to_string()])
                    .inc();
                LATENCY
                    .with_label_values(&[&path])
                    .observe(t.elapsed().as_secs_f64());
            }
            res
        })
    }
}

pub async fn report<T>(_: Context<T>) -> EndpointResult {
    let mut buf = vec![];
    let encoder = TextEncoder::new();
    let metrics_families = prometheus::gather();
    encoder.encode(&metrics_families, &mut buf)
        .map_err(|_| {
            let resp = http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error encoding metrics".into())
                .expect("Failed to build metrics encoding error");
            Error::from(resp)
        })?;

    let body = String::from_utf8(buf).map_err(|_| {
        let resp = http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body("Metrics are invalid UTF-8".into())
            .expect("Failed to build metrics utf-8 error");
        Error::from(resp)
    })?;

    let resp = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body.into())
        .unwrap();
    Ok(resp)
}


pub struct BearerAuth {
    pub db: Database
}

impl Default for BearerAuth {
    fn default() -> Self {
        Self {
            db: Database::new(),
        }
    }
}

fn unauthorized() -> http::Response<http_service::Body> {
    http::Response::builder()
        .status(http::StatusCode::UNAUTHORIZED)
        .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body("Invalid bearer token.\n".into())
        .unwrap()
}

impl<T: Send + Sync + 'static> Middleware<T> for BearerAuth {
    fn handle<'a>(&'a self, cx: Context<T>, next: Next<'a, T>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let path = cx.uri();

            if path.path().contains("bearer_tokens") {
                return unauthorized();
            }

            if !path.path().starts_with("/api") {
                return next.run(cx).await
            }

            let headers = cx.headers();
            let authz = headers.get(http::header::AUTHORIZATION);

            if let Some(val) = authz {
                let val = val.to_str().unwrap_or("");
                if !val.starts_with("Bearer ") {
                    return unauthorized();
                }
                let token = val.replace("Bearer ", "");
                if !self.db.clone().validate_token(token).await {
                    return unauthorized();
                }
                next.run(cx).await
            } else {
                return unauthorized();
            }

        })
    }
}
