mod hand;
mod header;
mod iter_info;
mod merge;

use crate::error::{MergeError, ParseError};
use crate::macros::{merge_err, parse_err};
use core::cmp::{min, Ordering};
use std::iter::Peekable;

#[derive(Clone, Debug)]
pub struct Hunk {
    _lines: Vec<String>,
    _header: [usize; 4],
}

impl Hunk {
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

    pub fn merge<'a>(mut self, other: Hunk) -> Result<Hunk, MergeError> {
        if !self.overlaps(&other) {
            return Err(merge_err!(
                "Expected hunks {:?} and {:?} to overlap, but they do not",
                self._header,
                other._header
            ));
        }
        let (_header, mut _lines) = merge::merge(
            &self._header,
            self._lines.into_iter().skip(1),
            &other._header,
            other._lines.into_iter().skip(1),
        )?;
        _lines.insert(0, header::dump(&_header));
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
        let lhs = Hunk::from_lines(&mut left.lines().peekable());
        let rhs = Hunk::from_lines(&mut right.lines().peekable());
        match (lhs, rhs) {
            (Ok(lhunk), Ok(rhunk)) => match lhunk.merge(rhunk) {
                Ok(merged) => {
                    let actual: Vec<_> =
                        merged._lines.iter().map(|s| s.as_str()).collect();
                    assert_eq!(
                        actual,
                        expected.lines().collect::<Vec<&str>>()
                    );
                }
                Err(err) => panic!("Error: {:?}", err),
            },
            (left, right) => panic!("Unexpected case: {:?}", (left, right)),
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
