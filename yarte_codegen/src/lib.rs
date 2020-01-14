use mime_guess::from_ext;
use proc_macro2::TokenStream;
use quote::quote;

use yarte_hir::{Each, IfElse, Mode, Struct, HIR};

mod html;
mod text;
pub mod wasm;

pub use self::{
    html::{HTMLCodeGen, HTMLMinCodeGen},
    text::TextCodeGen,
};

pub trait CodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream;
}

pub struct FmtCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
}

impl<'a, T: CodeGen> FmtCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct) -> FmtCodeGen<'n, T> {
        FmtCodeGen { codegen, s }
    }

    fn get_mime(&self) -> String {
        let ext = match self.s.mode {
            Mode::Text => match self.s.path.extension() {
                Some(s) => s.to_str().unwrap(),
                None => "txt",
            },
            _ => "html",
        };

        from_ext(ext).first_or_text_plain().to_string()
    }

    fn template(&self, size_hint: usize, tokens: &mut TokenStream) {
        let mime = self.get_mime() + "; charset=utf-8";
        let body = quote!(
            fn mime() -> &'static str { #mime }
            fn size_hint() -> usize {
                #size_hint
            }
        );
        tokens.extend(self.s.implement_head(quote!(::yarte::Template), &body));
    }

    fn display(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) -> usize {
        let nodes = self.codegen.gen(nodes);
        // heuristic based on https://github.com/lfairy/maud
        let size_hint = nodes.to_string().len();
        let func = quote!(
            fn fmt(&self, _fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #nodes
                Ok(())
            }
        );

        tokens.extend(self.s.implement_head(quote!(::std::fmt::Display), &func));

        size_hint
    }

    fn responder(&self, tokens: &mut TokenStream) {
        let err_msg = &self.s.err_msg;

        let body = quote!(
            type Error = ::yarte::aw::Error;
            type Future = ::yarte::aw::Ready<::std::result::Result<::yarte::aw::HttpResponse, Self::Error>>;

            #[inline]
            fn respond_to(self, _req: &::yarte::aw::HttpRequest) -> Self::Future {
                match self.call() {
                    Ok(body) => {
                        ::yarte::aw::ok(::yarte::aw::HttpResponse::Ok().content_type(Self::mime()).body(body))
                    }
                    Err(_) => {
                        ::yarte::aw::err(::yarte::aw::ErrorInternalServerError(#err_msg))
                    }
                }
            }
        );

        tokens.extend(self.s.implement_head(quote!(::yarte::aw::Responder), &body));
    }
}

impl<'a, T: CodeGen> CodeGen for FmtCodeGen<'a, T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();

        let size_hint = self.display(v, &mut tokens);
        self.template(size_hint, &mut tokens);

        if cfg!(feature = "actix-web") {
            self.responder(&mut tokens);
        }

        tokens
    }
}

pub trait EachCodeGen: CodeGen {
    fn gen_each(&mut self, Each { args, body, expr }: Each) -> TokenStream {
        let body = self.gen(body);
        quote!(for #expr in #args { #body })
    }
}

pub trait IfElseCodeGen: CodeGen {
    fn gen_if_else(&mut self, IfElse { ifs, if_else, els }: IfElse) -> TokenStream {
        let mut tokens = TokenStream::new();

        let (args, body) = ifs;
        let body = self.gen(body);
        tokens.extend(quote!(if #args { #body }));

        for (args, body) in if_else {
            let body = self.gen(body);
            tokens.extend(quote!(else if #args { #body }));
        }

        if let Some(body) = els {
            let body = self.gen(body);
            tokens.extend(quote!(else { #body }));
        }

        tokens
    }
}
