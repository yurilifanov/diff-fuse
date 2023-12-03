mod iter_info;
mod merge_fn;

use crate::error::MergeError;
use crate::hand::Hand;
use crate::hunk::{header, Hunk};
use crate::macros::merge_err;

#[derive(Debug)]
pub struct Merge {
    header: [usize; 4],
    data: Vec<(Hand, String)>,
}

impl Merge {
    pub fn new<
        T: Iterator<Item = (Hand, String)>,
        U: Iterator<Item = (Hand, String)>,
    >(
        lheader: &[usize; 4],
        llines: T,
        rheader: &[usize; 4],
        rlines: U,
    ) -> Result<Merge, MergeError> {
        let (header, data) =
            merge_fn::merge_fn(lheader, llines, rheader, rlines)?;
        Ok(Merge { header, data })
    }

    pub fn header(&self) -> &[usize; 4] {
        &self.header
    }

    pub fn merge(
        mut self,
        hand: Hand,
        hunk: Hunk,
    ) -> Result<Merge, MergeError> {
        let (header, iter) = hunk.into_data();
        let (header, data) = merge_fn::merge_fn(
            &header,
            std::iter::repeat(hand).zip(iter),
            &self.header,
            self.data.into_iter(),
        )?;
        Ok(Merge { header, data })
    }

    pub fn into_data(
        mut self,
    ) -> ([usize; 4], impl Iterator<Item = (Hand, String)>) {
        (self.header, self.data.into_iter())
    }
}

impl From<Merge> for Hunk {
    fn from(merge: Merge) -> Hunk {
        Hunk::new(
            merge.header,
            merge.data.into_iter().map(|(_, s)| s).collect(),
        )
    }
}

impl std::fmt::Display for Merge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", header::dump(&self.header))
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::Hunk;

    fn test(left: &str, right: &str, expected: &str) {
        let lhs = Hunk::from_lines(&mut left.lines().peekable());
        let rhs = Hunk::from_lines(&mut right.lines().peekable());
        match (lhs, rhs) {
            (Ok(lhunk), Ok(rhunk)) => match lhunk.merge(rhunk) {
                Ok(merge) => {
                    let hunk: Hunk = merge.into();
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
}
