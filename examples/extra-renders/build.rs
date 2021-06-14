fn main() {
    yarte::recompile::when_changed();
    if !yarte_helpers::definitely_not_nightly() {
        println!("cargo:rustc-cfg=nightly");
    }
}
