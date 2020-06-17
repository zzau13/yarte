use std::fmt::{Display, Formatter, Result, Write};
use std::mem::MaybeUninit;
use std::{io, slice};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use itoa;
use v_htmlescape::v_escape;
use yarte::{
    Template, TemplateBytes, TemplateBytesText, TemplateFixed, TemplateFixedText, TemplateText,
};

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    // 3 bytes
    c.bench_function("3 bytes byte-by-byte", write_3_bytes_bb);
    c.bench_function("3 bytes Memcpy", write_3_bytes_memcpy);
    c.bench_function("3 bytes", write_3_bytes);

    // Teams
    c.bench_function("Raw Teams byte-by-byte", raw_teams);
    c.bench_function("Raw Teams Memcpy", raws_teams_memcpy);
    c.bench_function("Fixed Text Teams", fixed_text_teams);
    c.bench_function("Bytes Text Teams", bytes_text_teams);
    c.bench_function("Raw Escaped Teams byte-by-byte", raws_teams_escaped);
    c.bench_function("Raw Escaped Teams Memcpy", raw_teams_escaped_memcpy);
    c.bench_function("Fixed Teams", fixed_teams);
    c.bench_function("Bytes Teams", bytes_teams);
    c.bench_function("Teams", teams);
    c.bench_function("Teams io writer implements io::Write", io_termcolor_teams);
    c.bench_function("Teams Unescaped", teams_text);
    c.bench_function("Formatter Teams", fmt_teams);

    // Big table
    const SIZE: usize = 100;
    c.bench_function("Raw Big table byte-by-byte", |b| raw_big_table(b, SIZE));
    c.bench_function("Raw Big table Memcpy", |b| raw_big_table_memcpy(b, SIZE));
    c.bench_function("Fixed Big Table", |b| fixed_big_table(b, SIZE));
    c.bench_function("Bytes Big Table", |b| bytes_big_table(b, SIZE));
    c.bench_function("Fixed Text Big Table", |b| fixed_text_big_table(b, SIZE));
    c.bench_function("Bytes Text Big Table", |b| bytes_text_big_table(b, SIZE));
    c.bench_function("Formatter Big table", |b| fmt_big_table(b, SIZE));
    c.bench_function("Big table", |b| big_table(b, SIZE));
    c.bench_function("Big table io writer", |b| io_big_table(b, SIZE));
    c.bench_function("Big table Unescaped", |b| big_table_text(b, SIZE));
}

// Helpers
fn build_big_table(size: usize) -> Vec<Vec<usize>> {
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }

    table
}

fn build_teams() -> Vec<Team> {
    vec![
        Team {
            name: "Jiangsu".into(),
            score: 43,
        },
        Team {
            name: "Beijing".into(),
            score: 27,
        },
        Team {
            name: "Guangzhou".into(),
            score: 22,
        },
        Team {
            name: "Shandong".into(),
            score: 12,
        },
    ]
}

struct Team {
    name: String,
    score: u8,
}

// Yarte
#[derive(Template)]
#[template(path = "teams")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

fn teams(b: &mut criterion::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| teams.call().unwrap());
}

#[derive(TemplateText)]
#[template(path = "teams")]
struct TeamsDisplay {
    year: u16,
    teams: Vec<Team>,
}

fn teams_text(b: &mut criterion::Bencher) {
    let teams = TeamsDisplay {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| teams.call().unwrap());
}

#[derive(Template)]
#[template(path = "big-table")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

fn big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTable {
        table: build_big_table(size),
    };
    b.iter(|| t.call().unwrap());
}

#[derive(TemplateText)]
#[template(path = "big-table")]
struct BigTableDisplay {
    table: Vec<Vec<usize>>,
}

fn big_table_text(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableDisplay {
        table: build_big_table(size),
    };
    b.iter(|| t.call().unwrap());
}

// Bytes
#[derive(TemplateBytes)]
#[template(path = "teams")]
struct TeamsB {
    year: u16,
    teams: Vec<Team>,
}

fn bytes_teams(b: &mut criterion::Bencher) {
    let teams = TeamsB {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| teams.call(2048).unwrap());
}

#[derive(TemplateBytesText)]
#[template(path = "teams")]
struct TeamsBT {
    year: u16,
    teams: Vec<Team>,
}

fn bytes_text_teams(b: &mut criterion::Bencher) {
    let teams = TeamsBT {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| teams.call(2048).unwrap());
}

#[derive(TemplateBytes)]
#[template(path = "big-table")]
struct BigTableB {
    table: Vec<Vec<usize>>,
}

fn bytes_big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableB {
        table: build_big_table(size),
    };
    b.iter(|| t.call(109915).unwrap());
}

#[derive(TemplateBytesText)]
#[template(path = "big-table")]
struct BigTableBT {
    table: Vec<Vec<usize>>,
}

fn bytes_text_big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableBT {
        table: build_big_table(size),
    };
    b.iter(|| t.call(109915).unwrap());
}

// Fixed
#[derive(TemplateFixed)]
#[template(path = "teams")]
struct TeamsF {
    year: u16,
    teams: Vec<Team>,
}

fn fixed_teams(b: &mut criterion::Bencher) {
    let teams = TeamsF {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| {
        black_box(unsafe { teams.call(&mut [MaybeUninit::uninit(); 2048]) }.unwrap());
    });
}

#[derive(TemplateFixedText)]
#[template(path = "teams")]
struct TeamsFT {
    year: u16,
    teams: Vec<Team>,
}

fn fixed_text_teams(b: &mut criterion::Bencher) {
    let teams = TeamsFT {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| {
        black_box(unsafe { teams.call(&mut [MaybeUninit::uninit(); 2048]) }.unwrap());
    });
}

#[derive(TemplateFixed)]
#[template(path = "big-table")]
struct BigTableF {
    table: Vec<Vec<usize>>,
}

fn fixed_big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableF {
        table: build_big_table(size),
    };
    b.iter(|| {
        black_box(unsafe { t.call(&mut [MaybeUninit::uninit(); 109915]) }.unwrap());
    });
}

#[derive(TemplateFixedText)]
#[template(path = "big-table")]
struct BigTableFT {
    table: Vec<Vec<usize>>,
}

fn fixed_text_big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableFT {
        table: build_big_table(size),
    };
    b.iter(|| {
        black_box(unsafe { t.call(&mut [MaybeUninit::uninit(); 109915]) }.unwrap());
    });
}

// Fmt
fn fmt_teams(b: &mut criterion::Bencher) {
    let teams = TeamsFmt {
        year: 2015,
        teams: build_teams(),
    };

    let mut buf = String::with_capacity(teams.to_string().len());
    b.iter(|| {
        buf.clear();
        write!(buf, "{}", teams).unwrap();
    });
}

struct TeamsFmt {
    year: u16,
    teams: Vec<Team>,
}

impl Display for TeamsFmt {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("<html><head><title>")?;
        self.year.fmt(f)?;
        f.write_str("</title></head><body><h1>CSL ")?;
        self.year.fmt(f)?;
        f.write_str("</h1><ul>")?;
        for (i, v) in self.teams.iter().enumerate() {
            f.write_str("<li class=\"")?;
            if i == 0 {
                f.write_str("champion")?;
            }
            f.write_str("\"><b>")?;
            v.name.fmt(f)?;
            f.write_str("</b>: ")?;
            v.score.fmt(f)?;
            f.write_str("</li>")?;
        }
        f.write_str("</ul></body></html>")
    }
}

fn fmt_big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableFmt {
        table: build_big_table(size),
    };
    let mut buf = String::with_capacity(t.to_string().len());
    b.iter(|| {
        buf.clear();
        write!(buf, "{}", t).unwrap();
    });
}

struct BigTableFmt {
    table: Vec<Vec<usize>>,
}

impl Display for BigTableFmt {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("<table>")?;
        for i in &self.table {
            f.write_str("<tr>")?;
            for j in i {
                f.write_str("<td>")?;
                j.fmt(f)?;
                f.write_str("</td>")?;
            }
            f.write_str("</tr>")?;
        }
        f.write_str("</table>")
    }
}

// Io write
#[inline]
fn _io_big_table<W: std::io::Write>(f: &mut W, table: &Vec<Vec<usize>>) -> std::io::Result<()> {
    f.write_all(b"<table>")?;
    for i in table {
        f.write_all(b"<tr>")?;
        for j in i {
            f.write_all(b"<td>")?;
            write!(f, "{}", j)?;
            f.write_all(b"</td>")?;
        }
        f.write_all(b"</tr>")?;
    }
    f.write_all(b"</table>")
}

fn io_big_table(b: &mut criterion::Bencher, size: usize) {
    let table = build_big_table(size);
    let mut buf = vec![];
    let _ = _io_big_table(&mut buf, &table);
    let len = buf.len();
    b.iter(|| {
        let mut buf = Vec::with_capacity(len);
        _io_big_table(&mut buf, &table).unwrap();
        buf
    });
}

// Version of `termcolor`
struct TeamsWriter<W>(pub W);

impl<W: io::Write> io::Write for TeamsWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: std::io::Write> TeamsWriter<W> {
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        use std::io::Write;
        self.write_all(s.as_bytes())
    }

    #[inline]
    fn io_writer_teams(&mut self, Teams { year, teams }: &Teams) -> std::io::Result<()> {
        use std::io::Write;
        self.write_str("<html><head><title>")?;
        write!(self, "{}", year)?;
        self.write_str("</title></head><body><h1>CSL ")?;
        write!(self, "{}", year)?;
        self.write_str("</h1><ul>")?;
        for (i, v) in teams.iter().enumerate() {
            self.write_str("<li class=\"")?;
            if i == 0 {
                self.write_str("champion")?;
            }
            self.write_str("\"><b>")?;
            write!(self, "{}", v.name)?;
            self.write_str("</b>: ")?;
            write!(self, "{}", v.score)?;
            self.write_str("</li>")?;
        }
        self.write_str("</ul></body></html>")
    }
}

fn io_termcolor_teams(b: &mut criterion::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: build_teams(),
    };
    let mut buf = TeamsWriter(vec![]);
    let _ = buf.io_writer_teams(&teams);
    let len = buf.0.len();
    b.iter(|| {
        let mut buf = TeamsWriter(Vec::with_capacity(len));
        buf.io_writer_teams(&teams).unwrap();
        buf
    });
}

// Raw
fn raw_big_table_memcpy(b: &mut criterion::Bencher, size: usize) {
    unsafe {
        let table = build_big_table(size);
        const LEN: usize = 109915;

        b.iter(|| {
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            macro_rules! buf_ptr {
                () => {
                    &mut buf as *mut _ as *mut u8
                };
            }

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        std::ptr::copy_nonoverlapping(
                            ($b as *const [u8] as *const u8),
                            buf_ptr!().add(curr),
                            $b.len(),
                        );
                        curr += $b.len();
                    }
                };
            }

            write_b!(b"<table>");
            for i in &table {
                write_b!(b"<tr>");
                for j in i {
                    write_b!(b"<td>");
                    curr += itoa::write(&mut buf[curr..], *j).unwrap();
                    write_b!(b"</td>");
                }
                write_b!(b"</tr>");
            }
            write_b!(b"</table>");
            black_box(slice::from_raw_parts(buf_ptr!(), curr));
        });
    }
}

fn raw_big_table(b: &mut criterion::Bencher, size: usize) {
    unsafe {
        let table = build_big_table(size);
        const LEN: usize = 109915;

        b.iter(|| {
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            macro_rules! buf_ptr {
                () => {
                    &mut buf as *mut _ as *mut u8
                };
            }

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        for i in $b {
                            buf_ptr!().add(curr).write(*i);
                            curr += 1;
                        }
                    }
                };
            }

            write_b!(b"<table>");
            for i in &table {
                write_b!(b"<tr>");
                for j in i {
                    write_b!(b"<td>");
                    curr += itoa::write(&mut buf[curr..], *j).unwrap();
                    write_b!(b"</td>");
                }
                write_b!(b"</tr>");
            }
            write_b!(b"</table>");
            black_box(slice::from_raw_parts(buf_ptr!(), curr));
        });
    }
}

fn raw_teams_escaped_memcpy(b: &mut criterion::Bencher) {
    unsafe {
        let teams = Teams {
            year: 2015,
            teams: build_teams(),
        };
        const LEN: usize = 256;

        b.iter(|| {
            let buf = &mut [MaybeUninit::uninit(); LEN];
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        std::ptr::copy_nonoverlapping(
                            ($b as *const _ as *const u8),
                            buf_ptr.add(curr) as *mut u8,
                            $b.len(),
                        );
                        curr += $b.len();
                    }
                };
            }

            write_b!(b"<html><head><title>");
            curr += itoa::write(
                slice::from_raw_parts_mut(buf_ptr.add(curr) as *mut u8, LEN - curr),
                teams.year,
            )
            .expect("buffer overflow");
            write_b!(b"</title></head><body><h1>CSL ");
            curr += itoa::write(
                slice::from_raw_parts_mut(buf_ptr.add(curr) as *mut u8, LEN - curr),
                teams.year,
            )
            .unwrap();
            write_b!(b"</h1><ul>");
            for (i, v) in teams.teams.iter().enumerate() {
                write_b!(b"<li class=\"");
                if i == 0 {
                    write_b!(b"champion");
                }
                write_b!(b"\"><b>");
                curr += v_escape(v.name.as_bytes(), &mut buf[curr..]).expect("buffer overflow");

                write_b!(b"</b>: ");
                curr += itoa::write(
                    slice::from_raw_parts_mut(buf_ptr.add(curr) as *mut u8, LEN - curr),
                    v.score,
                )
                .expect("buffer overflow");
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");

            black_box(slice::from_raw_parts(
                buf_ptr as *const _ as *const u8,
                curr,
            ));
        })
    }
}

fn raw_teams(b: &mut criterion::Bencher) {
    unsafe {
        let teams = Teams {
            year: 2015,
            teams: build_teams(),
        };
        const LEN: usize = 256;

        b.iter(|| {
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        for i in $b {
                            buf_ptr.add(curr).write(*i);
                            curr += 1;
                        }
                    }
                };
            }

            macro_rules! write_u16 {
                ($n:expr) => {
                    curr +=
                        itoa::write(slice::from_raw_parts_mut(buf_ptr.add(curr), LEN - curr), $n)
                            .expect("buffer overflow");
                };
            }

            write_b!(b"<html><head><title>");
            write_u16!(teams.year);
            write_b!(b"</title></head><body><h1>CSL ");
            write_u16!(teams.year);
            write_b!(b"</h1><ul>");
            for (i, v) in teams.teams.iter().enumerate() {
                write_b!(b"<li class=\"");
                if i == 0 {
                    write_b!(b"champion");
                }
                write_b!(b"\"><b>");
                write_b!(v.name.as_bytes());
                write_b!(b"</b>: ");
                write_u16!(v.score);
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");

            black_box(slice::from_raw_parts(buf_ptr, curr));
        })
    }
}

fn raws_teams_memcpy(b: &mut criterion::Bencher) {
    unsafe {
        let teams = Teams {
            year: 2015,
            teams: build_teams(),
        };
        const LEN: usize = 256;

        b.iter(|| {
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        std::ptr::copy_nonoverlapping(
                            ($b as *const [u8] as *const u8),
                            buf_ptr.add(curr),
                            $b.len(),
                        );
                        curr += $b.len();
                    }
                };
            }

            macro_rules! write_u16 {
                ($n:expr) => {
                    curr +=
                        itoa::write(slice::from_raw_parts_mut(buf_ptr.add(curr), LEN - curr), $n)
                            .expect("buffer overflow");
                };
            }

            write_b!(b"<html><head><title>");
            write_u16!(teams.year);
            write_b!(b"</title></head><body><h1>CSL ");
            write_u16!(teams.year);
            write_b!(b"</h1><ul>");
            for (i, v) in teams.teams.iter().enumerate() {
                write_b!(b"<li class=\"");
                if i == 0 {
                    write_b!(b"champion");
                }
                write_b!(b"\"><b>");
                write_b!(v.name.as_bytes());
                write_b!(b"</b>: ");
                write_u16!(v.score);
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");
            black_box(slice::from_raw_parts(buf_ptr, curr));
        })
    }
}

fn raws_teams_escaped(b: &mut criterion::Bencher) {
    unsafe {
        let teams = Teams {
            year: 2015,
            teams: build_teams(),
        };
        const LEN: usize = 256;

        b.iter(|| {
            let buf = &mut [MaybeUninit::uninit(); LEN];
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();

            macro_rules! write_b {
                ($b:expr) => {
                    if LEN < curr + $b.len() {
                        panic!("buffer overflow");
                    } else {
                        for i in $b {
                            buf_ptr.add(curr).write(MaybeUninit::new(*i));
                            curr += 1;
                        }
                    }
                };
            }

            macro_rules! write_u16 {
                ($n:expr) => {
                    curr += itoa::write(
                        slice::from_raw_parts_mut(
                            buf_ptr.add(curr) as *mut _ as *mut u8,
                            LEN - curr,
                        ),
                        $n,
                    )
                    .expect("buffer overflow");
                };
            }

            write_b!(b"<html><head><title>");
            write_u16!(teams.year);
            write_b!(b"</title></head><body><h1>CSL ");
            write_u16!(teams.year);
            write_b!(b"</h1><ul>");
            for (i, v) in teams.teams.iter().enumerate() {
                write_b!(b"<li class=\"");
                if i == 0 {
                    write_b!(b"champion");
                }
                write_b!(b"\"><b>");
                curr += v_escape(v.name.as_bytes(), &mut buf[curr..]).expect("buffer overflow");
                write_b!(b"</b>: ");
                write_u16!(v.score);
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");
            black_box(slice::from_raw_parts(
                buf_ptr as *const _ as *const u8,
                curr,
            ));
        })
    }
}

// 3 bytes
const STEPS: usize = 256;
#[derive(TemplateFixed)]
#[template(src = "{{# each 0..STEPS }}{{ \"a\" * 3 }}{{/each }}")]
struct Fixed3b;

fn write_3_bytes(b: &mut criterion::Bencher) {
    b.iter(|| {
        const LEN: usize = 3 * STEPS;
        black_box(
            unsafe { TemplateFixed::call(&Fixed3b, &mut [MaybeUninit::uninit(); LEN]) }.unwrap(),
        );
    })
}

fn write_3_bytes_bb(b: &mut criterion::Bencher) {
    const BYTES: usize = 3;
    unsafe {
        b.iter(|| {
            const LEN: usize = BYTES * STEPS;
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();
            for _ in 0..STEPS {
                if LEN < curr + BYTES {
                    panic!("buffer overflow");
                } else {
                    *buf_ptr.add(curr) = b'a';
                    *buf_ptr.add(curr + 1) = b'a';
                    *buf_ptr.add(curr + 2) = b'a';
                    curr += BYTES;
                }
            }
            black_box(&buf[..curr]);
        })
    }
}

fn write_3_bytes_memcpy(b: &mut criterion::Bencher) {
    const BYTES: usize = 3;
    unsafe {
        b.iter(|| {
            const LEN: usize = BYTES * STEPS;
            let mut buf: [u8; LEN] = MaybeUninit::uninit().assume_init();
            let mut curr = 0;
            let buf_ptr = buf.as_mut_ptr();
            for _ in 0..STEPS {
                if LEN < curr + BYTES {
                    panic!("buffer overflow");
                } else {
                    const B: [u8; BYTES] = [b'a'; BYTES];
                    std::ptr::copy_nonoverlapping(
                        &B as *const [u8] as *const u8,
                        buf_ptr.add(curr),
                        BYTES,
                    );
                    curr += BYTES;
                }
            }
            black_box(&buf[..curr]);
        })
    }
}
