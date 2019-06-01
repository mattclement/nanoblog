use tera::Tera;
use tide::http;
use tide::{Context, Route};

lazy_static! {
    pub static ref TERA: Tera = { compile_templates!("templates/**/*.html") };
}

pub async fn render<AppData: Clone + Send + Sync + 'static>(
    cx: Context<AppData>,
) -> http::Response<String> {
    let mut tera_ctx: tera::Context = tera::Context::new();
    let page: String = cx.param("post").expect("Page Not Found");
    tera_ctx.insert("post", &page);
    let body = TERA.render("index.html", &tera_ctx).unwrap();
    http::Response::builder()
        .status(http::status::StatusCode::OK)
        .header("Content-Type", "text/html; charset=UTF-8")
        .body(body)
        .unwrap()
}

pub fn routes<Data: Clone + Send + Sync + 'static>(router: &mut Route<Data>) {
    router.at("/:post").get(render);
}
