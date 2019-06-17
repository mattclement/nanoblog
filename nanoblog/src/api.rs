use tide::{Context, EndpointResult, Error, error::ResultExt};
use crate::db;
use http::status::StatusCode;


pub async fn list_posts(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let title: String = cx.header("post").client_err()?;
    let contents = client.validate_token().await;
}
