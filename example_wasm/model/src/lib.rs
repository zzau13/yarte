#[cfg(target_arch = "wasm32")]
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use serde::Serialize;

#[cfg_attr(target_arch = "wasm32", derive(Default, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
pub struct Fortune {
    pub id: i32,
    pub message: String,
}
