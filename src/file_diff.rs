use crate::error::{MergeError, ParseError};
use crate::hand::Hand;
use crate::header::Header;
use crate::hunk::handed::{HandedHunk, Mergeable};
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

    pub fn merge(mut self, other: FileDiff) -> Result<FileDiff, MergeError> {
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
        let mut lhs = self._hunks.into_iter().peekable();
        let mut rhs = other._hunks.into_iter().peekable();

        // lhs or rhs hunks in correct order
        let mut combined_iter =
            std::iter::from_fn(move || -> Option<HandedHunk> {
                let hunk: HandedHunk = match (lhs.peek(), rhs.peek()) {
                    (None, None) => {
                        return None;
                    }
                    (None, Some(_)) => (Hand::Right, rhs.next()?).into(),
                    (Some(_), None) => (Hand::Left, lhs.next()?).into(),
                    (Some(lhunk), Some(rhunk)) => match lhunk.cmp(rhunk) {
                        Ordering::Less => (Hand::Left, lhs.next()?).into(),
                        Ordering::Greater => (Hand::Right, rhs.next()?).into(),
                        Ordering::Equal => (Hand::Left, lhs.next()?).into(),
                    },
                };
                Some(hunk)
            })
            .peekable();

        // FIXME: hunk headers should be adjusted

        let merge_iter =
            std::iter::from_fn(move || -> Option<Result<Hunk, MergeError>> {
                let next = combined_iter.next()?;
                if let Some(peek) = combined_iter.peek() {
                    if !next.overlaps(peek) {
                        return Some(Ok(next.into()));
                    }

                    debugln!("Merging hunks {next} and {peek}");
                    let mut merge = match next.merge(combined_iter.next()?) {
                        Ok(m) => m,
                        Err(err) => {
                            return Some(Err(err));
                        }
                    };

                    while let Some(peek) = combined_iter.peek() {
                        if !peek.overlaps(&merge) {
                            return Some(Ok(merge.into()));
                        }

                        debugln!("Merging hunks {merge} (merged) and {peek}");
                        println!("{merge:?}");
                        merge = match combined_iter.next()?.merge(merge) {
                            Ok(m) => m,
                            Err(err) => {
                                return Some(Err(err));
                            }
                        };
                    }

                    return Some(Ok(merge.into()));
                }
                Some(Ok(next.into()))
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
mod test_parse {
    use crate::file_diff::FileDiff;

    fn test(expected: &str) {
        match FileDiff::from_lines(&mut expected.lines().peekable()) {
            Ok(result) => {
                assert_eq!(expected, result.to_string().as_str());
            }
            Err(err) => {
                panic!("{err:?}");
            }
        }
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
+b
@@ -3 +3 @@
-a
+b
",
        );
    }
}

#[cfg(test)]
mod test_merge {
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

        let merged = match ldiff.merge(rdiff) {
            Ok(diff) => diff,
            Err(err) => {
                panic!("{err:?}");
            }
        };

        assert_eq!(expected, merged.to_string().as_str());
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
}
