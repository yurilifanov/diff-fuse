use std::collections::HashMap;
use std::fs;
use std::iter::zip;
use std::path::PathBuf;
use std::slice::Iter;
use std::str::FromStr;

use crate::error::{MergeError, ParseError};
use crate::file_diff;
use crate::file_diff::FileDiff;
use crate::macros::{debugln, parse_err};

#[derive(Debug)]
pub struct Diff {
    _order: Vec<String>,
    _map: HashMap<String, FileDiff>,
}

pub struct LineIter<'a> {
    _diff: &'a Diff,
    _file_iter: Iter<'a, String>,
    _line_iter: file_diff::LineIter<'a>,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self._line_iter.next();
            if next.is_some() {
                return next;
            }

            let next_file = self._file_iter.next()?;
            let next_diff = self._diff._map.get(next_file);
            if next_diff.is_none() {
                panic!("Diff: Entry for '{}' not found", next_file);
            }

            self._line_iter = next_diff?.line_iter();
        }
    }
}

impl Diff {
    pub fn read(path: &PathBuf) -> Result<Diff, ParseError> {
        debugln!("Reading {}", path.display());
        let data = fs::read_to_string(path)?;
        Ok(Self::from(data.parse()?))
    }

    pub fn parse(lines: &[&str]) -> Result<Diff, ParseError> {
        let mut view = &lines[..];
        let mut _order: Vec<String> = Vec::new();
        let mut _map: HashMap<String, FileDiff> = HashMap::new();

        while !view.is_empty() {
            let file_diff = FileDiff::parse(view)?;
            let file_name = file_diff.header().file_name().to_string();

            if _map.contains_key(&file_name) {
                return Err(parse_err!(
                    "Diff: Invalid diff - multiple blocks for file {}",
                    file_name
                ));
            }

            view = &view[file_diff.num_lines()..];
            _order.push(file_name.clone());
            _map.insert(file_name, file_diff);
        }

        Ok(Diff { _order, _map })
    }

    pub fn line_iter(&self) -> LineIter {
        LineIter {
            _diff: self,
            _file_iter: self._order.iter(),
            _line_iter: file_diff::LineIter::default(),
        }
    }

    pub fn merge(&self, other: &Diff) -> Result<Diff, MergeError> {
        let mut _order: Vec<String> = Vec::new();
        let mut _map: HashMap<String, FileDiff> = HashMap::new();
        for (key, val) in other._map.iter() {
            if let Some(diff) = self._map.get(key) {
                _map.insert(key.clone(), diff.merge(val)?);
            } else {
                _map.insert(key.clone(), val.clone());
            }
            _order.push(key.clone());
        }
        for (key, val) in self._map.iter() {
            if !_map.contains_key(key) {
                _map.insert(key.clone(), val.clone());
                _order.push(key.clone());
            }
        }
        Ok(Diff { _order, _map })
    }
}

fn get_line_separator(data: &str) -> &str {
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

fn get_lines(data: &str) -> Vec<&str> {
    let sep = get_line_separator(data);
    data.split_terminator(sep).collect()
}

impl FromStr for Diff {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Diff::parse(&get_lines(s)[..])
    }
}

impl ToString for Diff {
    fn to_string(&self) -> String {
        let mut string = String::new();
        for line in self.line_iter() {
            string += line;
            string += "\n";
        }
        string
    }
}
