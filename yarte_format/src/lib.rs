#![cfg(nightly)]
#![feature(proc_macro_hygiene)]

/// Adapted and improve from [`fomat`](https://github.com/krdln/fomat-macros)
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
};

pub use yarte_derive::{yformat, yformat_html};
pub use yarte_helpers::{helpers::Render, recompile, Error, Result};

/// Closure wrapper
///
/// Wrap closure in mutable reference for dispatch it
pub struct DisplayFn<F: FnOnce(&mut Formatter) -> fmt::Result>(std::cell::Cell<Option<F>>);

impl<F: FnOnce(&mut Formatter) -> fmt::Result> DisplayFn<F> {
    pub fn new(f: F) -> Self {
        Self(Cell::new(Some(f)))
    }
}

// Remove double replace in favor of single by cell::Cell::take
impl<F: FnOnce(&mut Formatter) -> fmt::Result> Display for DisplayFn<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.take().ok_or(fmt::Error).and_then(|cl| cl(f))
    }
}
