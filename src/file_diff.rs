use crate::error::{MergeError, ParseError};
use crate::fuse::fuse_iter::fuse_iter;
use crate::header::Header;
use crate::hunk::Hunk;
use crate::macros::{debugln, merge_err, parse_err};
use core::cmp::Ordering;
use std::iter::Peekable;
use std::slice::Iter;

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
    pub fn from_lines<'a, T: Iterator<Item = &'a str>>(
        mut lines: &mut Peekable<T>,
    ) -> Result<FileDiff, ParseError> {
        let _header = Header::from_lines(lines)?;
        let mut _num_lines = _header.lines().len();
        let mut _hunks: Vec<Hunk> = Vec::new();
        while let Some(line) = lines.peek() {
            if line.chars().all(char::is_whitespace) {
                lines.next();
                continue;
            } else if !line.starts_with("@@") {
                break;
            }
            let hunk = Hunk::from_lines(&mut lines)?;
            debugln!("Parsed hunk {hunk}");
            _num_lines += hunk.lines().len();
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

    pub fn fuse(mut self, other: FileDiff) -> Result<FileDiff, MergeError> {
        let mut hunks: Vec<Hunk> = Vec::new();
        let mut _num_lines = self._header.lines().len();

        for item in fuse_iter(self._hunks, other._hunks) {
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

impl ToString for FileDiff {
    fn to_string(&self) -> String {
        let mut string = String::new();
        for line in self.line_iter() {
            string += line;
            string += "\n";
        }
        string
    }
}

#[cfg(test)]
mod tests {
    use crate::file_diff::FileDiff;

    fn test(lhs: &str, rhs: &str, expected: &str) {
        let ldiff = match FileDiff::from_lines(&mut lhs.lines().peekable()) {
            Ok(diff) => diff,
            Err(err) => {
                panic!("{err:?}");
            }
        };

        let rdiff = match FileDiff::from_lines(&mut rhs.lines().peekable()) {
            Ok(diff) => diff,
            Err(err) => {
                panic!("{err:?}");
            }
        };

        let fused = match ldiff.fuse(rdiff) {
            Ok(diff) => diff,
            Err(err) => {
                panic!("{err:?}");
            }
        };

        assert_eq!(fused.to_string().as_str(), expected);
    }

    #[test]
    fn case_1() {
        test(
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-a
+b
",
            "\
    Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-b
+c
",
            "\
    Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-a
+c
",
        );
    }

    #[test]
    fn case_2() {
        test(
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-a
+1
@@ -2 +2 @@
-b
+2
@@ -3 +3 @@
-c
+3
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,3 +1,3 @@
-1
-2
-3
+i
+ii
+iii
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,3 +1,3 @@
-a
-b
-c
+i
+ii
+iii
",
        );
    }

    #[test]
    fn case_3() {
        test(
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,3 +1,3 @@
-1
-2
-3
+i
+ii
+iii
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-i
+a
@@ -2 +2 @@
-ii
+b
@@ -3 +3 @@
-iii
+c
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,3 +1,3 @@
-1
-2
-3
+a
+b
+c
",
        );
    }

    #[test]
    fn case_4() {
        test(
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1 +1 @@
-1
+i
@@ -2,2 +2,2 @@
-2
-3
+ii
+iii
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,2 +1,2 @@
-i
-ii
+a
+b
@@ -3 +3 @@
-iii
+c
",
            "\
Index: test.txt
===================================================================
--- test.txt
+++ test.txt
@@ -1,3 +1,3 @@
-1
-2
-3
+a
+b
+c
",
        );
    }

    #[test]
    fn case_5() {
        test(
            "\
Index: text.txt
===================================================================
--- text.txt
+++ text.txt
@@ -6 +6 @@
-5
+e
",
            "\
Index: text.txt
===================================================================
--- text.txt
+++ text.txt
@@ -1 +0,0 @@
-0
@@ -9 +8 @@
-8
+viii
",
            "\
Index: text.txt
===================================================================
--- text.txt
+++ text.txt
@@ -1 +0,0 @@
-0
@@ -6 +5 @@
-5
+e
@@ -9 +8 @@
-8
+viii
",
        );
    }
}
