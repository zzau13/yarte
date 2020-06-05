#[cfg(feature = "json")]
pub mod json {
    use std::fmt::{self, Display};

    use serde::Serialize;
    use serde_json::{to_writer, to_writer_pretty};

    use crate::helpers::io_fmt::IoFmt;

    pub struct Json<'a, T: Serialize>(pub &'a T);
    pub struct JsonPretty<'a, T: Serialize>(pub &'a T);

    impl<'a, S: Serialize> Display for Json<'a, S> {
        #[inline(always)]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            to_writer(IoFmt::new(f), self.0).map_err(|_| fmt::Error)
        }
    }

    impl<'a, D: Serialize> Display for JsonPretty<'a, D> {
        #[inline(always)]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            to_writer_pretty(IoFmt::new(f), self.0).map_err(|_| fmt::Error)
        }
    }
}
#[cfg(feature = "json")]
pub use self::json::*;
