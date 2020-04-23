use std::{fmt, io, str::from_utf8_unchecked};

/// # Not Use is internal library
/// Io write implementation for Formatter
///
/// # Unsafe
/// Only use for write utf-8 strings
pub struct IoFmt<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> IoFmt<'a, 'b> {
    pub fn new<'n, 'm>(f: &'n mut fmt::Formatter<'m>) -> IoFmt<'n, 'm> {
        IoFmt(f)
    }
}

impl<'a, 'b> io::Write for IoFmt<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .write_str(unsafe { from_utf8_unchecked(buf) })
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helpers::display_fn::DisplayFn;
    use serde::Serialize;
    #[derive(Serialize)]
    struct Foo {
        n: usize,
    }
    #[test]
    fn test() {
        let j = Foo { n: 1 };
        assert_eq!(
            DisplayFn::new(|f| serde_json::to_writer(IoFmt::new(f), &j).map_err(|_| fmt::Error))
                .to_string(),
            serde_json::to_string(&j).unwrap()
        );

        assert_eq!(
            DisplayFn::new(
                |f| serde_json::to_writer_pretty(IoFmt::new(f), &j).map_err(|_| fmt::Error)
            )
            .to_string(),
            serde_json::to_string_pretty(&j).unwrap()
        );
    }
}
