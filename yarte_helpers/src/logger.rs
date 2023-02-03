pub fn log(s: &str) {
    match syn::parse_str(s) {
        Ok(file) => println!("{}", prettyplease::unparse(&file)),
        Err(_) => println!("{s}"),
    }
}
