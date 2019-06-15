use tera::Tera;
use tide::http;
use tide::{Context, EndpointResult, Error, error::ResultExt};

use crate::db;
use http::status::StatusCode;

lazy_static! {
    pub static ref TERA: Tera =
    { compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")) };
}

pub fn res_404() -> EndpointResult {
    let tera_ctx: tera::Context = tera::Context::new();
    let body = TERA.render("404.html", &tera_ctx).unwrap();
    let resp = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(http::header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .body(body.into())
        .unwrap();
    Ok(resp)
}


fn render(template: &str, tera_ctx: tera::Context) -> EndpointResult {
    let body = TERA.render(template, &tera_ctx)
        .map_err(|e| {
            let resp = http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("{:?}", e).into())
                .expect("Failed to build metrics encoding error");
            Error::from(resp)
        })?;

    let resp = http::Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .body(body.into())
        .expect("Error building response");
    Ok(resp)
}


pub async fn list_posts(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let mut tera_ctx: tera::Context = tera::Context::new();
    let contents: Vec<String> = client.list_posts().await;
    tera_ctx.insert("post_links", &contents);
    render("index.html", tera_ctx)
}


pub async fn get_post(cx: Context<db::Database>) -> EndpointResult {
    eprintln!("Test");
    let client = cx.app_data().to_owned();
    let mut tera_ctx: tera::Context = tera::Context::new();

    let title: String = cx.param("post").client_err()?;

    let contents = client.get_post(title.clone()).await;
    if contents.is_empty() {
        return res_404();
    }

    tera_ctx.insert("title", &title);
    tera_ctx.insert("contents", &contents);
    render("post.html", tera_ctx)
}
