#[cfg(feature = "json")]
pub mod json {
    use std::fmt::{self, Display};

    use serde::Serialize;
    use serde_json::{to_writer, to_writer_pretty};

    use crate::helpers::io_fmt::IoFmt;

    pub struct Json<'a, T>(pub &'a T);

    impl<'a, T> Clone for Json<'a, T> {
        fn clone(&self) -> Self {
            Json(self.0)
        }
    }

    impl<'a, T> Copy for Json<'a, T> {}

    pub trait AsJson {
        fn __as_json(&self) -> Json<'_, Self>
        where
            Self: Sized;
    }

    impl<S> AsJson for S {
        fn __as_json(&self) -> Json<'_, Self>
        where
            Self: Sized,
        {
            Json(self)
        }
    }

    pub struct JsonPretty<'a, T>(pub &'a T);

    impl<'a, T> Clone for JsonPretty<'a, T> {
        fn clone(&self) -> Self {
            JsonPretty(self.0)
        }
    }

    impl<'a, T> Copy for JsonPretty<'a, T> {}

    pub trait AsJsonPretty {
        fn __as_json_pretty(&self) -> JsonPretty<'_, Self>
        where
            Self: Sized;
    }

    impl<S> AsJsonPretty for S {
        fn __as_json_pretty(&self) -> JsonPretty<'_, Self>
        where
            Self: Sized,
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

    impl<'a, S: Serialize> Display for JsonPretty<'a, S> {
        #[inline(always)]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            to_writer_pretty(IoFmt::new(f), self.0).map_err(|_| fmt::Error)
        }
    }
}
#[cfg(feature = "json")]
pub use self::json::*;
