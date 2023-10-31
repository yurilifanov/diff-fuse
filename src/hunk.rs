mod info_iter;
mod merge_iter;

// use std::cmp::{max, min};

use crate::error::ParseError;
use crate::hunk::info_iter::InfoIter;
use crate::macros::parse_err;

#[derive(Debug)]
pub struct Hunk {
    _lines: Vec<String>,
    _header: [usize; 4],
}

fn split(string: &str) -> Vec<&str> {
    string.split(' ').flat_map(|s| s.split(',')).collect()
}

fn parse_usize(string: &&str) -> Result<usize, ParseError> {
    string
        .parse::<i64>()
        .map(|val| val.wrapping_abs() as usize)
        .map_err(|_| {
            parse_err!("Hunk header: Could not parse i64 from {}", string)
        })
}

fn parse_n<const N: usize>(
    values: &Vec<&str>,
) -> Result<[usize; N], ParseError> {
    values
        .iter()
        .flat_map(parse_usize)
        .collect::<Vec<usize>>()
        .try_into()
        .map_err(|_| {
            parse_err!(
                "Hunk header: Expected {} integers, got {:?}",
                N,
                values
            )
        })
}

fn parse(values: Vec<&str>) -> Result<[usize; 4], ParseError> {
    match values.len() {
        2 => {
            let result = parse_n::<2>(&values)?;
            Ok([result[0], 1, result[1], 1])
        }
        3 => {
            let result = parse_n::<3>(&values)?;
            if values.get(1).map_or(false, |s| s.starts_with('+')) {
                return Ok([result[0], 1, result[1], result[2]]);
            }
            Ok([result[0], result[1], result[2], 1])
        }
        4 => parse_n::<4>(&values),
        _ => Err(parse_err!(
            "Hunk header: Unexpected number of fields - {:?}",
            values
        )),
    }
}

impl Hunk {
    fn parse_header(line: &&str) -> Result<[usize; 4], ParseError> {
        let fields = line
            .strip_prefix("@@ ")
            .map_or_else(|| None, |s| s.strip_suffix(" @@"))
            .map(split)
            .ok_or_else(|| {
                parse_err!("Hunk: Could not parse header from '{}'", line)
            })?;

        parse(fields)
    }

    pub fn parse(lines: &[&str]) -> Result<Hunk, ParseError> {
        let first = lines
            .get(0)
            .ok_or(parse_err!("Hunk: Could not fetch first line"))?;

        let _header: [usize; 4] = Hunk::parse_header(&first)?;

        let mut _lines: Vec<String> = vec![first.to_string()];
        for line in lines.iter().skip(1) {
            if !line.starts_with(['+', '-', ' ']) {
                break;
            }
            _lines.push(line.to_string());
        }

        Ok(Hunk { _lines, _header })
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    // fn start_number(&self) -> usize {
    //     min(self._header[0], self._header[2])
    // }
    //
    // fn end_number(&self) -> usize {
    //     self.start_number() + max(self._header[1], self._header[3])
    // }
    //
    // fn number_range(&self) -> [usize; 2] {
    //     [self.start_number(), self.end_number()]
    // }
    //
    // pub fn overlaps(&self, other: &Hunk) -> bool {
    //     let [this_start, this_end] = self.number_range();
    //     let [that_start, that_end] = other.number_range();
    //     return this_start <= that_end && this_end >= that_start;
    // }

    // fn line_iter(&self) -> LineIter {
    //     LineIter::new(self._lines[1..].iter()) // skip header
    // }
}

// #[cfg(test)]
// mod test_merge {
//     use crate::hunk::Hunk;
//
//     #[test]
//     fn case_1() {
//         let left = "\
// @@ -1 +1 @@
// -a
// +b
// "
//         .lines()
//         .collect::<Vec<&str>>();
//         let right = "\
// @@ -1 +1,2 @@
// -b
// +c
// +d
// "
//         .lines()
//         .collect::<Vec<&str>>();
//         let expected = "\
// @@ -1 +1,2 @@
// -b
// +c
// +d
// "
//         .lines()
//         .collect::<Vec<&str>>();
//
//         let first = Hunk::parse(&left[..]).unwrap();
//         let second = Hunk::parse(&right[..]).unwrap();
//     }
// }
