use crate::error::{MergeError, ParseError};
use crate::macros::{merge_err, parse_err};
use core::cmp::{min, Ordering};

const NUM_FIELDS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub fields: [i64; NUM_FIELDS],
}

impl From<[i64; NUM_FIELDS]> for Header {
    fn from(fields: [i64; NUM_FIELDS]) -> Header {
        Header { fields }
    }
}

impl Default for Header {
    fn default() -> Header {
        Header {
            fields: [0, 0, 0, 0],
        }
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Header {
    pub fn parse(header: &str) -> Result<Header, ParseError> {
        let group_iter = header
            .strip_prefix("@@ ")
            .map_or(None, |s| s.strip_suffix(" @@")) // keep the last ' '
            .ok_or(parse_err!("Unexpected header format in '{header}'"))?
            .split(' ');

        let mut result: [i64; NUM_FIELDS] = [1; NUM_FIELDS];
        let mut i: usize = 0;

        for group in group_iter {
            for (j, field) in group.split(',').enumerate() {
                let index = i + j;
                if index >= NUM_FIELDS {
                    return Err(parse_err!(
                        "Too many header fields in '{header}'"
                    ));
                }
                result[index] = field
                    .trim_start_matches(['-', '+'])
                    .parse::<i64>()
                    .map_err(|_| {
                        parse_err!(
                            "Invalid header field '{field}' in '{header}'"
                        )
                    })?;
            }
            i += 2;
        }

        if i <= 2 {
            return Err(parse_err!("Too few header fields in '{header}'"));
        }

        Ok(result.into())
    }

    pub fn cmp(&self, other: &Header) -> Ordering {
        let [lhs_mmin, _, lhs_pmin, _] = self.fields;
        let [rhs_mmin, _, rhs_pmin, _] = other.fields;
        min(lhs_mmin, lhs_pmin).cmp(min(&rhs_mmin, &rhs_pmin))
    }

    pub fn fuse(&self, other: &Header) -> Header {
        let left = match [self.fields[1], other.fields[1]] {
            [0, 0] => min(self.fields[0], other.fields[0] - self.offset()),
            [_, 0] => self.fields[0],
            [0, _] => other.fields[0],
            _ => min(self.fields[0], other.fields[0] - self.offset()),
        };

        let right = match [self.fields[3], other.fields[3]] {
            [0, 0] => min(self.fields[2] + other.offset(), other.fields[2]),
            [_, 0] => self.fields[2],
            [0, _] => other.fields[2],
            _ => min(self.fields[2] + other.offset(), other.fields[2]),
        };

        [left, 0, right, 0].into()
    }

    pub fn overlaps(&self, other: &Header) -> bool {
        let [lhs_min, lhs_max] = self.minus_range();
        let [rhs_min, rhs_max] = other.minus_range();

        if lhs_min < rhs_max && rhs_min < lhs_max {
            return true;
        }

        let [lhs_min, lhs_max] = self.plus_range();
        let [rhs_min, rhs_max] = other.plus_range();
        if lhs_min < rhs_max && rhs_min < lhs_max {
            return true;
        }

        false
    }

    pub fn should_fuse(&self, other: &Header) -> bool {
        let [lhs_min, lhs_max] = other.minus_range();
        let [rhs_min, rhs_max] = self.plus_range();
        lhs_min < rhs_max && rhs_min < lhs_max
    }

    pub fn to_string(&self) -> String {
        let [mmin, mnum, pmin, pnum] = self.fields;
        match [mnum, pnum] {
            [1, 1] => format!("@@ -{mmin} +{pmin} @@"),
            [_, 1] => format!("@@ -{mmin},{mnum} +{pmin} @@"),
            [1, _] => format!("@@ -{mmin} +{pmin},{pnum} @@"),
            _ => format!("@@ -{mmin},{mnum} +{pmin},{pnum} @@"),
        }
    }

    pub fn with_offset(
        mut self,
        left: i64,
        right: i64,
    ) -> Result<Header, MergeError> {
        self.fields[0] += left;
        self.fields[2] += right;
        if self.fields[0] < 0 || self.fields[2] < 0 {
            return Err(merge_err!(
                "Tried to offset header {self} by {:?}",
                (left, right)
            ));
        }
        Ok(self)
    }

    pub fn is_empty(&self) -> bool {
        self.fields[1] == 0 && self.fields[3] == 0
    }

    fn offset(&self) -> i64 {
        // how many lines were added before this hunk
        (self.fields[2] + if self.fields[3] == 0 { 1 } else { 0 })
            - (self.fields[0] + if self.fields[1] == 0 { 1 } else { 0 })
    }

    fn minus_range(&self) -> [i64; 2] {
        [self.fields[0], self.fields[0] + self.fields[1]]
    }

    fn plus_range(&self) -> [i64; 2] {
        [self.fields[2], self.fields[2] + self.fields[3]]
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::header::Header;

    fn success(string: &str, expected: [i64; 4]) {
        assert_eq!(expected, Header::parse(string).unwrap().fields);
    }

    fn failure(string: &str) {
        assert_eq!(true, Header::parse(string).is_err());
    }

    #[test]
    fn case_1() {
        success("@@ 2 2 @@", [2, 1, 2, 1]);
    }

    #[test]
    fn case_2() {
        success("@@ 1,2 3 @@", [1, 2, 3, 1]);
    }

    #[test]
    fn case_3() {
        success("@@ 1 2,3 @@", [1, 1, 2, 3]);
    }

    #[test]
    fn case_4() {
        success("@@ 1,2 3,4 @@", [1, 2, 3, 4]);
    }

    #[test]
    fn case_5() {
        failure("@@ @@");
    }

    #[test]
    fn case_6() {
        failure("@@ 123123 @@");
    }

    #[test]
    fn case_7() {
        failure("@@ 123:123 @@");
    }

    #[test]
    fn case_8() {
        failure("@@ 123 : 123 @@");
    }

    #[test]
    fn case_9() {
        failure("@@ 1 2 3 4 5 @@");
    }

    #[test]
    fn case_10() {
        failure("@@ 1,2,3,4 @@");
    }

    #[test]
    fn case_11() {
        failure("@@ 1 2 3 4 @@");
    }
}
