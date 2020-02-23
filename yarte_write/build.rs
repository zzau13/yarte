use yarte_helpers::helpers::definitely_not_nightly;

fn main() {
    if !definitely_not_nightly() {
        println!("cargo:rustc-cfg=nightly");
    }
}
