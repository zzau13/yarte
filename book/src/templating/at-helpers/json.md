# Json
You can serialize json in your template with [`serde::Serialize`](https://docs.serde.rs/serde/trait.Serialize.html)
```rust
use serde::Serialize;
#[derive(Template)]
#[template(path = "foo")]
struct Foo<S: Serialize> {
    foo: S
}
```

```handlebars
{{ @json foo }}
```

```handlebars
{{ @json_pretty foo }}
```

Don't escape html characters. 

If you are looking to paint it as html text (like "Text" in `<h1>Text</h1>`):
```handlebars
<h1>{{ serde_json::to_string(&foo).map_err(|_| yarte::Error)? }}</h1>
```