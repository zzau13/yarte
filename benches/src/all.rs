use criterion::{criterion_group, criterion_main, Criterion};
use std::fmt::{Display, Formatter, Result, Write};
use yarte::Template;

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    c.bench_function("Teams", teams);
    c.bench_function("Teams Unescaped", teams_display);
    c.bench_function("Formatter Teams", teams_fmt);
    c.bench_function("Big table", |b| big_table(b, 100));
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
#[template(path = "big-table.hbs")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

fn big_table_display(b: &mut criterion::Bencher, size: usize) {
    let t = BigTableDisplay {
        table: build_big_table(size),
    };
    b.iter(|| t.call().unwrap());
}

#[derive(Template)]
#[template(path = "big-table.hbs", assured = true)]
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
#[template(path = "teams.hbs")]
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

#[derive(Template)]
#[template(path = "teams.hbs", assured = true)]
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
