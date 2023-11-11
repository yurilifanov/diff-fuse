use std::slice::Iter;

use crate::error::{MergeError, ParseError};
use crate::header::Header;
use crate::hunk::Hunk;
use crate::macros::{debugln, merge_err, parse_err};
use core::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct FileDiff {
    _header: Header,
    _hunks: Vec<Hunk>,
    _num_lines: usize,
}

pub struct LineIter<'a> {
    _hunk_iter: Iter<'a, Hunk>,
    _line_iter: Iter<'a, String>,
}

impl LineIter<'_> {
    pub fn default() -> LineIter<'static> {
        LineIter {
            _hunk_iter: Iter::default(),
            _line_iter: Iter::default(),
        }
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self._line_iter.next();
            if next.is_some() {
                return next;
            }
            let next_hunk = self._hunk_iter.next()?;
            self._line_iter = next_hunk.lines().iter();
        }
    }
}

impl FileDiff {
    pub fn parse(lines: &[&str]) -> Result<FileDiff, ParseError> {
        let _header = Header::parse(lines)?;
        let mut _num_lines = _header.lines().len();
        let mut view = &lines[_num_lines..];
        let mut _hunks: Vec<Hunk> = Vec::new();

        while let Some((num_consumed, hunk)) = Hunk::parse(view)? {
            debugln!("Parsed hunk {hunk}");
            _num_lines += num_consumed;
            view = &view[num_consumed..];
            _hunks.push(hunk);
        }

        for (i, lhs) in _hunks.iter().enumerate() {
            for rhs in _hunks.iter().skip(i + 1) {
                if lhs.overlaps(rhs) {
                    return Err(parse_err!(
                        "Could not parse file {}: hunks {} and {} overlap",
                        _header.file_name(),
                        lhs,
                        rhs
                    ));
                }
            }
        }

        Ok(FileDiff {
            _header,
            _hunks,
            _num_lines,
        })
    }

    pub fn num_lines(&self) -> usize {
        self._num_lines
    }

    pub fn header(&self) -> &Header {
        &self._header
    }

    pub fn line_iter(&self) -> LineIter {
        LineIter {
            _hunk_iter: self._hunks.iter(),
            _line_iter: self._header.lines().iter(),
        }
    }

    pub fn merge(&self, other: &FileDiff) -> Result<FileDiff, MergeError> {
        // A file diff is an ordered set X[i], i >= 0 of non-overlapping hunks.
        // Consider two file diffs, X and Y.
        //
        // If X[i].overlaps(Y[j]) is false, then:
        //   1. X[i].overlaps(Y[k]) is false for k > j
        //   2. X[i].overlaps(X[l].merge(Y[k])) is false for l > i and k >= j
        //
        // To see why point 2. applies consider two overlapping hunks x & y.
        // For headers rx, ry and rxy of x, y amd x.merge(y) respectively:
        //   1. rxy[0] = min(rx[0], ry[0])
        //   2. rxy[2] = min(rx[2], ry[2])
        //
        // So if hunk z doesn't overlap x or y, it's clear that:
        //   1. rz[0] + rz[1] < min(rx[0], ry[0])
        //   2. rz[2] + rz[3] < min(rx[2], ry[2])
        let (lhs_file, rhs_file) =
            (self._header.file_name(), other._header.file_name());
        if lhs_file != rhs_file {
            return Err(merge_err!(
                "File names {lhs_file} and {rhs_file} do not match"
            ));
        }

        debugln!("Merging {lhs_file}");
        let mut lhs = self._hunks.iter().peekable();
        let mut rhs = other._hunks.iter().peekable();

        // lhs or rhs hunks in correct order
        let mut combined_iter =
            std::iter::from_fn(move || -> Option<&Hunk> {
                match (lhs.peek(), rhs.peek()) {
                    (None, None) => None,
                    (None, Some(_)) => rhs.next(),
                    (Some(_), None) => lhs.next(),
                    (Some(lhunk), Some(rhunk)) => match lhunk.cmp(rhunk) {
                        Ordering::Less => lhs.next(),
                        Ordering::Greater => rhs.next(),
                        Ordering::Equal => lhs.next(),
                    },
                }
            })
            .peekable();

        // merged or cloned hunks
        let merge_iter =
            std::iter::from_fn(move || -> Option<Result<Hunk, MergeError>> {
                let next = combined_iter.next()?;
                if let Some(peek) = combined_iter.peek() {
                    if !next.overlaps(peek) {
                        return Some(Ok(next.clone()));
                    }

                    // TODO: there must be a more egonomic way
                    debugln!("Merging hunks {next} and {peek}");
                    let mut merged = match next.merge(peek) {
                        Ok(hunk) => hunk,
                        Err(err) => {
                            return Some(Err(err));
                        }
                    };
                    combined_iter.next();

                    while let Some(peek) = combined_iter.peek() {
                        if !merged.overlaps(peek) {
                            return Some(Ok(merged));
                        }
                        merged = match merged.merge(peek) {
                            Ok(hunk) => hunk,
                            Err(err) => {
                                return Some(Err(err));
                            }
                        };
                        combined_iter.next();
                    }

                    return Some(Ok(merged));
                }
                None
            });

        let mut hunks: Vec<Hunk> = Vec::new();
        let mut _num_lines = self._header.lines().len();
        for item in merge_iter {
            let hunk = item?;
            _num_lines += hunk.lines().len();
            hunks.push(hunk);
        }

        Ok(FileDiff {
            _header: self._header.clone(),
            _hunks: hunks,
            _num_lines,
        })
    }
}
