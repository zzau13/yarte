use criterion::{criterion_group, criterion_main, Criterion};
use yarte::Template;

mod fmt;

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    c.bench_function("Teams", teams);
    c.bench_function("Formatter Teams", fmt::teams);
    c.bench_function("Big table", |b| big_table(b, &100));
    c.bench_function("Formatter Big table", |b| fmt::big_table(b, &100));
}

fn big_table(b: &mut criterion::Bencher, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }
    let ctx = BigTable { table };
    b.iter(|| ctx.call().unwrap());
}

#[derive(Template)]
#[template(path = "big-table.hbs")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

fn teams(b: &mut criterion::Bencher) {
    let teams = Teams {
        year: 2015,
        teams: vec![
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
        ],
    };
    b.iter(|| teams.call().unwrap());
}

#[derive(Template)]
#[template(path = "teams.hbs")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}

struct Team {
    name: String,
    score: u8,
}
