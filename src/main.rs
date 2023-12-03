mod diff;
mod error;
mod file_diff;
mod hand;
mod header;
mod hunk;
mod input;
mod macros;
mod merge;

use diff::Diff;

fn main() {
    if input::has_help_arg() {
        println!("Usage: ./diff-fuse [-h, --help] path ...");
        return;
    }

    let paths = input::get_paths();
    if paths.len() < 1 {
        println!("Expected at least one path");
        return;
    }

    let mut path_iter = paths.into_iter();
    let first_path = path_iter.next().unwrap();
    let diff = path_iter
        .fold(Diff::read(&first_path).unwrap(), |diff, path| {
            diff.merge(Diff::read(&path).unwrap()).unwrap()
        });

    for line in diff.line_iter() {
        println!("{line}");
    }
}
