#![allow(clippy::into_iter_on_ref)]

use yarte::Template;

#[test]
fn big_table() {
    let size = 3;
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }

    let table = BigTable { table };

    assert_eq!(
        table.call().unwrap(),
        "<table><tr><td>0</td><td>1</td><td>2</td></tr><tr><td>0</td><td>1</td><td>2</td></\
         tr><tr><td>0</td><td>1</td><td>2</td></tr></table>"
    );
}

#[derive(Template)]
#[template(path = "big-table.hbs")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

#[test]
fn teams() {
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
    assert_eq!(
        teams.call().unwrap(),
        "<html><head><title>2015</title></head><body><h1>CSL 2015</h1><ul><li \
         class=\"champion\"><b>Jiangsu</b>: 43</li><li class=\"\"><b>Beijing</b>: 27</li><li \
         class=\"\"><b>Guangzhou</b>: 22</li><li class=\"\"><b>Shandong</b>: \
         12</li></ul></body></html>"
    );
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
