use criterion;
use std::fmt::{Display, Formatter, Result, Write};

pub fn big_table(b: &mut criterion::Bencher, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }

    let table = BigTable { table };
    let mut buf = String::with_capacity(table.to_string().len());
    b.iter(|| {
        write!(buf, "{}", table).unwrap();
    });
}

struct BigTable {
    table: Vec<Vec<usize>>,
}

impl Display for BigTable {
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

pub fn teams(b: &mut criterion::Bencher) {
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

    // count with wc -c
    let mut buf = String::with_capacity(teams.to_string().len());
    b.iter(|| {
        write!(buf, "{}", teams).unwrap();
    });
}

struct Teams {
    year: u16,
    teams: Vec<Team>,
}

impl Display for Teams {
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

struct Team {
    name: String,
    score: u8,
}
