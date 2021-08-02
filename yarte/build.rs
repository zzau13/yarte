use yarte_helpers::definitely_not_nightly;
use yarte_helpers::recompile::when_changed;

fn main() {
    when_changed();
    if !definitely_not_nightly() {
        println!("cargo:rustc-cfg=nightly");
    }
}
