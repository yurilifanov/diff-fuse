mod error;
mod file_diff;
mod header;
mod hunk;
mod input;
mod macros;

use std::fs;
use std::iter::zip;

use file_diff::FileDiff;
use macros::debugln;

fn get_line_separator(data: &String) -> &str {
    if data.find("\r\n").is_none() {
        debugln!("LF separator detected");
        return "\n";
    }

    debugln!("CRLF separator detected");
    let last = data.len() - 1;
    let zipped = zip(data[..last].chars(), data[1..].chars());
    for (prev, curr) in zipped {
        if curr == '\n' && prev != '\r' {
            panic!("Inconsistent line separator");
        }
    }
    "\r\n"
}

fn get_lines(data: &String) -> Vec<&str> {
    let sep = get_line_separator(data);
    data.split_terminator(sep).collect()
}

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

    let lines = get_lines(&data);
    debugln!("{} lines read", lines.len());

    let file_diff = FileDiff::parse(&lines[..]).unwrap();
    debugln!("{:?}", file_diff);
}
