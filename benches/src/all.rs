use std::fmt::{Display, Formatter, Result, Write};
use std::mem::MaybeUninit;
use std::{io, slice};

use criterion::{criterion_group, criterion_main, Criterion};

use itoa;
use yarte::{Template, TemplateText};

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    c.bench_function("Unsafe Teams", max_size_teams_io_writer);
    c.bench_function("Safe Unsafe Teams", safe_max_size_teams_io_writer);
    c.bench_function("Teams", teams);
    c.bench_function("Teams io writer implements io::Write", teams_io_writer);
    c.bench_function("Teams Unescaped", teams_display);
    c.bench_function("Formatter Teams", teams_fmt);
    c.bench_function("Big table", |b| big_table(b, 100));
    c.bench_function("Big table io writer", |b| big_table_io_writer(b, 100));
    c.bench_function("Big table Unescaped", |b| big_table_display(b, 100));
    c.bench_function("Formatter Big table", |b| big_table_fmt(b, 100));
}

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

fn big_table(b: &mut criterion::Bencher, size: usize) {
    let t = BigTable {
        table: build_big_table(size),
    };
    b.iter(|| t.call().unwrap());
}

#[derive(Template)]
#[template(path = "big-table")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

fn big_table_display(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableDisplay {
        table: build_big_table(size),
    };
    b.iter(|| t.call().unwrap());
}

#[derive(TemplateText)]
#[template(path = "big-table")]
struct BigTableDisplay {
    table: Vec<Vec<usize>>,
}

fn big_table_fmt(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableFmt {
        table: build_big_table(size),
    };
    let mut buf = String::with_capacity(t.to_string().len());
    b.iter(|| {
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

struct Team {
    name: String,
    score: u8,
}

fn teams(b: &mut criterion::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: build_teams(),
    };
    b.iter(|| teams.call().unwrap());
}

#[derive(Template)]
#[template(path = "teams")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

fn teams_display(b: &mut criterion::Bencher) {
    let teams = TeamsDisplay {
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

fn teams_fmt(b: &mut criterion::Bencher) {
    let teams = TeamsFmt {
        year: 2015,
        teams: build_teams(),
    };

    let mut buf = String::with_capacity(teams.to_string().len());
    b.iter(|| {
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

fn io_writer_big_table<W: std::io::Write>(
    f: &mut W,
    table: &Vec<Vec<usize>>,
) -> std::io::Result<()> {
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

fn big_table_io_writer(b: &mut criterion::Bencher, size: usize) {
    let table = build_big_table(size);
    let mut buf = vec![];
    let _ = io_writer_big_table(&mut buf, &table);
    let mut buf = Vec::with_capacity(buf.len());
    b.iter(|| {
        let _ = io_writer_big_table(&mut buf, &table);
    });
}

fn max_size_teams_io_writer(b: &mut criterion::Bencher) {
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
                    for i in $b {
                        buf_ptr.add(curr).write(*i);
                        curr += 1;
                    }
                };
            }

            write_b!(b"<html><head><title>");
            curr += itoa::write(
                slice::from_raw_parts_mut(buf_ptr.add(curr), LEN - curr),
                teams.year,
            )
            .expect("buffer overflow");
            write_b!(b"</title></head><body><h1>CSL ");
            curr += itoa::write(
                slice::from_raw_parts_mut(buf_ptr.add(curr), LEN - curr),
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
                write_b!(v.name.as_bytes());

                write_b!(b"</b>: ");
                curr += itoa::write(
                    slice::from_raw_parts_mut(buf_ptr.add(curr), LEN - curr),
                    v.score,
                )
                .expect("buffer overflow");
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");
            let _ = slice::from_raw_parts(buf_ptr, curr).to_vec();
        })
    }
}

fn safe_max_size_teams_io_writer(b: &mut criterion::Bencher) {
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
                    if curr + $b.len() < LEN {
                        for i in $b {
                            buf_ptr.add(curr).write(*i);
                            curr += 1;
                        }
                    } else {
                        panic!("buffer overflow");
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
                // TODO: v_escape
                write_b!(v.name.as_bytes());
                write_b!(b"</b>: ");
                write_u16!(v.score);
                write_b!(b"</li>");
            }
            write_b!(b"</ul></body></html>");
            let _ = slice::from_raw_parts(buf_ptr, curr).to_vec();
        })
    }
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

fn teams_io_writer(b: &mut criterion::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: build_teams(),
    };
    let mut buf = TeamsWriter(vec![]);
    let _ = buf.io_writer_teams(&teams);
    let mut buf = TeamsWriter(Vec::with_capacity(buf.0.len()));
    b.iter(|| {
        let _ = buf.io_writer_teams(&teams);
    });
}
