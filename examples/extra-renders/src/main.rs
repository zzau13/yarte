#![cfg_attr(nightly, feature(proc_macro_hygiene, stmt_expr_attributes))]
use std::io::{stdout, Write};

use uuid::Uuid;
use yarte::*;

/// without comma or error
/// `message: stable/nightly mismatch`
fn write_str(buffer: String) {
    stdout().lock().write_all(buffer.as_bytes()).unwrap();
}

#[cfg(nightly)]
fn nightly(uuid: &Uuid) {
    // without comma or error
    // `message: stable/nightly mismatch`
    #[rustfmt::skip]
    write_str(#[html] "{{> hello }}");
}

fn main() {
    let uuid = Uuid::nil();

    #[cfg(not(nightly))]
    write_str(auto!(ywrite_html!(String, "{{> hello }}")));

    #[cfg(nightly)]
    nightly(&uuid);
}
