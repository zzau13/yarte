# Getting started

Follow with the help of code in `example` directory.

Add Yarte dependency to your Cargo.toml file:

```toml
[dependencies]
yarte = "0.10"
```
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


In order to use a struct in a Yarte template  you will have to call 
the procedural macro `Template`. For example, in the following 
code we are going to use struct `CardTemplate`, to then 
define `template` as a `CardTemplate` with content. 

```rust
use yarte::Template;

#[derive(Template)]
#[template(path = "hello")]
struct CardTemplate<'a> {
    title: &'a str,
    body: &'a str,
}
```

```rust
let template = CardTemplate {
    title: "My Title",
    body: "My Body",
};
```

In this case `template` is an object `CardTemplate` correctly defined, so now `template` 
can be rendered using function `self.call()` and call your template to allocate the 
result in a `String` and return it wrapped with yarte::Result.

```rust
template.call()
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
