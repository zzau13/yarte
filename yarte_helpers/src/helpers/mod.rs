use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[cfg(feature = "big_num")]
pub mod big_num;
mod error;
mod markup;
mod read;

pub use error::{emitter, ErrorMessage};
pub use markup::Render;
pub use read::{read, Sources};

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
