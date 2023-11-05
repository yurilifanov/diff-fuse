use crate::error::MergeError;
use crate::hunk::info_iter::{Info, InfoIter};
use crate::macros::merge_err;
use core::cmp::Ordering;
use std::iter::Peekable;

struct MergeIter<'a, T: Iterator<Item = &'a str> + Clone> {
    lhs: Peekable<InfoIter<'a, T>>,
    rhs: Peekable<InfoIter<'a, T>>,
}

fn with_prefix(string: &str, prefix: &str) -> String {
    let mut result = string[1..].to_string();
    result.insert_str(0, prefix);
    result
}

fn process<'a, T: Iterator<Item = &'a str> + Clone>(
    iter: MergeIter<'a, T>,
) -> Result<([usize; 4], Vec<String>), MergeError> {
    // (group, index), line
    let mut lines: Vec<((usize, usize), String)> = Vec::new();

    let mut counters: (usize, usize, usize) = (0, 0, 0);
    let mut update_counters = |s: &str| {
        if s.starts_with('-') {
            counters.1 += 1;
            (counters.0, counters.1)
        } else if s.starts_with('+') {
            counters.2 += 1;
            (counters.0, counters.2)
        } else {
            counters = (counters.0 + 1, 0, 0);
            (counters.0, 1)
        }
    };

    for item in iter {
        println!("{:?}", item);
        match item {
            MergeItem::Single(line) => {
                lines.push((update_counters(line), line.to_string()));
            }
            MergeItem::Pair((b, a)) => {
                if let Some((_, line)) = lines.last() {
                    if line[1..] == a[1..] {
                        lines.pop();
                        lines
                            .push((update_counters(" "), with_prefix(a, " ")));
                    } else {
                        lines
                            .push((update_counters("+"), with_prefix(a, "+")));
                    }
                } else {
                    lines.push((update_counters("+"), with_prefix(a, "+")));
                }
                lines.push((update_counters("-"), with_prefix(b, "-")));
            }
            MergeItem::Err(err) => {
                return Err(err);
            }
        }
    }

    Ok((
        [0, 0, 0, 0],
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

#[derive(Debug)]
enum MergeItem<'a> {
    Single(&'a str),
    Pair((&'a str, &'a str)),
    Err(MergeError),
}

impl<'a> From<Info<'a>> for MergeItem<'a> {
    fn from(value: Info<'a>) -> MergeItem<'a> {
        match value {
            (Some(b), Some(a), _) => MergeItem::Pair((b, a)),
            (None, Some(a), _) => MergeItem::Single(a),
            (Some(b), None, _) => MergeItem::Single(b),
            _ => MergeItem::Err(merge_err!(
                "Cannot build a MergeItem from (None, None)"
            )),
        }
    }
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for MergeIter<'a, T> {
    type Item = MergeItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        println!("{:?}", (self.lhs.peek(), self.rhs.peek()));

        match (self.lhs.peek(), self.rhs.peek()) {
            (None, None) => None,
            (None, Some(rhs)) => Some(MergeItem::from(self.rhs.next()?)),
            (Some(lhs), None) => Some(MergeItem::from(self.lhs.next()?)),
            (Some((lbefore, lafter, ln)), Some((rbefore, rafter, rn))) => {
                if ln < rn {
                    return Some(MergeItem::from(self.lhs.next()?));
                }
                if ln > rn {
                    return Some(MergeItem::from(self.rhs.next()?));
                }
                if let (Some(lhs), Some(rhs)) = (lafter, rbefore) {
                    if lhs[1..] != rhs[1..] {
                        return Some(MergeItem::Err(merge_err!(
                            "Conflict at ({}, {})",
                            lhs,
                            rhs
                        )));
                    }
                }
                match (lbefore, lafter, rbefore, rafter) {
                    (Some(lb), _, _, Some(ra)) => {
                        if lb == ra {
                            self.lhs.next();
                            return Some(MergeItem::Single(
                                self.rhs.next()?.1?,
                            ));
                        }
                        Some(MergeItem::Pair((
                            self.lhs.next()?.0?,
                            self.rhs.next()?.1?,
                        )))
                    }
                    (None, Some(_), Some(rb), Some(ra)) => {
                        if rb == ra {
                            self.rhs.next();
                            return Some(MergeItem::Single(
                                self.lhs.next()?.1?,
                            ));
                        }
                        self.lhs.next();
                        Some(MergeItem::Single(self.rhs.next()?.1?))
                    }
                    (Some(_), Some(_), _, None) => {
                        self.rhs.next();
                        Some(MergeItem::from(self.lhs.next()?))
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::info_iter::InfoIter;
    use crate::hunk::merge_iter::{process, MergeItem, MergeIter};

    struct Case<'a> {
        lines: [Vec<&'a str>; 2],
        headers: [[usize; 4]; 2],
        expected: ([usize; 4], Vec<&'a str>),
    }

    impl<'a> Case<'a> {
        fn info_iter(
            &'a self,
            index: usize,
        ) -> InfoIter<'a, std::vec::IntoIter<&str>> {
            InfoIter::new(
                &self.headers[index],
                self.lines[index].clone().into_iter(),
            )
        }

        fn merge_iter(&'a self) -> MergeIter<'a, std::vec::IntoIter<&str>> {
            MergeIter {
                lhs: self.info_iter(0).peekable(),
                rhs: self.info_iter(1).peekable(),
            }
        }

        pub fn run(&'a self) {
            match process(self.merge_iter()) {
                Ok((header, lines)) => {
                    assert_eq!(lines, self.expected.1);
                }
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn case_1() {
        Case {
            headers: [[1, 6, 1, 8], [1, 8, 1, 8]],
            lines: [
                vec![
                    "+1", "+2", " a", "-b", " c", "-d", "+D", " e", " f", "+3",
                ],
                vec![
                    "+0", " 1", "-2", "-a", "+A", " c", "-D", " e", " f",
                    "+2", " 3",
                ],
            ],
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
            headers: [[1, 3, 1, 1], [1, 1, 1, 2]],
            lines: [vec!["-1", "-2", " a"], vec!["+1", " a"]],
            expected: ([1, 3, 1, 2], vec![" 1", "-2", " a"]),
        }
        .run()
    }

    #[test]
    fn case_3() {
        Case {
            headers: [[5, 3, 5, 1], [3, 3, 3, 1]],
            lines: [vec!["-5", "-6", " 7"], vec!["-3", "-4", " 7"]],
            expected: ([3, 5, 3, 1], vec!["-3", "-4", "-5", "-6", " 7"]),
        }
        .run()
    }

    #[test]
    fn case_4() {
        Case {
            headers: [[3, 3, 3, 1], [1, 3, 1, 1]],
            lines: [vec!["-3", "-4", " 5"], vec!["-1", "-2", " 5"]],
            expected: ([1, 5, 1, 1], vec!["-1", "-2", "-3", "-4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_5() {
        Case {
            headers: [[5, 1, 5, 3], [5, 1, 3, 3]],
            lines: [vec!["+5", "+6", " 7"], vec!["+3", "+4", " 5"]],
            expected: ([5, 1, 3, 5], vec!["+3", "+4", "+5", "+6", " 7"]),
        }
        .run()
    }

    #[test]
    fn case_6() {
        Case {
            headers: [[1, 1, 1, 3], [3, 1, 3, 3]],
            lines: [vec!["+1", "+2", " 5"], vec!["+3", "+4", " 5"]],
            expected: ([1, 1, 1, 5], vec!["+1", "+2", "+3", "+4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_7() {
        Case {
            headers: [[2, 3, 2, 3], [3, 3, 3, 3]],
            lines: [vec![" 2", " 3", " 4"], vec![" 3", " 4", " 5"]],
            expected: ([2, 4, 2, 4], vec![" 2", " 3", " 4", " 5"]),
        }
        .run()
    }

    #[test]
    fn case_8() {
        Case {
            headers: [[1, 3, 1, 3], [3, 1, 3, 3]],
            lines: [vec![" 1", " 2", " 5"], vec!["+3", "+4", " 5"]],
            expected: ([1, 3, 1, 5], vec![" 1", " 2", "+3", "+4", " 5"]),
        }
        .run()
    }
}
