use serde::Deserialize;

use crate::token_types::*;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    OpenGroup(Delimiter),
    CloseGroup(Delimiter),
    #[serde(borrow)]
    Ident(Ident<'a>),
    Punct(Punct),
    #[serde(borrow)]
    Literal(Literal<'a>),
}
