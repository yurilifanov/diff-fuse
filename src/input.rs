use std::env;
use std::path::{Path, PathBuf};

use crate::macros::debugln;

pub fn has_help_arg() -> bool {
    let predicate = |arg: &String| arg == "-h" || arg == "--help";
    env::args().find(predicate).is_some()
}

pub fn get_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in env::args().skip(1) {
        paths.push(Path::new(&arg).into());
    }
    debugln!("Received {} paths from input", paths.len());
    paths
}
