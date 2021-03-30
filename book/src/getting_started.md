# Getting started

As yarte files are not compiler-specific, you have to add: 

#### `Cargo.toml`
```toml
[build-dependencies]
yarte = "0.15"
```

#### `build.rs`
```rust
fn main() {
    yarte::recompile::when_changed();
}
```
In order to recompile when something in the template directory changed ;

Yarte templates look like regular text, with embedded yarte expressions. 
Create a simple Yarte template called `hello.hbs` in your template directory.

```handlebars
<div class="entry">
  <h1>{{title}}</h1>
  <div class="body">
    {{body}}
  </div>
</div>
```

```rust
use yarte::*;

struct Card<'a> {
    title: &'a str,
    body: &'a str,
}

fn foo() -> String {
    let my_card = Card {
        title: "My Title",
        body: "My Body",
    };

    auto!(ywrite_html!(String, "{{> hello my_card }}"))
}
```

will write in the formatter the following string:
```html
<div class="entry">
  <h1> My Title </h1>
  <div class="body">
    My Body
  </div>
</div>
```
