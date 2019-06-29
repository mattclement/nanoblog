use tera::Tera;
use tide::http;
use tide::{Context, EndpointResult, Error, error::ResultExt};

use crate::db;
use http::status::StatusCode;
use pulldown_cmark::{Parser, Options, html};

const INDEX: &str = "index.html";
const POST: &str = "post.html";
const NOT_FOUND: &str = "404.html";

lazy_static! {
    pub static ref TERA: Tera =
    { compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")) };
}


pub fn res_404() -> EndpointResult {
    let tera_ctx: tera::Context = tera::Context::new();
    let body = TERA.render(NOT_FOUND, &tera_ctx).unwrap();
    let resp = http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(http::header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .body(body.into())
        .unwrap();
    Ok(resp)
}


/// Render will apply the tera template context to the template and wrap it in a
/// EndpointResult
fn render(template: &str, tera_ctx: tera::Context) -> EndpointResult {
    let body = TERA.render(template, &tera_ctx)
        .map_err(|e| {
            let resp = http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(http::header::CONTENT_TYPE, "text/html; charset=UTF-8")
                .body(format!("<html><body>{:?}</body></html>", e).into())
                .expect("Failed to build failure response");
            Error::from(resp)
        })?;

    let resp = http::Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .body(body.into())
        .expect("Error building response");
    Ok(resp)
}

/// Render markdown contents
fn render_markdown(contents: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(contents, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}


pub async fn list_posts(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let mut tera_ctx: tera::Context = tera::Context::new();
    let contents = client.list_posts().await;

    let mut contents: Vec<(String, db::PostMetadata)> = contents
        .into_iter()
        .collect();

    contents.sort_by_key(|k| k.1.date_created.clone());

    tera_ctx.insert("post_links", &contents);
    render(INDEX, tera_ctx)
}


pub async fn get_post(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let mut tera_ctx: tera::Context = tera::Context::new();

    let title: String = cx.param("post").client_err()?;

    let contents = client.get_post(title.clone()).await;
    if contents.is_err() {
        return res_404();
    }
    let contents = contents.unwrap();

    tera_ctx.insert("title", &contents.title);
    tera_ctx.insert("date_created", &contents.date_created);
    tera_ctx.insert("body", &render_markdown(&contents.body));
    render(POST, tera_ctx)
}
