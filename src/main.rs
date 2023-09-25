use std::{env, fs};
use std::path::{Path, PathBuf};

fn get_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in env::args() {
        paths.push(Path::new(&arg).into());
    }
    paths
}

fn main() {
    let paths = get_paths();
    for path in paths.as_slice() {
        assert!(path.is_file());
    }

    let path = paths.get(1).unwrap();
    println!("{}", path.display());

    let data = fs::read_to_string(path).unwrap();
    println!("{}", data);
}
