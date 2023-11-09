mod diff;
mod error;
mod file_diff;
mod header;
mod hunk;
mod input;
mod macros;

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

    let diff = paths.iter().skip(1).fold(
        Diff::read(paths.get(0).unwrap()).unwrap(),
        |diff, path| {
            let next = Diff::read(path).unwrap();
            diff.merge(&next).unwrap()
        },
    );

    for line in diff.line_iter() {
        println!("{line}");
    }
}
