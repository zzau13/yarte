#[cfg(feature = "json")]
pub mod json {
    use std::fmt::{self, Display};

    use serde::Serialize;
    use serde_json::{to_writer, to_writer_pretty};

    use crate::helpers::io_fmt::IoFmt;

    pub struct Json<'a, T: Serialize>(pub &'a T);

    pub trait AsJson {
        fn __as_json(&self) -> Json<'_, Self>
        where
            Self: Serialize + Sized;
    }

    impl<'a, S: Serialize> AsJson for S {
        fn __as_json(&self) -> Json<'_, Self>
        where
            Self: Serialize + Sized,
        {
            Json(self)
        }
    }

    pub struct JsonPretty<'a, T: Serialize>(pub &'a T);

    pub trait AsJsonPretty {
        fn __as_json_pretty(&self) -> JsonPretty<'_, Self>
        where
            Self: Serialize + Sized;
    }

    impl<'a, S: Serialize> AsJsonPretty for S {
        fn __as_json_pretty(&self) -> JsonPretty<'_, Self>
        where
            Self: Serialize + Sized,
        {
            JsonPretty(self)
        }
    }

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
