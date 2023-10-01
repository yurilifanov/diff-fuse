mod diff;
mod error;
mod file_diff;
mod header;
mod hunk;
mod input;
mod macros;

use std::fs;

use diff::Diff;
use macros::debugln;

fn main() {
    if input::has_help_arg() {
        println!("Usage: ./diff-fuse [-h, --help] path ...");
        return;
    }

    let paths = input::get_paths();
    for path in paths.as_slice() {
        assert!(path.is_file());
    }

    let path = paths.get(0).unwrap();
    debugln!("{}", path.display());

    let data = fs::read_to_string(path).unwrap();
    debugln!("{}", data);

    // let lines = get_lines(&data);
    // debugln!("{} lines read", lines.len());

    let diff = Diff::from(data.parse().unwrap());
    debugln!("{:?}", diff);

    for line in diff.line_iter() {
        println!("{}", line);
    }
}
