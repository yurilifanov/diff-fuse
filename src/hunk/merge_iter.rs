use crate::error::MergeError;
use crate::hunk::info_iter::InfoIter;
use crate::macros::merge_err;
use core::cmp::{min, Ordering};
use std::iter::Peekable;

pub struct MergeIter<'a, T: Iterator<Item = &'a str> + Clone> {
    lhs: Peekable<InfoIter<'a, T>>,
    rhs: Peekable<InfoIter<'a, T>>,
    start_nums: (usize, usize),
    synced: bool,
}

impl<'a, T: Iterator<Item = &'a str> + Clone> MergeIter<'a, T> {
    pub fn new(
        headers: ([usize; 4], [usize; 4]),
        lines: (T, T),
    ) -> MergeIter<'a, T> {
        MergeIter {
            lhs: InfoIter::new(&headers.0, lines.0).peekable(),
            rhs: InfoIter::new(&headers.1, lines.1).peekable(),
            start_nums: (
                min(headers.0[0], headers.1[0]),
                min(headers.0[2], headers.1[2]),
            ),
            synced: false,
        }
    }
}

pub fn process<'a, T: Iterator<Item = &'a str> + Clone>(
    iter: MergeIter<'a, T>,
) -> Result<([usize; 4], Vec<String>), MergeError> {
    let mut header: [usize; 4] = [iter.start_nums.0, 0, iter.start_nums.1, 0];
    let mut counters: (usize, usize, usize) = (0, 0, 0);

    // returns (group, index)
    let mut update_counters = |s: &String| {
        if s.starts_with('-') {
            counters.1 += 1;
            header[1] += 1;
            (counters.0, counters.1)
        } else if s.starts_with('+') {
            counters.2 += 1;
            header[3] += 1;
            (counters.0, counters.2)
        } else {
            counters = (counters.0 + 1, 0, 0);
            header[1] += 1;
            header[3] += 1;
            (counters.0, 1)
        }
    };

    let mut lines: Vec<((usize, usize), String)> = Vec::new();
    for item in iter {
        let line = item?;
        lines.push((update_counters(&line), line));
    }

    Ok((
        header,
        sort_lines(lines)?.into_iter().map(|(_, s)| s).collect(),
    ))
}

fn sort_lines(
    mut lines: Vec<((usize, usize), String)>,
) -> Result<Vec<((usize, usize), String)>, MergeError> {
    let mut err: Option<MergeError> = None;
    let mut update_err = |e: MergeError| {
        if err.is_none() {
            err = Some(e);
        }
    };

    // each group can only have one line with prefix ' ', so order:
    // - within each group
    //     - ' ' line, if any, first
    //     - '-' lines, if any, follow
    //     - '+' lines, if any, last
    // - keep the order of lines according to their index
    // - keep the group order according to the group index
    lines.sort_by(
        |((lhs_group, lhs_index), lhs_line),
         ((rhs_group, rhs_index), rhs_line)| {
            if lhs_group != rhs_group {
                return lhs_group.cmp(rhs_group);
            }

            let lhs_prefix = if let Some(val) = lhs_line.chars().nth(0) {
                val
            } else {
                update_err(merge_err!("Empty line in sort"));
                ' '
            };

            let rhs_prefix = if let Some(val) = rhs_line.chars().nth(0) {
                val
            } else {
                update_err(merge_err!("Empty line in sort"));
                ' '
            };

            if lhs_prefix == rhs_prefix {
                return lhs_index.cmp(rhs_index);
            }

            match (lhs_prefix, rhs_prefix) {
                ('+', '-') => Ordering::Greater,
                ('-', '+') => Ordering::Less,
                (' ', _) => Ordering::Less,
                (_, ' ') => Ordering::Greater,
                _ => {
                    update_err(merge_err!(
                        "Unexpected line prefixes: {}, {}",
                        lhs_prefix,
                        rhs_prefix
                    ));
                    Ordering::Equal
                }
            }
        },
    );

    if let Some(e) = err {
        return Err(e);
    }
    Ok(lines)
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for MergeIter<'a, T> {
    type Item = Result<String, MergeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let some_ok = |s: String| -> Option<Self::Item> { Some(Ok(s)) };

        let some_ok_string =
            |s: &str| -> Option<Self::Item> { some_ok(s.to_string()) };

        loop {
            match (self.lhs.peek(), self.rhs.peek()) {
                (None, None) => {
                    return None;
                }
                (None, Some(_)) => {
                    return some_ok_string(self.rhs.next()?.line);
                }
                (Some(_), None) => {
                    return some_ok_string(self.lhs.next()?.line);
                }
                (Some(lhs), Some(rhs)) => {
                    if !self.synced {
                        match lhs.cmp(&rhs) {
                            Ordering::Less => {
                                return some_ok_string(self.lhs.next()?.line);
                            }
                            Ordering::Greater => {
                                return some_ok_string(self.rhs.next()?.line);
                            }
                            _ => {
                                self.synced = true;
                            }
                        }
                    }
                    match [lhs.prefix(), rhs.prefix()] {
                        ['+', '+'] => {
                            return some_ok_string(self.rhs.next()?.line);
                        }
                        ['-', '-'] => {
                            return some_ok_string(self.lhs.next()?.line);
                        }
                        ['-', ' '] => {
                            return some_ok_string(self.lhs.next()?.line);
                        }
                        [' ', _] => {
                            if lhs.line[1..] == rhs.line[1..] {
                                self.lhs.next();
                                return some_ok_string(self.rhs.next()?.line);
                            } else {
                                return some_ok_string(self.rhs.next()?.line);
                            }
                        }
                        ['-', '+'] => {
                            if lhs.line[1..] == rhs.line[1..] {
                                self.lhs.next();
                                return some_ok(format!(
                                    " {}",
                                    &self.rhs.next()?.line[1..],
                                ));
                            } else {
                                return some_ok_string(self.lhs.next()?.line);
                            }
                        }
                        ['+', ' '] => {
                            if lhs.line[1..] == rhs.line[1..] {
                                self.rhs.next();
                                return some_ok_string(self.lhs.next()?.line);
                            } else {
                                return some_ok_string(self.rhs.next()?.line);
                            }
                        }
                        ['+', '-'] => {
                            if lhs.line[1..] == rhs.line[1..] {
                                self.lhs.next();
                                self.rhs.next();
                            } else {
                                return some_ok_string(self.rhs.next()?.line);
                            }
                        }
                        [lp, rp] => {
                            return Some(Err(merge_err!(
                                "Unexpected prefix combination [{lp}, {rp}]"
                            )));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::merge_iter::{process, MergeIter};

    struct Case<'a> {
        lines: (Vec<&'a str>, Vec<&'a str>),
        headers: ([usize; 4], [usize; 4]),
        expected: ([usize; 4], Vec<&'a str>),
    }

    impl<'a> Case<'a> {
        fn merge_iter(&'a self) -> MergeIter<'a, std::vec::IntoIter<&str>> {
            MergeIter::new(
                self.headers,
                (
                    self.lines.0.clone().into_iter(),
                    self.lines.1.clone().into_iter(),
                ),
            )
        }

        pub fn run(&'a self) {
            match process(self.merge_iter()) {
                Ok((header, lines)) => {
                    assert_eq!(header, self.expected.0);
                    assert_eq!(lines, self.expected.1);
                }
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn case_1() {
        Case {
            headers: ([1, 6, 1, 8], [1, 8, 1, 8]),
            lines: (
                vec![
                    "+1", "+2", " a", "-b", " c", "-d", "+D", " e", " f", "+3",
                ],
                vec![
                    "+0", " 1", "-2", "-a", "+A", " c", "-D", " e", " f",
                    "+2", " 3",
                ],
            ),
            expected: (
                [1, 6, 1, 8],
                vec![
                    "-a", "-b", "+0", "+1", "+A", " c", "-d", " e", " f",
                    "+2", "+3",
                ],
            ),
        }
        .run()
    }

    #[test]
    fn case_2() {
        Case {
            headers: ([1, 3, 1, 1], [1, 1, 1, 2]),
            lines: (vec!["-1", "-2", " a"], vec!["+1", " a"]),
            expected: ([1, 3, 1, 2], vec![" 1", "-2", " a"]),
        }
        .run()
    }

    #[test]
    fn case_3() {
        Case {
            headers: ([5, 3, 5, 1], [3, 3, 3, 1]),
            lines: (vec!["-5", "-6", " 7"], vec!["-3", "-4", " 7"]),
            expected: ([3, 5, 3, 1], vec!["-3", "-4", "-5", "-6", " 7"]),
        }
        .run()
    }

    #[test]
    fn case_4() {
        Case {
            headers: ([3, 3, 3, 1], [1, 3, 1, 1]),
            lines: (vec!["-3", "-4", " 5"], vec!["-1", "-2", " 5"]),
            expected: ([1, 5, 1, 1], vec!["-1", "-2", "-3", "-4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_5() {
        Case {
            headers: ([5, 1, 5, 3], [5, 1, 3, 3]),
            lines: (vec!["+5", "+6", " 7"], vec!["+3", "+4", " 5"]),
            expected: ([5, 1, 3, 5], vec!["+3", "+4", "+5", "+6", " 7"]),
        }
        .run()
    }

    #[test]
    fn case_6() {
        Case {
            headers: ([1, 1, 1, 3], [3, 1, 3, 3]),
            lines: (vec!["+1", "+2", " 5"], vec!["+3", "+4", " 5"]),
            expected: ([1, 1, 1, 5], vec!["+1", "+2", "+3", "+4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_7() {
        Case {
            headers: ([2, 3, 2, 3], [3, 3, 3, 3]),
            lines: (vec![" 2", " 3", " 4"], vec![" 3", " 4", " 5"]),
            expected: ([2, 4, 2, 4], vec![" 2", " 3", " 4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_8() {
        Case {
            headers: ([1, 3, 1, 3], [3, 1, 3, 3]),
            lines: (vec![" 1", " 2", " 5"], vec!["+3", "+4", " 5"]),
            expected: ([1, 3, 1, 5], vec![" 1", " 2", "+3", "+4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_9() {
        Case {
            headers: ([1, 0, 1, 1], [1, 1, 1, 0]),
            lines: (vec!["+1"], vec!["-1"]),
            expected: ([1, 0, 1, 0], vec![]),
        }
        .run()
    }
}
