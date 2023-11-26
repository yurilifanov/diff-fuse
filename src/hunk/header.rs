use crate::error::ParseError;
use crate::macros::parse_err;

pub fn parse(header: &str) -> Result<[usize; 4], ParseError> {
    let group_iter = header
        .strip_prefix("@@ ")
        .map_or(None, |s| s.strip_suffix(" @@")) // keep the last ' '
        .ok_or(parse_err!("Unexpected header format in '{header}'"))?
        .split(' ');

    const NUM_FIELDS: usize = 4;
    let mut result: [usize; NUM_FIELDS] = [1; NUM_FIELDS];
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
                .parse::<usize>()
                .map_err(|_| {
                    parse_err!("Invalid header field '{field}' in '{header}'")
                })?;
        }
        i += 2;
    }

    if i <= 2 {
        return Err(parse_err!("Too few header fields in '{header}'"));
    }

    Ok(result)
}

pub fn dump(header: &[usize; 4]) -> String {
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

#[cfg(test)]
mod tests {
    use crate::hunk::header::parse;

    fn success(string: &str, expected: [usize; 4]) {
        assert_eq!(expected, parse(string).unwrap());
    }

    fn failure(string: &str) {
        assert_eq!(true, parse(string).is_err());
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