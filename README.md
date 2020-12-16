# Warning
`bytes-buf` feature can produce **SIGILL** https://github.com/botika/v_escape/issues/54. It's complitate because the `avx` and `sse` flags are in almost all cpus of `x86` and `x86_64` architectures. I have already warned. More details in the issue.

On `tokio@0.2` (`actix`, `hyper`, `ntex`, ...) use `yarte@0.12`. 
On `tokio@0.3` use `yarte@0.13`. 
In `yarte@0.14` there will be both options.

# Disclaimer
All structures and functions in this crate -- even those based on the real crates -- are entirely fictional. 
All celebrity codes are impersonated...poorly.
The following program contains coarse language and due to its content it should not be used by anyone.

# Yarte [![Latest version](https://img.shields.io/crates/v/yarte.svg)](https://crates.io/crates/yarte) [![Build Status](https://travis-ci.org/botika/yarte.svg?branch=master)](https://travis-ci.org/botika/yarte)
Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine. Uses a Handlebars-like syntax, 
well-known and intuitive for most developers. Yarte is an optimized, and easy-to-use 
rust crate, with which developers can create logic around their 
HTML templates using conditionals, loops, rust code and template composition. 

## Features
- Almost all Rust expressions are valid
- Meta programming system with almost all Rust expressions, conditionals, loops, modules and partial recursion
- Low level, SIMD and zero copy runtime
- A [fancy-text debug](https://asciinema.org/a/TQAodSQXevgHgO01vzC6vdo6v?autoplay=1) mode to visualize the code generated by Yarte
- Emit snipped annotations at error
- Improved daily and has full coverage (without stupid bugs that take months or years to fix)

### Is it really fast?
See it for yourself in the [TechEmpower benchmarks][bench] with [`actix`][actix] and [`ntex`][ntex] 

## Documentation
In order to  fully understand Yarte's capabilities take a look at the following documentation:
- [Tests](./yarte/tests)
- [Our book](https://yarte.netlify.com/)
- [Crate documentation](https://docs.rs/yarte/)
- Minimum supported Rust version: 1.45 or later

## Acknowledgment
Yarte is based on all previous templates engines, syntax as well as its documentation 
is highly influenced by [Handlebars][handlebars]. 

[bench]: https://tfb-status.techempower.com/
[handlebars]: https://handlebarsjs.com/
[ntex]: https://github.com/ntex-rs/ntex
[actix]: https://github.com/actix/actix-web

## Contributing
Please, contribute to Yarte! The more the better! Feel free to open an issue and/or contacting directly with the 
owner for any request or suggestion.

### Code of conduct
This Code of Conduct is adapted from the [Contributor Covenant][homepage], version 1.4, available at [http://contributor-covenant.org/version/1/4][version]

[homepage]: http://contributor-covenant.org
[version]: http://contributor-covenant.org/version/1/4/

### License
This project is distributed under the terms of both the Apache License (Version 2.0) and the MIT license, specified in 
[LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) respectively.
