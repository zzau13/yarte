# Changes
### [0.14.0] (2021-xx-xx)
### Refactor
- Add abstract lexer 

### [0.13.0] (2020-10-25)
## Update
- `buf-min` version to `0.2`
- Parser coverage

## Refactor
- Move App runtime to delorean-rs

### [0.12.0] (2020-07-05)
## Added
- Json serializer
- Use `buf-min::Buffer` as default bytes buffer

## [0.11.0] (2020-06-22)
## Added
- `TemplateBytesTrait` for render to `&mut BytesMut`
- sse2 iota over `BytesMut` 
- `ccall` for consume struct

## [0.10.0] (2020-06-15)
## Added
- `TemplateFixedTrait` for render to `&mut [u8]`
- Use `dtoa` and `itoa` for render number
- Escape char

## Update
- `v_htmlescape` to 0.7

## [0.9.0] (2020-05-05)
## Added
- compile error at derive struct

## Update
- `v_eval` to 0.5
- use `proc_macro2` force fallback

## Fixed
- default recursion at 128
- change attribute `recursion-limit` to `recursion`

## [0.8.0] (2020-04-24)
## Added
- `{{ @json obj }}` and `{{ @json_pretty obj }}` @helpers 
- User compile error `{{$ "message" }}`

## Fixed
- Annotate snippets error messages for parse and lowering

## Refactor
- Remove `mode` in favor of features and alias
- Remove `with-actix-web` feature
- Remove `ext`, `err` attributes
- Unique path for templates `hbs`

## Update
- `annotate-snippets` to `0.8`
- `prettyprint` to `0.8`

## [0.7.0] (2020-02-21)
## Fixed 
- Use `html` mode by default
- Unnecessary alloc

## Refactor
- Remove yarte_template crate
- Add new derive App
- Move wasm app derive and helpers to yarte_wasm_app
- Rename features

## [0.6.0] (2020-02-03)
### Added
- Html minifier 
- [Partial Block](https://handlebarsjs.com/guide/partials.html#partial-blocks)
- Recursion in partial and partial-block
- `recursion-limit` attribute
- Resolve expression `{{? expr }}`
- Add `server` mode for wasm app server site

### Refactor 
- Split `Template` trait

## [0.5.0] (2019-12-30)
### Added
- Annotate snippets error message
- Async core for wasm applications

### Updated
- Remove `nom` and `memchr` from dependencies
- `actix-web` version to `2.0.0`

### Refactor
- Parser for support `Span`
- Split derive in crates

## [0.4.0] (2019-12-13)

### Added 

- `err` attribute at `with-actix-web` feature for specify body of error response
- compile time evaluator for all expressions and helpers

### Fixed
- Dev-dependencies remove `bytes`
 
## [0.3.5] (2019-10-15)

### Fixed

- Fix some issues in parser

## [0.3.4] (2019-10-05)

### Added

- Partial cyclic dependency detection
- match arm guard in expression

## [0.3.3] (2019-09-26)

### Added

- cargo clippy and fix his issues
- Add prettyprint@0.7

### Fixed

- Remove reference in fmt method
- Propagate errors at write_str
- Minor issues

### Updated

- `syn`, `quote`, `proc-macro2` version to `1.0`
- `env_logger` version to `0.7`

## [0.3.2] (2019-08-08)

### Fixed

- Remove prettyprint library for rust-onig@0.4.2 dependency compile problems

### Changed

- Ignore errors at write_str

## [0.3.1] (2019-08-05)

### Fixed 

- `Mime types` update mime_guess to *2.0* and append `; charset=utf-8`

### Changed 

- Performance of render trait for html escape

## [0.3.0] (2019-07-07)

### Added

- Render trait for typed escaped

## [0.2.0] (2019-04-01)

### Fixed

- Some issues

## [0.1.2] (2019-03-19)

### Added 

- Conditional expressions in compile-time to evaluator.

### Fixed 

- Parenthesis expressions.

- Variable name resolution for partials.

## [0.1.1] (2019-03-13)

### Added

- More debugging options in configure file. New option are `grid`, `header`, `paging`, and `short`.

## [0.1.0] (2019-03-11)

### First release
