//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).
// TODO: Remove
#![allow(dead_code)]

use std::{cell::RefCell, cmp, fmt, path::PathBuf};

use syn::export::Debug;

use crate::parser::strnom::{skip_ws, Cursor, PResult};

thread_local! {
    static SOURCE_MAP: RefCell<SourceMap> = RefCell::new(SourceMap {
        files: vec![],
    });
}

/// Add file to source map and return lower bound
///
/// Use in the same thread
pub(crate) fn add_file(p: &PathBuf, s: &str) -> u32 {
    SOURCE_MAP.with(|x| x.borrow_mut().add_file(p, s).lo)
}

/// Reinitialize source map instance when run multiple times in the same thread
///
/// Use in the same thread
pub(crate) fn clean() {
    SOURCE_MAP.with(|x| *x.borrow_mut() = SourceMap { files: vec![] });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineColumn {
    pub line: usize,
    pub column: usize,
}

struct FileInfo {
    name: PathBuf,
    span: Span,
    lines: Vec<usize>,
}

impl FileInfo {
    fn offset_line_column(&self, offset: usize) -> LineColumn {
        assert!(self.span_within(Span {
            lo: offset as u32,
            hi: offset as u32,
        }));
        let offset = offset - self.span.lo as usize;
        match self.lines.binary_search(&offset) {
            Ok(found) => LineColumn {
                line: found + 1,
                column: 0,
            },
            Err(idx) => LineColumn {
                line: idx,
                column: offset - self.lines[idx - 1],
            },
        }
    }

    fn span_within(&self, span: Span) -> bool {
        span.lo >= self.span.lo && span.hi <= self.span.hi
    }
}

/// Computes the offsets of each line in the given source string.
fn lines_offsets(s: &str) -> Vec<usize> {
    let mut lines = vec![0];
    let mut prev = 0;
    while let Some(len) = s[prev..].find('\n') {
        prev += len + 1;
        lines.push(prev);
    }
    lines
}

struct SourceMap {
    files: Vec<FileInfo>,
}

impl SourceMap {
    fn next_start_pos(&self) -> u32 {
        // Add 1 so there's always space between files.
        //
        self.files.last().map(|f| f.span.hi + 1).unwrap_or(0)
    }

    fn add_file(&mut self, name: &PathBuf, src: &str) -> Span {
        let lines = lines_offsets(src);
        let lo = self.next_start_pos();
        let span = Span {
            lo,
            hi: lo + (src.len() as u32),
        };

        self.files.push(FileInfo {
            name: name.to_owned(),
            span,
            lines,
        });

        span
    }

    fn fileinfo(&self, span: Span) -> &FileInfo {
        for file in &self.files {
            if file.span_within(span) {
                return file;
            }
        }
        panic!("Invalid span with no related FileInfo!");
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Span {
    pub lo: u32,
    pub hi: u32,
}

// Don't allow `Span` to transfer between thread
//impl !Send for Span {}
//impl !Sync for Span {}

impl Span {
    /// Assume a <= b
    pub fn from_cursor(a: Cursor, b: Cursor) -> Span {
        debug_assert!(a.off <= b.off);
        Span {
            lo: a.off,
            hi: b.off,
        }
    }

    pub fn from_len(i: Cursor, len: usize) -> Span {
        Span {
            lo: i.off,
            hi: i.off + (len as u32),
        }
    }

    pub fn file_path(self) -> PathBuf {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            fi.name.clone()
        })
    }

    pub fn start(self) -> LineColumn {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            fi.offset_line_column(self.lo as usize)
        })
    }

    pub fn end(self) -> LineColumn {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            fi.offset_line_column(self.hi as usize)
        })
    }

    pub fn join(self, other: Span) -> Option<Span> {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            // If `other` is not within the same FileInfo as us, return None.
            if !cm.fileinfo(self).span_within(other) {
                return None;
            }
            Some(Span {
                lo: cmp::min(self.lo, other.lo),
                hi: cmp::max(self.hi, other.hi),
            })
        })
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bytes({}..{})", self.lo, self.hi)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct S<T: Debug + PartialEq + Clone>(pub(super) T, pub(super) Span);

impl<T: Debug + PartialEq + Clone> S<T> {
    pub fn t(&self) -> &T {
        &self.0
    }
    pub fn span(&self) -> &Span {
        &self.1
    }
}

pub(crate) fn spanned<'a, T: Debug + PartialEq + Clone>(
    input: Cursor<'a>,
    f: fn(Cursor<'a>) -> PResult<'a, T>,
) -> PResult<'a, S<T>> {
    let input = skip_ws(input);
    let lo = input.off;
    let (a, b) = f(input)?;
    let hi = a.off;
    let span = Span { lo, hi };
    Ok((a, S(b, span)))
}
