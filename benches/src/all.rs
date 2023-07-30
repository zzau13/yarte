#![feature(proc_macro_hygiene, stmt_expr_attributes)]

use criterion::{criterion_group, criterion_main, Criterion};

use yarte::yarte;

criterion_group!(benches, functions);
criterion_main!(benches);

fn functions(c: &mut Criterion) {
    // Teams
    c.bench_function("Teams", teams);

    // Big table
    const SIZE: usize = 100;
    c.bench_function("Big table", |b| big_table(b, SIZE));
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

fn teams(b: &mut criterion::Bencher) {
    let year = 2015u16;
    let teams = build_teams();

    b.iter(|| {
        let _: String = #[yarte] "{{> teams }}";
    });
}

fn big_table(b: &mut criterion::Bencher, size: usize) {
    let table = build_big_table(size);
    b.iter(|| {
        let _: String = #[yarte] "{{> big-table }}";
    });
}
