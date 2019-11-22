#![cfg(feature = "with-actix-web")]

use actix_web::{get, http, test, web::Bytes, App, Responder};
use yarte::Template;

#[derive(Template)]
#[template(path = "hello.hbs")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[get("/")]
pub fn index() -> impl Responder {
    HelloTemplate { name: "world" }
}

#[test]
fn test_actix_web() {
    let mut app = test::init_service(App::new().service(index));

    let req = test::TestRequest::with_uri("/").to_request();
    let resp = test::call_service(&mut app, req);

    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let bytes = test::read_body(resp);
    assert_eq!(bytes, Bytes::from_static("Hello, world!".as_ref()));
}
