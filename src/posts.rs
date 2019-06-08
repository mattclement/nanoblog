use tera::Tera;
use tide::http;
use tide::{Context, Route};

use crate::db;

lazy_static! {
    pub static ref TERA: Tera =
        { compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")) };
}

pub fn res_404() -> http::Response<String> {
    let tera_ctx: tera::Context = tera::Context::new();
    let body = TERA.render("404.html", &tera_ctx).unwrap();
    http::Response::builder()
        .status(http::status::StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=UTF-8")
        .body(body)
        .unwrap()
}

pub async fn get_post(cx: Context<db::Database>) -> http::Response<String> {
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

    let body = TERA.render("index.html", &tera_ctx).unwrap();
    http::Response::builder()
        .status(http::status::StatusCode::OK)
        .header("Content-Type", "text/html; charset=UTF-8")
        .body(body)
        .unwrap()
}

pub fn routes(router: &mut Route<db::Database>) {
    router.at("/:post").get(get_post);
}
