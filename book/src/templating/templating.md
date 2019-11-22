# Templating

Yarte uses opening characters `{{` and closing 
characters `}}` to parse the inside depending 
on the feature used. Most of the features are 
defined by Handlebars such as paths, comments, 
html, helpers and partials. Others such as 
adding rust code to a template, are obviously 
defined by Yarte. Each of these features have
a symbol associated to it (`# { R >`) that is
added after the opening characters, for example
`{{#` used for helpers. If no symbol is added 
Yarte will interpret  inside code as a valid 
rust expression.

Let's say we want to use the following template `template.html`
```html
<h1> Hello, {{name}}! </h1>
```
Now we create a struct with the variable `name`
```rust
#[derive(Template)]
#[template(path = "template.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}
```
If we now render the template with `"world"` as value of `name`,
```rust
HelloTemplate { 
    name: "world" 
}
.call().unwrap()
```