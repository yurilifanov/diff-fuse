use crate::error::MergeError;
use crate::hunk::info_iter::{Info, InfoIter};
use crate::macros::merge_err;
use core::cmp::Ordering;
use std::iter::Peekable;

struct MergeIter<'a, T: Iterator<Item = &'a str> + Clone> {
    lhs: Peekable<InfoIter<'a, T>>,
    rhs: Peekable<InfoIter<'a, T>>,
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
        match item {
            MergeItem::Single(it) => {
                lines.push((update_counters(it.line), it.line.to_string()));
            }
            MergeItem::Pair((a, b)) => {
                if a.line == b.line {
                    lines.push((update_counters(a.line), a.line.to_string()));
                } else if a.line[1..] == b.line[1..] {
                    let mut line = a.line[1..].to_string();
                    line.insert_str(0, " ");
                    lines.push((update_counters(" "), line));
                } else {
                    lines.push((update_counters(a.line), a.line.to_string()));
                    lines.push((update_counters(b.line), b.line.to_string()));
                }
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
    Single(Info<'a>),
    Pair((Info<'a>, Info<'a>)),
    Err(MergeError),
}

type InfoPack<'a> = (Option<Info<'a>>, Option<Info<'a>>);

impl<'a> From<InfoPack<'a>> for MergeItem<'a> {
    fn from(value: InfoPack<'a>) -> MergeItem<'a> {
        match value {
            (None, None) => MergeItem::Err(merge_err!(
                "Cannot build a MergeItem from (None, None)"
            )),
            (None, Some(before)) => MergeItem::Single(before),
            (Some(after), None) => MergeItem::Single(after),
            (Some(after), Some(before)) => MergeItem::Pair((after, before)),
        }
    }
}

type MaybeInfoPack<'a> = Option<InfoPack<'a>>;

impl<'a> From<MaybeInfoPack<'a>> for MergeItem<'a> {
    fn from(value: MaybeInfoPack<'a>) -> MergeItem<'a> {
        if let Some(pack) = value {
            MergeItem::from(pack)
        } else {
            MergeItem::Err(merge_err!("Cannot build a MergeItem from (None)"))
        }
    }
}

impl<'a> From<(MaybeInfoPack<'a>, MaybeInfoPack<'a>)> for MergeItem<'a> {
    fn from(value: (MaybeInfoPack<'a>, MaybeInfoPack<'a>)) -> MergeItem<'a> {
        if let (Some(lhs), Some(rhs)) = value {
            match (lhs, rhs) {
                ((_, Some(lb)), (Some(ra), _)) => MergeItem::Pair((ra, lb)),
                unsupported => MergeItem::Err(merge_err!(
                    "Unsupported case: {:?}",
                    unsupported
                )),
            }
        } else {
            MergeItem::Err(merge_err!(
                "Cannot build a MergeItem from (None, None)"
            ))
        }
    }
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for MergeIter<'a, T> {
    type Item = MergeItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let nil: (Option<Info>, Option<Info>) = (None, None);

        while self.lhs.peek().is_some() || self.rhs.peek().is_some() {
            let (lafter, lbefore) = self.lhs.peek().unwrap_or(&nil);
            let (rafter, rbefore) = self.rhs.peek().unwrap_or(&nil);

            // println!("{:?}", (lafter, lbefore, rafter, rbefore));

            // if can match the line numbers,
            // bring lhs and rhs to the point where the hunks overlap
            if let (Some(lhs), Some(rhs)) = (lafter, rbefore) {
                if lhs.num < rhs.num {
                    return Some(MergeItem::from(self.lhs.next()));
                }
                if lhs.num > rhs.num {
                    return Some(MergeItem::from(self.rhs.next()));
                }
                if lhs.line[1..] != rhs.line[1..] {
                    return Some(MergeItem::Err(merge_err!(
                        "Mismatch at {:?} and {:?}",
                        lhs,
                        rhs
                    )));
                }
            }

            // 4 ^ 2 = 16 cases
            match [lafter, lbefore, rafter, rbefore] {
                // la, lb, ra, rb
                [Some(_), None, None, None] => {
                    return Some(MergeItem::from(self.lhs.next()));
                }
                [None, Some(_), None, None] => {
                    return Some(MergeItem::from(self.lhs.next()));
                }
                [None, None, Some(_), None] => {
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [None, None, None, Some(_)] => {
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [Some(_), Some(_), None, None] => {
                    // rhs empty
                    return Some(MergeItem::from(self.lhs.next()));
                }
                [None, None, Some(_), Some(_)] => {
                    // lhs empty
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [Some(la), None, Some(ra), None] => {
                    // both added a line
                    // no rb, so this can only be outside of lhs-rhs overlap
                    if la.num < ra.num {
                        return Some(MergeItem::from(self.lhs.next()));
                    }
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [Some(_), None, None, Some(_)] => {
                    // lhs added - rhs removed, so skip
                    self.lhs.next();
                    self.rhs.next();
                }
                [None, Some(lb), Some(ra), None] => {
                    // lhs removed, rhs added
                    // change may have been reverted
                    return Some(MergeItem::from((
                        self.lhs.next(),
                        self.rhs.next(),
                    )));
                }
                [None, Some(_), None, Some(_)] => {
                    // both removed a line
                    // no la, so this can only be outside of lhs-rhs overlap
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [None, Some(_), Some(_), Some(_)] => {
                    // lhs removed, rhs maybe changed
                    return Some(MergeItem::from(self.lhs.next()));
                }
                [Some(_), None, Some(ra), Some(_)] => {
                    // lhs added, rhs maybe changed
                    if ra.line.starts_with('+') {
                        self.lhs.next();
                        if let Some((Some(after), Some(_))) = self.rhs.next() {
                            return Some(MergeItem::Single(after));
                        }
                    } else {
                        self.rhs.next();
                        if let Some((Some(after), _)) = self.lhs.next() {
                            return Some(MergeItem::Single(after));
                        }
                    }
                }
                [Some(la), Some(_), Some(ra), None] => {
                    // lhs maybe changed, rhs added
                    if la.num < ra.num {
                        return Some(MergeItem::from(self.lhs.next()));
                    }
                    return Some(MergeItem::from(self.rhs.next()));
                }
                [Some(_), Some(lb), None, Some(_)] => {
                    // lhs maybe changed, rhs removed
                    if lb.line.starts_with('-') {
                        self.rhs.next();
                        if let Some((Some(_), Some(before))) = self.lhs.next()
                        {
                            return Some(MergeItem::Single(before));
                        }
                    } else {
                        self.lhs.next();
                        if let Some((_, Some(before))) = self.rhs.next() {
                            return Some(MergeItem::Single(before));
                        }
                    }
                }
                [Some(_), Some(lb), Some(ra), Some(_)] => {
                    // both maybe changed
                    if lb.line.starts_with('-') && ra.line.starts_with('+') {
                        return Some(MergeItem::from((
                            self.lhs.next(),
                            self.rhs.next(),
                        )));
                    } else if lb.line.starts_with('-') {
                        self.rhs.next();
                        return Some(MergeItem::from(self.lhs.next()));
                    } else {
                        self.lhs.next();
                        return Some(MergeItem::from(self.rhs.next()));
                    }
                }
                array => {
                    return Some(MergeItem::Err(merge_err!(
                        "Unexpected case: {:?}",
                        array
                    )));
                }
            }
        }
        None
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
    fn case_9() {
        Case {
            headers: [[1, 3, 1, 3], [3, 1, 3, 3]],
            lines: [vec![" 1", " 2", " 5"], vec!["+3", "+4", " 5"]],
            expected: ([1, 3, 1, 5], vec![" 1", " 2", "+3", "+4", " 5"]),
        }
        .run()
    }
}
