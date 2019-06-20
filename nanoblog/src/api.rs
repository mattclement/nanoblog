use tide::{Context, EndpointResult, Error, http};
use crate::db;
use http::status::StatusCode;
use tide::error::ResultExt;

pub async fn list_posts(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let posts = client.list_posts().await;
    let body = serde_json::to_string(&posts)
        .unwrap_or_default();
    let res = http::Response::builder()
        .status(StatusCode::OK)
        .body(format!("{}\n", body).into())
        .client_err()?;
    Ok(res)
}

pub async fn get_raw_post(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let title = cx.param("post").client_err()?;
    client.get_post(title)
        .await
        .map(|p| {
            let body = serde_json::to_string(&p)
                .unwrap_or_default();
            let res = http::Response::builder()
                .status(StatusCode::OK)
                .body(format!("{}\n", body).into())
                .expect("Error unwrapping raw post body");
            Ok(res)
        })
        .map_err(|e| {
            let res = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(e.into())
                .expect("Error generating error response");
            Error::from(res)
        })?
}

pub async fn upsert_post(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let title = cx.param("post").client_err()?;
    let body = cx.body_string().await.client_err()?;

    let post: db::Post = match client.get_post(title).await {
        Ok(p) => {
            p.date_updated = Some("now".into());
            p
        },
        Err(_) => {
            db::Post {
                title,
                body,
                date_created: "now".into(),
                date_updated: None,
            }
        }
    };

    client
        .save_post(post)
        .await
        .map(|p| {
            let body = serde_json::to_string(&p)
                .unwrap_or_default();
            let res = http::Response::builder()
                .status(StatusCode::OK)
                .body(format!("{}\n", body).into())
                .expect("Error unwrapping raw post body");
            Ok(res)
        })
        .map_err(|e| {
            let res = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(e.into())
                .expect("Error generating error response");
            Error::from(res)
        })?
}
