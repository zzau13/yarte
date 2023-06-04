use crate::error::Error;
use crate::token_types::{Delimiter, Ident, Literal, Punct};
use yarte_strnom::error::LexError;
use yarte_strnom::source_map::S;

pub type SResult = std::result::Result<State, LexError<Error>>;

pub enum State {
    Continue,
    Stop,
}

pub trait Sink<'a>: 'a {
    fn open_group(&mut self, del: S<Delimiter>) -> SResult;
    fn close_group(&mut self, del: S<Delimiter>) -> SResult;
    fn ident(&mut self, ident: S<Ident<'a>>) -> SResult;
    fn punct(&mut self, punct: S<Punct>) -> SResult;
    fn literal(&mut self, literal: S<Literal<'a>>) -> SResult;
    fn end(&mut self) -> SResult;
}
