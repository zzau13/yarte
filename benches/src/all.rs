use criterion::{criterion_group, criterion_main, Criterion};

use yarte::{Template, TemplateBytes, TemplateBytesText, TemplateText};

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    // Teams
    c.bench_function("Bytes Text Teams", bytes_text_teams);
    c.bench_function("Bytes Teams", bytes_teams);
    c.bench_function("Teams", teams);
    c.bench_function("Teams Unescaped", teams_text);

    // Big table
    const SIZE: usize = 100;
    c.bench_function("Bytes Big Table", |b| bytes_big_table(b, SIZE));
    c.bench_function("Bytes Text Big Table", |b| bytes_text_big_table(b, SIZE));
    c.bench_function("Big table", |b| big_table(b, SIZE));
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
    b.iter(|| teams.call::<String>(2048));
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
    b.iter(|| teams.call::<String>(2048));
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
    b.iter(|| t.call::<String>(109915));
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
    b.iter(|| t.call::<String>(109915));
}
