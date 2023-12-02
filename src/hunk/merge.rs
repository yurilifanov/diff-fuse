use crate::error::MergeError;
use crate::hunk::iter_info::{iter_info, Info, InfoType};
use crate::macros::merge_err;
use core::cmp::{min, Ordering};
use std::iter::Peekable;

pub fn merge<T: Iterator<Item = String>, U: Iterator<Item = String>>(
    lheader: &[usize; 4],
    llines: T,
    rheader: &[usize; 4],
    rlines: U,
) -> Result<([usize; 4], Vec<String>), MergeError> {
    let mut header: [usize; 4] = [
        min(lheader[0], rheader[0]),
        0,
        min(lheader[2], rheader[2]),
        0,
    ];

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
    for item in merge_iter(lheader, llines, rheader, rlines) {
        match item? {
            MergeItem::Single(line) => {
                lines.push((update_counters(&line), line));
            }
            MergeItem::Pair((lline, rline)) => {
                lines.push((update_counters(&lline), lline));
                lines.push((update_counters(&rline), rline));
            }
            MergeItem::None() => {}
        }
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

enum MergeItem {
    None(),
    Single(String),
    Pair((String, String)),
}

fn merge_iter<T: Iterator<Item = String>, U: Iterator<Item = String>>(
    lheader: &[usize; 4],
    llines: T,
    rheader: &[usize; 4],
    rlines: U,
) -> impl Iterator<Item = Result<MergeItem, MergeError>> {
    let mut liter = iter_info(lheader, llines, InfoType::Minus()).peekable();
    let mut riter = iter_info(rheader, rlines, InfoType::Plus()).peekable();
    std::iter::from_fn(move || -> Option<Result<MergeItem, MergeError>> {
        // next(&mut liter, &mut riter)
        match [liter.peek(), riter.peek()] {
            [None, None] => None,
            [None, Some(_)] => take(&mut riter),
            [Some(_), None] => take(&mut liter),

            // what's on the right and left should be determined based on
            // info precedence
            [Some(linfo), Some(rinfo)] => {
                let index = (
                    linfo.prefix(),
                    rinfo.prefix(),
                    linfo.rank.cmp(&rinfo.rank),
                );
                match linfo.cmp(&rinfo) {
                    Ordering::Less => next(index, &mut liter, &mut riter),
                    Ordering::Greater => next(index, &mut riter, &mut liter),

                    // FIXME: more elaborate error
                    _ => Some(Err(merge_err!("Merge conflict detected"))),
                }
            }
        }
    })
}

fn next<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    index: (char, char, Ordering),
    liter: &mut T,
    riter: &mut U,
) -> Option<Result<MergeItem, MergeError>> {
    match index {
        ('+', '+', Ordering::Less) => take(liter),
        ('+', '+', _) => take(riter),

        (' ', '+', Ordering::Less) => take(liter),
        (' ', '+', _) => take(riter),

        ('-', ' ', Ordering::Greater) => take(riter),
        ('-', ' ', _) => take(liter),

        ('-', '-', Ordering::Greater) => take(riter),
        ('-', '-', _) => take(liter),

        ('-', '+', Ordering::Less) => take(liter),
        ('-', '+', Ordering::Greater) => take(riter),
        ('-', '+', _) => {
            let lline = liter.next()?.line;
            let mut rline = riter.next()?.line;
            if lline[1..] == rline[1..] {
                rline.replace_range(0..1, " ");
                Some(Ok(MergeItem::Single(rline)))
            } else {
                Some(Ok(MergeItem::Pair((lline, rline))))
            }
        }

        ('+', '-', Ordering::Less) => take(liter),
        ('+', '-', Ordering::Greater) => take(riter),
        ('+', '-', _) => skip(liter, riter),

        ('+', ' ', Ordering::Less) => take(liter),
        ('+', ' ', Ordering::Greater) => take(riter),
        ('+', ' ', _) => skip_take(riter, liter),

        (' ', ' ', Ordering::Less) => take(liter),
        (' ', ' ', Ordering::Greater) => take(riter),
        (' ', ' ', _) => skip_take(liter, riter),

        (' ', '-', Ordering::Less) => take(liter),
        (' ', '-', Ordering::Greater) => take(liter),
        (' ', '-', _) => skip_take(liter, riter),

        _ => {
            let lline = liter.next()?.line;
            let rline = riter.next()?.line;
            Some(Err(merge_err!(
                "Unexpected prefixes on lines '{lline}' and '{rline}'"
            )))
        }
    }
}

fn take<T: Iterator<Item = Info>>(
    iter: &mut T,
) -> Option<Result<MergeItem, MergeError>> {
    Some(Ok(MergeItem::Single(iter.next()?.line)))
}

fn skip<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    lhs: &mut T,
    rhs: &mut U,
) -> Option<Result<MergeItem, MergeError>> {
    let lline = lhs.next()?.line;
    let rline = rhs.next()?.line;
    if lline[1..] == rline[1..] {
        Some(Ok(MergeItem::None()))
    } else {
        Some(Err(merge_err!(
            "Mismatch between lines '{lline}' and '{rline}'"
        )))
    }
}

fn skip_take<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    lhs: &mut T,
    rhs: &mut U,
) -> Option<Result<MergeItem, MergeError>> {
    let lline = lhs.next()?.line;
    let rline = rhs.next()?.line;
    if lline[1..] != rline[1..] {
        Some(Err(merge_err!(
            "Mismatch between lines '{lline}' and '{rline}'"
        )))
    } else {
        Some(Ok(MergeItem::Single(rline)))
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::merge::merge;

    struct Case<'a> {
        lines: (Vec<&'a str>, Vec<&'a str>),
        headers: ([usize; 4], [usize; 4]),
        expected: ([usize; 4], Vec<&'a str>),
    }

    impl<'a> Case<'a> {
        pub fn run(&'a mut self) {
            let result = merge(
                &self.headers.0,
                self.lines
                    .0
                    .clone()
                    .into_iter()
                    .map(|s: &str| s.to_string()),
                &self.headers.1,
                self.lines
                    .1
                    .clone()
                    .into_iter()
                    .map(|s: &str| s.to_string()),
            );

            match result {
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
