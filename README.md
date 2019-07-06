# Yarte [![Documentation](https://docs.rs/yarte/badge.svg)](https://docs.rs/yarte/) [![Latest version](https://img.shields.io/crates/v/yarte.svg)](https://crates.io/crates/yarte)
Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine, 
is the fastest template engine. Uses a Handlebars-like syntax, 
well known and intuitive for most developers. Yarte is an optimized, and easy-to-use 
rust crate, with which developers can create logic around their 
HTML templates using using conditionals, loops, rust code 
and using templates composition. 

## Features
- Yarte incorporates feature `with-actix-web`, an 
implementation of `actix-web`'s trait Responder for those using this framework.
- Ignores unnecessary conditionals (`if` and `unless`) from users code.
- Evaluation of rust expression using `eval` function.
- A fancy-text debug mode to visualize the code generated by Yarte.

## Documentation
In order to  fully understand Yarte's capabilities take a look at the following documentation:
 - [Our book](https://yarte.netlify.com/)
 - [Crate documentation](https://docs.rs/yarte/).
 - [Tests](./yarte/tests)

## Getting started
Follow with the help of code in `example` directory.

Add Yarte dependency to your Cargo.toml file:

```toml
[dependencies]
yarte = "0.2"
```
Yarte templates look like regular text, with embedded yarte expressions. 
Create a simple Yarte template called `hello.html` in your template directory.

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
#[template(path = "hello.html")]
struct CardTemplate<'a> {
    title: &'a str,
    body: &'a str,
}
```
Yarte will read `hello.html` and build a parser for the template at compile time,
that can be later applied to any `CardTemplate` object.

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

## Roadmap
- [ ] intellij plugin
- [ ] error report
- [ ] Minimize html5, css and js
- [ ] Derive builders for generate defined helpers and filters
- [ ] `>|` filters on fmt::Formatter
- [ ] Concatenate filters, unix like, on fmt::Formatter (when is possible)
- [ ] ... you can open a issue!

We are not looking for anything other than render HTML5 and text as fast as possible. 
You can open a pull request in another case.

## Acknowledgment
Yarte is based on all previous templates engines, syntax as well as its documentation 
is highly influenced by [Handlebars][handlebars]. Implemented mainly with `nom`, 
`memchr` and `syn`, among others crates . As many ideas as possible used in 
Yarte are from other repositories. Comments in the code clarify which ideas are used, 
and  from where.


##### Is it really the fastest?
 See it for yourself in the [benchmarks][bench]!

[bench]: https://gitlab.com/botika/template-benchmarks-rs
[handlebars]: https://handlebarsjs.com/ 

## Contributing

Please, contribute to Yarte! The more the better! Feel free to to open an issue and/or contacting directly with the 
owner for any request or suggestion.


## Code of conduct
This Code of Conduct is adapted from the [Contributor Covenant][homepage], version 1.4, available at [http://contributor-covenant.org/version/1/4][version]

[homepage]: http://contributor-covenant.org
[version]: http://contributor-covenant.org/version/1/4/

## License
This project is distributed under the terms of both the Apache License (Version 2.0) and the MIT license, specified in 
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) respectively.

## Support
[Patreon][patreon]

[patreon]: https://www.patreon.com/r_iendo 