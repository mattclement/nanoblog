use tide::{Context, EndpointResult, Error, http};
use crate::db;
use http::status::StatusCode;
use tide::error::ResultExt;
use chrono::Local;

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
    let title: String = cx.param("post").client_err()?;
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


pub async fn upsert_post(mut cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let post = cx.body_json::<db::NewPost>().await.client_err()?;
    let now = Local::today().format("%F").to_string();

    let draft = cx.uri().query()
        .unwrap_or_default()
        .contains("draft=true");

    let mut post: db::Post = post.into();
    if let Ok(p) = client.get_post(post.slug.clone()).await {
        post.date_created = p.date_created;
        post.date_updated = Some(now);
    }

    let res = client.save_post(post.clone())
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
        })?;

    if draft {
        return res
    }

    client.activate_post(post.into())
        .await
        .map_err(|e| {
            let err = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(e.into())
                .expect("Error generating error response");
            Error::from(err)
        })?;

    res
}
