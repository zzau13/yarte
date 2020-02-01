/// Adapted from [`html5ever`](https://github.com/servo/html5ever)
#[macro_use]
mod macros;
pub mod driver;
#[macro_use]
pub mod interface;
pub mod serializer;
pub mod tokenizer;
pub mod tree_builder;
pub mod utils;
