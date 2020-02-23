#![cfg(feature = "nightly")]
#![feature(proc_macro_hygiene)]

use yarte_write::yformat;

#[test]
fn test() {
    let world = "World";
    let res = yformat!("Hello {{ world }}!");

    assert_eq!(res, "Hello World!")
}
