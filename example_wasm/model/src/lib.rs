#[cfg(target_arch = "wasm32")]
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use yarte::Serialize;

#[cfg_attr(target_arch = "wasm32", derive(Default, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
pub struct Fortune {
    pub id: u32,
    pub message: String,
    pub foo: Vec<usize>,
    pub bar: Vec<Item>,
}

#[cfg_attr(target_arch = "wasm32", derive(Default, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize))]
pub struct Item {
    pub fol: usize,
}
