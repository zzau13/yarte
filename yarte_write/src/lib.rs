#![cfg(feature = "nightly")]
#![feature(proc_macro_hygiene)]

/// Adapted from [`fomat`](https://github.com/krdln/fomat-macros)
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
};

pub use yarte_derive::ywrite;
pub use yarte_helpers::{helpers::Render, recompile, Error, Result};

#[doc(hidden)]
pub struct DisplayFn<F: FnOnce(&mut Formatter) -> fmt::Result>(std::cell::Cell<Option<F>>);

impl<F: FnOnce(&mut Formatter) -> fmt::Result> DisplayFn<F> {
    pub fn new(f: F) -> Self {
        Self(Cell::new(Some(f)))
    }
}

impl<F: FnOnce(&mut Formatter) -> fmt::Result> Display for DisplayFn<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.take().ok_or(fmt::Error).and_then(|cl| cl(f))
    }
}
