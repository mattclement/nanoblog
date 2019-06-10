use tera::Tera;
use tide::http;
use tide::{Context, Route, EndpointResult, Error};

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

pub async fn get_post(cx: Context<db::Database>) -> EndpointResult {
    let client = cx.app_data().to_owned();
    let mut tera_ctx: tera::Context = tera::Context::new();

    let title: Result<String, _> = cx.param("post");
    if title.is_err() {
        return res_404();
    }
    let title = title.unwrap();

    let contents = client.get_post(title.clone()).await;
    if contents.is_empty() {
        return res_404();
    }

    tera_ctx.insert("title", &title);
    tera_ctx.insert("contents", &contents);

    let body = TERA.render("index.html", &tera_ctx)
        .map_err(|_| {
            let resp = http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error encoding metrics".into())
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

pub fn routes(router: &mut Route<db::Database>) {
    router.at("/:post").get(get_post);
}
