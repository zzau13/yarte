#[cfg(feature = "wasm-app")]
pub mod client;
#[cfg(all(feature = "wasm-server", feature = "bytes-buf"))]
pub mod server {
    use proc_macro2::TokenStream;

    use yarte_dom::dom_fmt::to_wasmfmt;
    use yarte_hir::{Struct, HIR};

    use crate::{CodeGen, EachCodeGen, HTMLMinBytesCodeGen, IfElseCodeGen};

    pub struct WASMCodeGen<'a> {
        s: &'a Struct<'a>,
        buf: &'a syn::Expr,
        parent: &'static str,
    }

    impl<'a> EachCodeGen for WASMCodeGen<'a> {}
    impl<'a> IfElseCodeGen for WASMCodeGen<'a> {}

    impl<'a> WASMCodeGen<'a> {
        pub fn new<'n>(
            s: &'n Struct<'n>,
            buf: &'n syn::Expr,
            parent: &'static str,
        ) -> WASMCodeGen<'n> {
            WASMCodeGen { s, buf, parent }
        }
    }

    impl<'a> CodeGen for WASMCodeGen<'a> {
        fn gen(&mut self, ir: Vec<HIR>) -> TokenStream {
            let ir = to_wasmfmt(ir, self.s).expect("html");
            HTMLMinBytesCodeGen::new(self.buf, self.parent).gen(ir)
        }
    }
}
