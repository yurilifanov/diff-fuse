pub mod handed;
pub mod header;

use crate::error::{MergeError, ParseError};
use crate::hand::Hand;
use crate::info::{iter_info, Info};
use crate::macros::{merge_err, parse_err};
use crate::merge::Merge;
use core::cmp::{min, Ordering};
use std::iter::{repeat, Peekable};

#[derive(Clone, Debug)]
pub struct Hunk {
    _lines: Vec<String>,
    _header: [usize; 4],
}

impl Hunk {
    pub fn new(_header: [usize; 4], mut _lines: Vec<String>) -> Hunk {
        _lines.insert(0, header::dump(&_header));
        Hunk { _header, _lines }
    }

    pub fn cmp(&self, other: &Hunk) -> Ordering {
        let [lhs_mmin, _, lhs_pmin, _] = self._header;
        let [rhs_mmin, _, rhs_pmin, _] = other._header;
        min(lhs_mmin, lhs_pmin).cmp(min(&rhs_mmin, &rhs_pmin))
    }

    pub fn from_lines<'a, T: Iterator<Item = &'a str>>(
        lines: &mut Peekable<T>,
    ) -> Result<Hunk, ParseError> {
        if let Some(line) = lines.peek() {
            if !line.starts_with("@@") {
                return Err(parse_err!("Expected hunk header, got '{line}'"));
            }

            let _header = header::parse(line)?;
            let mut _lines: Vec<String> = vec![line.to_string()];
            lines.next();

            let mut counts: (usize, usize) = (0, 0);
            while let Some(line) = lines.peek() {
                match line.chars().nth(0).unwrap_or('!') {
                    '-' => {
                        counts.0 += 1;
                    }
                    '+' => {
                        counts.1 += 1;
                    }
                    ' ' => {
                        counts.0 += 1;
                        counts.1 += 1;
                    }
                    _ => {
                        break;
                    }
                }
                _lines.push(line.to_string());
                lines.next();
            }

            if counts.0 != _header[1] || counts.1 != _header[3] {
                return Err(parse_err!(
                    "Hunk validation failed: line count = {:?}, header = {:?}",
                    counts,
                    _header
                ));
            }

            Ok(Hunk { _lines, _header })
        } else {
            Err(parse_err!("Cannot parse hunk: line iterator empty"))
        }
    }

    pub fn header(&self) -> &[usize; 4] {
        &self._header
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    pub fn unpack(self) -> ([usize; 4], std::vec::IntoIter<String>) {
        let mut lines = self._lines.into_iter();
        lines.next();
        (self._header, lines)
    }

    pub fn into_data(mut self) -> ([usize; 4], impl Iterator<Item = String>) {
        (self._header, self._lines.into_iter().skip(1))
    }

    pub fn overlaps(&self, other: &Hunk) -> bool {
        header::overlap(&self._header, &other._header)
    }

    pub fn with_offset(self, offset: &i64) -> Hunk {
        self
    }

    fn num_lines_added(&self) -> i64 {
        0
    }

    fn num_lines_removed(&self) -> i64 {
        0
    }

    pub fn offset(&self) -> i64 {
        self.num_lines_added() - self.num_lines_removed()
    }

    pub fn into_lines(mut self) -> std::vec::IntoIter<String> {
        self._lines.into_iter()
    }

    pub fn into_info(
        self,
        hand: Hand,
    ) -> ([usize; 4], impl Iterator<Item = Info>) {
        let (header, lines) = self.into_data();
        (header, iter_info(&header, repeat(hand).zip(lines)))
    }

    pub fn merge<'a>(mut self, other: Hunk) -> Result<Merge, MergeError> {
        if !self.overlaps(&other) {
            return Err(merge_err!(
                "Expected hunks {:?} and {:?} to overlap, but they do not",
                self._header,
                other._header
            ));
        }
        Merge::new(
            &self._header,
            iter_info(
                &self._header,
                repeat(Hand::Left).zip(self._lines.into_iter().skip(1)),
            ),
            &other._header,
            iter_info(
                &other._header,
                repeat(Hand::Right).zip(other._lines.into_iter().skip(1)),
            ),
        )
    }
}

impl std::fmt::Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self._lines.get(0) {
            write!(f, "{}", &line)
        } else {
            write!(f, "[no lines; header = {:?}]", self._header)
        }
    }
}
