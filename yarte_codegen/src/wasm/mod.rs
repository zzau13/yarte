#[allow(warnings)]

#[cfg(not(target_arch = "wasm32"))]
#[path = "./server/mod.rs"]
mod codegen;

#[cfg(target_arch = "wasm32")]
#[path = "./client/mod.rs"]
mod codegen;

pub use self::codegen::WASMCodeGen;
