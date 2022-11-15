use crate::error::Error;
use crate::token_types::{Delimiter, Ident, Literal, Punct};
use yarte_strnom::error::LexError;

pub type SResult = std::result::Result<State, LexError<Error>>;

pub enum State {
    Continue,
    Stop,
}

pub trait Sink<'a>: 'a {
    fn open_group(&mut self, del: Delimiter) -> SResult;
    fn close_group(&mut self, del: Delimiter) -> SResult;
    fn ident(&mut self, ident: Ident<'a>) -> SResult;
    fn punct(&mut self, punct: Punct) -> SResult;
    fn literal(&mut self, literal: Literal<'a>) -> SResult;
    fn end(&mut self) -> SResult;
}
