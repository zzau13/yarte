extern crate yarte;
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use yarte::Template;

mod fmt;
mod std_write;

criterion_main!(benches);
criterion_group!(benches, functions);

fn functions(c: &mut Criterion) {
    c.bench_function("Teams", teams);
    c.bench_function("Write Teams", std_write::teams);
    c.bench_function("Formatter Teams", fmt::teams);
    c.bench_function("Big table", |b| big_table(b, &100));
    c.bench_function("Write Big table", |b| std_write::big_table(b, &100));
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
