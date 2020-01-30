use std::fmt::{self, Write};

use yarte_helpers::Result;

/// Template trait, will implement by derive like `Display` or `actix_web::Responder` (with feature)
///
pub trait Template: fmt::Display {
    /// which will write this template
    fn call(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::size_hint());
        write!(buf, "{}", self).map(|_| buf)
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
    #[cfg(feature = "mime")]
    fn mime() -> &'static str
    where
        Self: Sized;

    /// Approximation of output size used in method `call`.
    /// Yarte implements an heuristic algorithm of allocation.
    fn size_hint() -> usize;
}
