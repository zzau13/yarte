/// Adapted from [`cargo-expand`](https://github.com/dtolnay/cargo-expand)
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use prettyprint::{PagingMode, PrettyPrinter};

use yarte_config::PrintOption;
use yarte_helpers::helpers::definitely_not_nightly;

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

    let mut builder = PrettyPrinter::default();
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

    let printer = if let Some(theme) = option.theme {
        builder.theme(theme);
        let printer = builder.build().unwrap();
        let themes = printer.get_themes();
        if themes.get(theme).is_none() {
            let msg: Vec<String> = themes.keys().map(|x| format!("{:?}", x)).collect();
            eprintln!("Themes: {}", msg.join(",\n        "));
        }
        printer
    } else {
        builder.theme(DEFAULT_THEME);
        builder.build().unwrap()
    };

    // Ignore any errors.
    let _ = printer.string_with_header(s, path);
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
