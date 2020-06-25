/// Adapted from [`cargo-expand`](https://github.com/dtolnay/cargo-expand)
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use bat::{PagingMode, PrettyPrinter};

use yarte_helpers::{config::PrintOption, definitely_not_nightly};

pub fn log(s: &str, path: String, option: &PrintOption) {
    if definitely_not_nightly() {
        logger(s, path, option);
    } else {
        println!("{}", s);
    }
}

fn logger(s: &str, path: String, option: &PrintOption) {
    let rustfmt =
        which_rustfmt().expect("Install rustfmt by running `rustup component add rustfmt`.");

    let mut builder = tempfile::Builder::new();
    builder.prefix("yarte");
    let outdir = builder.tempdir().expect("failed to create tmp file");
    let outfile_path = outdir.path().join("expanded");
    fs::write(&outfile_path, s).expect("correct write to file");

    // Ignore any errors.
    let _status = Command::new(rustfmt)
        .arg("--config")
        .arg("format_strings=true,max_width=120")
        .args(&["--edition", "2018"])
        .arg(&outfile_path)
        .stderr(Stdio::null())
        .status();

    let mut s = fs::read_to_string(&outfile_path).unwrap();
    if option.short.unwrap_or(false) {
        let lines: Vec<&str> = s.lines().collect();
        s = if cfg!(feature = "actix-web") {
            lines[0..lines.len() - 25].join("\n")
        } else {
            // TODO: Count lines
            lines[0..lines.len() - 5].join("\n")
        };
        s.push('\n');
    }

    let mut builder = PrettyPrinter::new();
    builder.language("rust");
    builder.header(option.header.unwrap_or(true));
    builder.grid(option.grid.unwrap_or(false));
    builder.line_numbers(option.number_line.unwrap_or(false));
    builder.paging_mode(option.paging.map_or(PagingMode::Never, |s| {
        if s {
            PagingMode::Always
        } else {
            PagingMode::Never
        }
    }));

    if let Some(theme) = option.theme {
        if builder.themes().find(|x| *x == theme).is_none() {
            let msg: Vec<String> = builder.themes().map(|x| format!("{:?}", x)).collect();
            eprintln!("Themes: {}", msg.join(",\n        "));
        } else {
            builder.theme(theme);
        }
    } else {
        builder.theme(DEFAULT_THEME);
    };

    // Ignore any errors.
    builder.input_from_bytes_with_name(s.as_bytes(), path);
    let _ = builder.print();
}

fn which_rustfmt() -> Option<PathBuf> {
    match env::var_os("RUSTFMT") {
        Some(which) => {
            if which.is_empty() {
                None
            } else {
                Some(PathBuf::from(which))
            }
        }
        None => toolchain_find::find_installed_component("rustfmt"),
    }
}

static DEFAULT_THEME: &str = "zenburn";
