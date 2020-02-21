# `with-actix-web` feature


Yarte implements template with `actix-web`'s `Responder` trait to make it easier for the user to incorporate templates
in this framework. This is done using feature `with-actix-web` of your `Cargo.toml`:

```toml
yarte = { version = "0.7", features = ["with-actix-web"]  }
```

For example, let's create an api with `actix-web` that will serve a template `index` when the root url is called.

The template will have no context in this case and will look this:

```handlebars
{{! Simple example !}}
{{> doc/t ~}}
<html>
{{~> doc/head title = "Actix web" ~}}
<body>
  {{~#if let Some(name) = query.get("name") }}
      {{ let lastname = query.get("lastname").ok_or(yarte::Error)? }}
  {{/if ~}}
</body></html>

```

This template will use the following struct defined in a file later refer as :

```rust
use std::collections::HashMap;

use actix_web::{get, middleware::Logger, web, App, HttpServer, Responder};
use yarte::Template;

#[derive(Template)]
#[template(path = "index.hbs", err = "error message")]
struct IndexTemplate {
    query: web::Query<HashMap<String, String>>,
}

#[get("/")]
pub fn index(query: web::Query<HashMap<String, String>>) -> impl Responder {
    IndexTemplate { query }
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // start http server
    HttpServer::new(move || App::new().wrap(Logger::default()).service(index))
        .bind("127.0.0.1:8080")?
        .run()
}
```

