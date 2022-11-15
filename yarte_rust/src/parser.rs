use serde::Deserialize;
use yarte_strnom::Cursor;

use crate::token_types::*;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum Token<'a> {
    OpenGroup(Delimiter),
    CloseGroup(Delimiter),
    #[serde(borrow)]
    Ident(Ident<'a>),
    Punct(Punct),
    #[serde(borrow)]
    Literal(Literal<'a>),
}
