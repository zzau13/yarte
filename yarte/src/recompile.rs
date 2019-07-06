use std::fs;

use yarte_config::{config_file_path, read_config_file, Config};

/// Recompile when changed. Put me on your `build.rs`
pub fn when_changed() {
    // rerun when config file change
    println!(
        "cargo:rerun-if-changed={}",
        config_file_path().to_str().unwrap()
    );

    let file = read_config_file();
    let config = Config::new(&file);

    let mut stack = vec![config.get_dir().clone()];
    loop {
        if let Some(dir) = stack.pop() {
            // rerun when dir change
            println!("cargo:rerun-if-changed={}", dir.to_str().unwrap());
            for entry in fs::read_dir(dir).expect("valid directory") {
                let path = entry.expect("valid directory entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    // rerun when file change
                    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
                }
            }
        } else {
            break;
        }
    }
}
