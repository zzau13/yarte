<br/>

<a href="https://commons.wikimedia.org/wiki/File:Logo_yarte.png">
<img align="left" src="https://upload.wikimedia.org/wikipedia/commons/b/bb/Logo_yarte.png" alt="Yet Another Rust Template Engine" width="200" height="200">
</a>

# Yarte [![Latest version](https://img.shields.io/crates/v/yarte.svg)](https://crates.io/crates/yarte) [![Netlify Status](https://api.netlify.com/api/v1/badges/1ccce8b0-cb08-41b1-a781-f883a6cc7767/deploy-status)](https://app.netlify.com/sites/yarte/deploys)
Yarte stands for **Y**et **A**nother **R**usty **T**emplate **E**ngine. Uses a Handlebars-like syntax, 
well-known and intuitive for most developers. Yarte is an optimized, and easy-to-use 
rust crate, with which developers can create logic around their 
HTML templates using conditionals, loops, rust code and template composition. 

<br/>
<br/>


## Documentation
In order to  fully understand Yarte's capabilities take a look at the following documentation:
- [Tests](./yarte/tests)
- [Our book](https://yarte.netlify.com/)
- [Crate documentation](https://docs.rs/yarte/)

Or, in nightly, just:
```rust
#[yarte] "{{> my_template }}"
```
`bytes-buf` feature can produce **SIGILL**.
More details in https://github.com/botika/v_escape/issues/54.

Yarte is under development.

### Is it really fast?
Run `cargo bench` with rust nightly.

Results in my `AMD Ryzen 9 5900HX`
```
Teams                   time:   [62.335 ns 62.704 ns 63.138 ns]
Big table               time:   [28.546 µs 28.690 µs 28.873 µs]
```

See it for yourself in the [TechEmpower benchmarks][bench] with [`actix`][actix] and [`ntex`][ntex]

## Acknowledgment
Yarte is based on all previous templates engines, syntax as well as its documentation 
is highly influenced by [Handlebars][handlebars]. 
Logo adapted from [Creative Commons][commons] images

[bench]: https://tfb-status.techempower.com/
[handlebars]: https://handlebarsjs.com/
[ntex]: https://github.com/ntex-rs/ntex
[actix]: https://github.com/actix/actix-web
[commons]: https://commons.wikimedia.org

### Code of conduct
This Code of Conduct is adapted from the [Contributor Covenant][homepage], version 1.4, available at [http://contributor-covenant.org/version/1/4][version]

[homepage]: http://contributor-covenant.org
[version]: http://contributor-covenant.org/version/1/4/

### License
This project is distributed under the terms of both the Apache License (Version 2.0) and the MIT license, specified in 
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) respectively.

# Disclaimer
Spain is not a democracy, with your tourism or your purchases you are subsidizing this barbarism. 

All structures and functions in this crate -- even those based on the real crates -- are entirely fictional. 
All celebrity codes are impersonated...poorly.
The following program contains coarse language and due to its content it should not be used by anyone.

exile or hemlock
