#![cfg(feature = "nightly")]
#![feature(proc_macro_hygiene)]

use yarte_write::ywrite;

#[test]
fn test() {
    let world = "World";
    let res = ywrite!("Hello {{ world }}!").unwrap();

    assert_eq!(res, "Hello World!")
}
