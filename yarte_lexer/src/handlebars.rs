use crate::{Cursor, Ki, KiError, PResult};

/// Eat comment
pub fn comment<'a, K: Ki<'a>>(_: Cursor<'a>) -> PResult<&'a str, K::Error> {
    // TODO: shorter ascii token builders
    // TODO: why not run here ... is a literal and run in trait const
    // const A: Ascii = ascii!(b'-');

    Err(next!(K::Error))
}
