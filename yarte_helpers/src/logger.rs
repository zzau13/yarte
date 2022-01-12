pub fn log(s: &str) {
    println!("{}", prettyplease::unparse(&syn::parse_str(s).unwrap()));
}
