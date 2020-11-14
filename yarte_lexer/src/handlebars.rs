use crate::{Cursor, Ki, KiError, PResult};

/// Eat comment
pub fn comment<'a, K: Ki<'a>>(_: Cursor<'a>) -> PResult<&'a str, K::Error> {
    // TODO: shorter ascii token builders
    // TODO: Error because is in the same crate it's a rustc compilation fail
    // const A: Ascii = ascii!(b'-');

    Err(next!(K::Error))
}
