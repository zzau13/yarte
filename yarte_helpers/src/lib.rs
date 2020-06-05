pub use std::fmt::Error;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    ffi::OsString,
    hash::{Hash, Hasher},
    process::Command,
};

pub type Result<I> = ::std::result::Result<I, Error>;

#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "config")]
pub mod recompile;

pub mod at_helpers;
pub mod helpers;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn definitely_not_nightly() -> bool {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("--version");

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return false,
    };

    let version = match String::from_utf8(output.stdout) {
        Ok(version) => version,
        Err(_) => return false,
    };

    version.starts_with("cargo 1") && !version.contains("nightly")
}

fn cargo_binary() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| "cargo".to_owned().into())
}
