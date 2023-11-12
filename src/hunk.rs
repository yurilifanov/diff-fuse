mod info_iter;
mod merge_iter;

use crate::error::{MergeError, ParseError};
use crate::hunk::merge_iter::{process, MergeIter};
use crate::macros::{merge_err, parse_err};
use core::cmp::{min, Ordering};

#[derive(Clone, Debug)]
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
            parse_err!("Hunk header: Could not parse i64 from {string}")
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
            parse_err!("Hunk header: Expected {N} integers, got {values:?}")
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
            "Hunk header: Unexpected number of fields - {values:?}"
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
                parse_err!("Hunk: Could not parse header from '{line}'")
            })?;

        parse(fields)
    }

    fn serialize_header(header: &[usize; 4]) -> String {
        let [mut mmin, mnum, mut pmin, pnum] = *header;
        if mnum == 0 {
            mmin = 0;
        }
        if pnum == 0 {
            pmin = 0;
        }
        match [mnum, pnum] {
            [1, 1] => format!("@@ -{mmin} +{pmin} @@"),
            [_, 1] => format!("@@ -{mmin},{mnum} +{pmin} @@"),
            [1, _] => format!("@@ -{mmin} +{pmin},{pnum} @@"),
            _ => format!("@@ -{mmin},{mnum} +{pmin},{pnum} @@"),
        }
    }

    pub fn cmp(&self, other: &Hunk) -> Ordering {
        let [lhs_mmin, _, lhs_pmin, _] = self._header;
        let [rhs_mmin, _, rhs_pmin, _] = other._header;
        min(lhs_mmin, lhs_pmin).cmp(min(&rhs_mmin, &rhs_pmin))
    }

    pub fn parse(lines: &[&str]) -> Result<Option<(usize, Hunk)>, ParseError> {
        let found = lines
            .iter()
            .enumerate()
            .find(|(_, s)| !["\n", "\r\n"].contains(s));

        if !found.map_or(false, |(_, s)| s.starts_with("@@")) {
            return Ok(None);
        }

        let (index, line) = found.unwrap();
        let _header: [usize; 4] = Hunk::parse_header(&line)?;

        let mut _lines: Vec<String> = vec![line.to_string()];
        let mut counts: (usize, usize) = (0, 0);
        for line in lines.iter().skip(1) {
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
        }

        if counts.0 != _header[1] || counts.1 != _header[3] {
            return Err(parse_err!(
                "Could not parse hunk: line count = {:?}, header = {:?}",
                counts,
                _header
            ));
        }

        Ok(Some((index + _lines.len(), Hunk { _lines, _header })))
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    fn minus_range(&self) -> [usize; 2] {
        [self._header[0], self._header[0] + self._header[1]]
    }

    fn plus_range(&self) -> [usize; 2] {
        [self._header[2], self._header[2] + self._header[3]]
    }

    pub fn overlaps(&self, other: &Hunk) -> bool {
        {
            let [lhs_min, lhs_max] = self.minus_range();
            let [rhs_min, rhs_max] = other.minus_range();
            if lhs_min <= rhs_max && rhs_min <= lhs_max {
                return true;
            }
        }
        {
            let [lhs_min, lhs_max] = self.plus_range();
            let [rhs_min, rhs_max] = other.plus_range();
            if lhs_min <= rhs_max && rhs_min <= lhs_max {
                return true;
            }
        }
        false
    }

    pub fn merge<'a>(&'a self, other: &'a Hunk) -> Result<Hunk, MergeError> {
        if !self.overlaps(other) {
            return Err(merge_err!(
                "Expected hunks {:?} and {:?} to overlap, but they do not",
                self._header,
                other._header
            ));
        }
        let as_str = |s: &'a String| -> &'a str { s.as_str() };
        let (_header, mut _lines) = process(MergeIter::new(
            (self._header, other._header),
            (
                self._lines[1..].iter().map(as_str),
                other._lines[1..].iter().map(as_str),
            ),
        ))?;
        _lines.insert(0, Self::serialize_header(&_header));
        Ok(Hunk { _header, _lines })
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

#[cfg(test)]
mod test_merge {
    use crate::hunk::Hunk;

    fn test(left: &str, right: &str, expected: &str) {
        let lhs = Hunk::parse(&left.lines().collect::<Vec<&str>>()[..]);
        let rhs = Hunk::parse(&right.lines().collect::<Vec<&str>>()[..]);
        match (&lhs, &rhs) {
            (Ok(Some((_, lhunk))), Ok(Some((_, rhunk)))) => {
                match lhunk.merge(&rhunk) {
                    Ok(merged) => {
                        let actual: Vec<_> =
                            merged._lines.iter().map(|s| s.as_str()).collect();
                        assert_eq!(
                            actual,
                            expected.lines().collect::<Vec<&str>>()
                        );
                    }
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
            _ => panic!("Enexpected case: {:?}", (lhs, rhs)),
        }
    }

    #[test]
    fn case_1() {
        test(
            "\
@@ -1 +1 @@
-a
+b
",
            "\
@@ -1 +1,2 @@
-b
+c
+d
",
            "\
@@ -1 +1,2 @@
-a
+c
+d
",
        );
    }

    #[test]
    fn case_2() {
        test(
            "\
@@ -2,4 +2,5 @@
 3
 4
 5
+6
 7
",
            "\
@@ -1,5 +1,6 @@
 1
+2
 3
 4
 5
 6
",
            "\
@@ -1,5 +1,7 @@
 1
+2
 3
 4
 5
+6
 7
",
        );
    }
}
