# Description

Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine, 
is the fastest template engine. Uses a Handlebars-like syntax, 
well known and intuitive for most developers. Yarte is an optimized, and easy-to-use 
rust crate, with which developers can create logic around them 
HTML templates using conditionals, loops, rust code 
and using templates composition with partials.

Yarte is intended to be more than just a substitute for PHP, I want to encompass 
all GUI types with a front end based on Handlebars, HTML5 and Rust.

The process is tedious and very complicated, so certain expendable parts are omitted 
and will be subject to several refactors before first release.

## Derive attributes
- `src`: template sources
- `path`: path to sources relative to template directory
- `print`: `all`, `ast` or `code` display debug info. Overridden by config file print option.
- `recursion`: `default: 128` Set limits of partial deep, can produce stackoverflow at compile time

## Rules
- Only use `}}` or `{{\` for expressions or blocks (If you want to use them in any place, you are free to implement a tokenizer that includes the syntax of yarte and rust and do PR)
- Default template extension is `hbs`, since it includes much of the language (If you want to better IDE support, you are free to write a plugin with `yrt` extension)
