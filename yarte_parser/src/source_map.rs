//! Adapted from [`proc-macro2`](https://github.com/alexcrichton/proc-macro2).
use std::{
    cell::RefCell,
    fmt::{self, Debug},
    path::Path,
    rc::Rc,
};

use crate::strnom::{skip_ws, Cursor, PResult};

thread_local! {
    static SOURCE_MAP: RefCell<SourceMap> = RefCell::new(Default::default());
}

/// Add file to source map and return lower bound
///
/// Use in the same thread
pub fn get_cursor(p: Rc<Path>, rest: &str) -> Cursor {
    SOURCE_MAP.with(|x| Cursor {
        rest,
        off: x.borrow_mut().add_file(p, rest).lo,
    })
}

/// Reinitialize source map instance when run multiple times in the same thread
///
/// Use in the same thread
pub fn clean() {
    SOURCE_MAP.with(|x| *x.borrow_mut() = Default::default());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

struct FileInfo {
    name: Rc<Path>,
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

    fn get_ranges(&self, span: Span) -> ((usize, usize), (usize, usize)) {
        assert!(self.span_within(span));
        let lo = (span.lo - self.span.lo) as usize;
        let hi = (span.hi - self.span.lo) as usize;
        let lo_line = match self.lines.binary_search(&lo) {
            Ok(_) => lo,
            Err(idx) => self.lines[idx - 1],
        };
        let hi_line = match self.lines.binary_search(&hi) {
            Ok(_) => hi,
            Err(idx) => self
                .lines
                .get(idx)
                .copied()
                .unwrap_or(self.span.hi as usize),
        };
        ((lo_line, hi_line), (lo - lo_line, hi - lo_line))
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

#[derive(Default)]
struct SourceMap {
    files: Vec<FileInfo>,
}

impl SourceMap {
    fn next_start_pos(&self) -> u32 {
        // Add 1 so there's always space between files.
        self.files.last().map(|f| f.span.hi + 1).unwrap_or(0)
    }

    fn add_file(&mut self, name: Rc<Path>, src: &str) -> Span {
        let lines = lines_offsets(src);
        let lo = self.next_start_pos();
        let span = Span {
            lo,
            hi: lo + (src.len() as u32),
        };

        self.files.push(FileInfo {
            name: Rc::clone(&name),
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

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
}

// Don't allow `Span` to transfer between thread
// impl !Send for Span {}
// impl !Sync for Span {}

impl Span {
    /// Assume a <= b
    #[inline]
    pub fn from_cursor(a: Cursor, b: Cursor) -> Self {
        debug_assert!(a.off <= b.off);
        Self {
            lo: a.off,
            hi: b.off,
        }
    }

    pub fn from_len(i: Cursor, len: usize) -> Self {
        Self {
            lo: i.off,
            hi: i.off + (len as u32),
        }
    }

    pub fn from_range(i: Cursor, (lo, hi): (usize, usize)) -> Self {
        Self {
            lo: i.off + (lo as u32),
            hi: i.off + (hi as u32),
        }
    }

    pub fn join_proc(self, proc: proc_macro2::Span) -> Self {
        let start = self.start();
        let p_start = proc.start();
        let p_end = proc.end();
        let lo = if p_start.line == 1 {
            self.lo + p_start.column as u32
        } else {
            SOURCE_MAP.with(|cm| {
                let cm = cm.borrow();
                let fi = cm.fileinfo(self);
                fi.lines[start.line + p_start.line - 2] as u32 + p_start.column as u32
            })
        };
        let hi = if p_end.line == 1 {
            self.lo + p_end.column as u32
        } else {
            SOURCE_MAP.with(|cm| {
                let cm = cm.borrow();
                let fi = cm.fileinfo(self);
                fi.lines[start.line + p_end.line - 2] as u32 + p_end.column as u32
            })
        };

        Self { lo, hi }
    }

    /// Returns line bounds and range in bounds
    pub fn range_in_file(self) -> ((usize, usize), (usize, usize)) {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            fi.get_ranges(self)
        })
    }

    pub fn file_path(self) -> Rc<Path> {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            Rc::clone(&fi.name)
        })
    }

    pub fn start(self) -> LineColumn {
        SOURCE_MAP.with(|cm| {
            let cm = cm.borrow();
            let fi = cm.fileinfo(self);
            fi.offset_line_column(self.lo as usize)
        })
    }
}

impl<'a> From<Cursor<'a>> for Span {
    fn from(c: Cursor) -> Self {
        Self::from_cursor(c, c)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bytes({}..{})", self.lo, self.hi)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct S<T: Debug + PartialEq + Clone>(pub(super) T, pub(super) Span);

impl<T: Debug + PartialEq + Clone> S<T> {
    pub fn t(&self) -> &T {
        &self.0
    }
    pub fn span(&self) -> Span {
        self.1
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
