use std::collections::BTreeMap;

use quote::quote;
use syn::parse2;

use yarte_config::Config;
use yarte_hir::{generate, visit_derive};
use yarte_parser::{parse, source_map::get_cursor};

#[test]
fn test() {
    let src = r#""#;
    let config = &Config::new("");
    let path = config.get_dir().join("Test.hbs");
    let der = parse2(quote! {
        #[derive(Template)]
        #[template(src = #src, mode = "wasm")]
        #[msg(pub enum Msg {
            Foo,
        })]
        pub struct Test {
            black_box: <Self as Template>::BlackBox,
        }
    })
    .unwrap();
    let s = visit_derive(&der, config);
    assert_eq!(s.path, path);
    assert_eq!(s.src, src);
    let sources = parse(get_cursor(&path, src)).unwrap();
    let mut ctx = BTreeMap::new();
    ctx.insert(&path, sources);

    let _hir = generate(config, &s, &ctx).unwrap_or_else(|e| panic!());
}
