#![allow(dead_code)]

use proc_macro2::TokenStream;

use yarte_config::Config;
use yarte_hir::{Struct, HIR};

use super::CodeGen;

pub struct WASMCodeGen<'a> {
    s: &'a Struct<'a>,
    config: &'a Config<'a>,
}

impl<'a> WASMCodeGen<'a> {
    pub fn new<'n>(config: &'n Config<'n>, s: &'n Struct<'n>) -> WASMCodeGen<'n> {
        WASMCodeGen { config, s }
    }
}

impl<'a> CodeGen for WASMCodeGen<'a> {
    fn gen(&self, _: Vec<HIR>) -> TokenStream {
        todo!()
    }
}
