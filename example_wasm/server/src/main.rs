#![allow(clippy::into_iter_on_ref)]

use serde::Serialize;
use yarte::Template;

use actix_web::{get, middleware::Logger, App, HttpServer, Responder};

use model::Fortune;

#[derive(Template, Serialize)]
#[template(
    path = "fortune.hbs",
    print = "code",
    mode = "iso",
    script = "./client.js"
)]
pub struct Test {
    fortunes: Vec<Fortune>,
    head: String,
}

#[get("/")]
async fn index() -> impl Responder {
    Test {
        fortunes: vec![Fortune {
            id: 0,
            message: "foo".to_string(),
        }],
        head: "bar".to_string(),
    }
}

// TODO: serve files
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // start http server
    HttpServer::new(move || App::new().wrap(Logger::default()).service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
