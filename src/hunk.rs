pub mod header;
mod info_source;

pub use header::Header;

use crate::error::{MergeError, ParseError};
use crate::fuse::core::fuse;
use crate::macros::{merge_err, parse_err};

use core::cmp::{min, Ordering};
use std::iter::Peekable;

#[derive(Clone, Debug)]
pub struct Hunk {
    _lines: Vec<String>,
    _header: Header,
}

impl Hunk {
    pub fn new(_header: Header, mut _lines: Vec<String>) -> Hunk {
        _lines.insert(0, _header.to_string());
        Hunk { _header, _lines }
    }

    pub fn cmp(&self, other: &Hunk) -> Ordering {
        self._header.cmp(&other._header)
    }

    pub fn from_lines<'a, T: Iterator<Item = &'a str>>(
        lines: &mut Peekable<T>,
    ) -> Result<Hunk, ParseError> {
        if let Some(line) = lines.peek() {
            if !line.starts_with("@@") {
                return Err(parse_err!("Expected hunk header, got '{line}'"));
            }

            let _header = Header::parse(line)?;
            let mut _lines: Vec<String> = vec![line.to_string()];
            lines.next();

            let mut counts: (i64, i64) = (0, 0);
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

            if counts.0 != _header.fields[1] || counts.1 != _header.fields[3] {
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

    pub fn header(&self) -> &Header {
        &self._header
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    pub fn unpack(self) -> (Header, std::vec::IntoIter<String>) {
        let mut lines = self._lines.into_iter();
        lines.next();
        (self._header, lines)
    }

    pub fn overlaps(&self, other: &Hunk) -> bool {
        self._header.overlaps(&other._header)
    }

    pub fn with_offset(
        self,
        left: i64,
        right: i64,
    ) -> Result<Hunk, MergeError> {
        let _header = self._header.with_offset(left, right)?;
        let mut _lines = self._lines;
        _lines[0] = _header.to_string();
        Ok(Hunk { _header, _lines })
    }

    pub fn offset(&self) -> i64 {
        let mut num_added = 0i64;
        let mut num_removed = 0i64;
        for line in self._lines.iter() {
            if line.starts_with('-') {
                num_removed += 1;
            }
            if line.starts_with('+') {
                num_added += 1;
            }
        }
        num_added - num_removed
    }

    pub fn fuse(mut self, other: Hunk) -> Result<Hunk, MergeError> {
        if !self.header().should_fuse(other.header()) {
            return Err(merge_err!(
                "Expected hunks {} and {} to overlap, but they do not",
                self,
                other
            ));
        }

        // if self.cmp(&other) == Ordering::Less {
        //     self = self.with_offset(other.offset())?;
        // }

        fuse(
            self._header.fuse(&other._header),
            info_source::InfoSource::new(self, other),
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

#[cfg(test)]
mod tests {
    use crate::hunk::Hunk;

    fn test(left: &str, right: &str, expected: &str) {
        let lhs = Hunk::from_lines(&mut left.lines().peekable());
        let rhs = Hunk::from_lines(&mut right.lines().peekable());
        match (lhs, rhs) {
            (Ok(lhunk), Ok(rhunk)) => match lhunk.fuse(rhunk) {
                Ok(hunk) => {
                    let actual: Vec<_> =
                        hunk.lines().iter().map(|s| s.as_str()).collect();
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

    #[test]
    fn case_3() {
        test(
            "\
@@ -3 +3 @@
-c
+C
",
            "\
@@ -1,3 +0,0 @@
-a
-b
-C
",
            "\
@@ -1,3 +0,0 @@
-a
-b
-c
",
        );
    }

    #[test]
    fn case_4() {
        test(
            "\
@@ -1,0 +3,1 @@
+d
",
            "\
@@ -1,3 +1,4 @@
+a
 b
 c
 d
",
            "\
@@ -1,2 +1,4 @@
+a
 b
 c
+d
",
        );
    }

    #[test]
    fn case_5() {
        test(
            "\
@@ -9 +6,2 @@
 9
+x
",
            "\
@@ -6 +5,0 @@
-9
",
            "\
@@ -9 +6 @@
-9
+x
",
        );
    }

    #[test]
    fn case_6() {
        test(
            "\
@@ -9 +6,2 @@
 9
+x
",
            "\
@@ -5,2 +5,1 @@
 8
-9
",
            "\
@@ -8,2 +5,2 @@
 8
-9
+x
",
        );
    }

    #[test]
    fn case_7() {
        test(
            "\
@@ -2,0 +3,2 @@
+3
+4
",
            "\
@@ -3,2 +2,0 @@
-3
-4
",
            "\
@@ -0,0 +0,0 @@
",
        );
    }
}
