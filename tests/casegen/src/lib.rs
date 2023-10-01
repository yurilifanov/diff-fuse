extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::PathBuf;
use std::{env, fs};

fn quote(string: &str) -> String {
    format!("\"{}\"", string)
}

fn unquote(string: &String) -> &str {
    string.trim_start_matches("\"").trim_end_matches("\"")
}

fn manifest_dir() -> PathBuf {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = PathBuf::from(dir);
    if !path.is_dir() {
        panic!("CARGO_MANIFEST_DIR: {} is not a directory", path.display());
    }
    path
}

fn pathbuf_expr(path: &PathBuf) -> String {
    let string = path.to_str().unwrap();
    format!("PathBuf::from({})", quote(string))
}

#[proc_macro]
pub fn for_each_file(_item: TokenStream) -> TokenStream {
    println!("{:?}", _item);

    let mut itr = _item.into_iter();
    let path_suffix = itr.next().unwrap().to_string();
    println!("path_suffix = {}", path_suffix);

    let _ = itr.next().unwrap(); // Punct

    let callable = itr.next().unwrap().to_string();
    println!("callable = {}", callable);

    let source_dir = manifest_dir().join(unquote(&path_suffix));
    if !source_dir.is_dir() {
        panic!("{} is not a directory!", source_dir.display());
    }

    let paths = fs::read_dir(source_dir)
        .unwrap()
        .map(|r| r.map(|e| e.path()));

    let mut cases = String::new();
    for entry in paths {
        let path = entry.unwrap();
        let expr = pathbuf_expr(&path);
        let call = format!("{}({})", callable, expr);
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let case = format!("#[test]\nfn case_{}() {{{}}}\n", stem, call);
        cases += case.as_str();
    }

    cases.parse().unwrap()
}
