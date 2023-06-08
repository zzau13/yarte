use yarte_strnom::source_map::S;

use crate::parser::ast::expr::Expr;
use crate::sink::SResult;
use crate::tokens::Punct;

pub struct ExprPunct<'a> {
    pub expr: &'a mut Option<Expr>,
}

pub fn expr_punct(punct: S<Punct>, _state: ExprPunct) -> SResult {
    match punct.0 {
        Punct::Exclamation => {}
        Punct::Hash => {}
        Punct::Dollar => {}
        Punct::Percent => {}
        Punct::And => {}
        Punct::Apostrophe => {}
        Punct::Asterisk => {}
        Punct::Plus => {}
        Punct::Comma => {}
        Punct::Hyphen => {}
        Punct::Dot => {}
        Punct::Slash => {}
        Punct::Colon => {}
        Punct::SemiColon => {}
        Punct::GreaterThan => {}
        Punct::Equal => {}
        Punct::LessThan => {}
        Punct::Question => {}
        Punct::At => {}
        Punct::Circumflex => {}
        Punct::Underscore => {}
        Punct::Backtick => {}
        Punct::Bar => {}
        Punct::Tilde => {}
    };
    todo!("expression")
}
