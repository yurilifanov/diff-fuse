use std::slice::Iter;

use crate::error::ParseError;
use crate::header::Header;
use crate::hunk::Hunk;
use crate::macros::debugln;

#[derive(Debug)]
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

        loop {
            let hunk = Hunk::parse(view)?;
            debugln!("Got hunk {:?}", hunk);

            let hunk_lines = hunk.lines().len();
            _num_lines += hunk_lines;

            view = &view[hunk_lines..];
            _hunks.push(hunk);

            let predicate = |s: &&str| s.starts_with("Index: ");
            if view.get(0).map_or(true, predicate) {
                break;
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
}
