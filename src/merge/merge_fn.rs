use crate::error::MergeError;
use crate::hand::Hand;
use crate::info::{iter_info, Info};
use crate::macros::merge_err;
use core::cmp::{min, Ordering};
use std::iter::{repeat, Peekable};

pub fn merge_fn<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    lheader: &[usize; 4],
    llines: T,
    rheader: &[usize; 4],
    rlines: U,
) -> Result<([usize; 4], Vec<(Hand, String)>), MergeError> {
    let mut header: [usize; 4] = [
        min(lheader[0], rheader[0]),
        0,
        min(lheader[2], rheader[2]),
        0,
    ];

    let mut counters: (usize, usize, usize) = (0, 0, 0);

    // returns (group, index)
    let mut update_counters = |info: &Info| {
        if info.line.starts_with('-') {
            counters.1 += 1;
            header[1] += 1;
            (counters.0, counters.1)
        } else if info.line.starts_with('+') {
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

    let mut data: Vec<((usize, usize), Info)> = Vec::new();
    for item in merge_iter(llines, rlines) {
        match item? {
            MergeItem::Single(info) => {
                data.push((update_counters(&info), info));
            }
            MergeItem::Pair((linfo, rinfo)) => {
                data.push((update_counters(&linfo), linfo));
                data.push((update_counters(&rinfo), rinfo));
            }
            MergeItem::None() => {}
        }
    }

    Ok((
        header,
        sort_data(data)?
            .into_iter()
            .map(|(_, info)| info.data())
            .collect(),
    ))
}

fn sort_data(
    mut data: Vec<((usize, usize), Info)>,
) -> Result<Vec<((usize, usize), Info)>, MergeError> {
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
    data.sort_unstable_by(
        |((lhs_group, lhs_index), (linfo)),
         ((rhs_group, rhs_index), (rinfo))| {
            if lhs_group != rhs_group {
                return lhs_group.cmp(rhs_group);
            }

            let lhs_prefix = if let Some(val) = linfo.line.chars().nth(0) {
                val
            } else {
                update_err(merge_err!("Empty line in sort"));
                ' '
            };

            let rhs_prefix = if let Some(val) = rinfo.line.chars().nth(0) {
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
    Ok(data)
}

enum MergeItem {
    None(),
    Single(Info),
    Pair((Info, Info)),
}

fn merge_iter<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    linfo: T,
    rinfo: U,
) -> impl Iterator<Item = Result<MergeItem, MergeError>> {
    let mut liter = linfo.peekable();
    let mut riter = rinfo.peekable();
    std::iter::from_fn(move || -> Option<Result<MergeItem, MergeError>> {
        println!("{:?} -- {:?}", liter.peek(), riter.peek());
        match [liter.peek(), riter.peek()] {
            [None, None] => None,
            [None, Some(_)] => take(&mut riter),
            [Some(_), None] => take(&mut liter),
            [Some(linfo), Some(rinfo)] => match linfo.hand.cmp(&rinfo.hand) {
                Ordering::Less => match linfo.rank.cmp(&rinfo.rank) {
                    Ordering::Less => take(&mut liter),
                    Ordering::Greater => take(&mut riter),
                    _ => {
                        let index = [linfo.prefix(), rinfo.prefix()];
                        next(index, &mut liter, &mut riter)
                    }
                },
                Ordering::Greater => match rinfo.rank.cmp(&linfo.rank) {
                    Ordering::Less => take(&mut riter),
                    Ordering::Greater => take(&mut liter),
                    _ => {
                        let index = [rinfo.prefix(), linfo.prefix()];
                        next(index, &mut riter, &mut liter)
                    }
                },
                _ => match linfo.rank.cmp(&rinfo.rank) {
                    Ordering::Less => take(&mut liter),
                    Ordering::Greater => take(&mut riter),
                    _ => {
                        let lline = liter.next()?.line;
                        let rline = riter.next()?.line;
                        Some(Err(merge_err!(
                            "Conflict between lines '{}' and '{}'",
                            lline,
                            rline
                        )))
                    }
                },
            },
        }
    })
}

fn next<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    index: [char; 2],
    liter: &mut T,
    riter: &mut U,
) -> Option<Result<MergeItem, MergeError>> {
    match index {
        [' ', ' '] => skip_take(liter, riter),
        ['+', ' '] => skip_take(riter, liter),
        ['-', ' '] => take(liter),

        [' ', '+'] => take(riter),
        ['+', '+'] => take(riter),
        ['-', '+'] => {
            let linfo = liter.next()?;
            let mut rinfo = riter.next()?;
            if linfo.line[1..] == rinfo.line[1..] {
                rinfo.line.replace_range(0..1, " ");
                Some(Ok(MergeItem::Single(rinfo)))
            } else {
                Some(Ok(MergeItem::Pair((linfo, rinfo))))
            }
        }

        [' ', '-'] => skip_take(liter, riter),
        ['+', '-'] => skip(liter, riter),
        ['-', '-'] => take(liter),

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
    Some(Ok(MergeItem::Single(iter.next()?)))
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
            "skip: Mismatch between lines '{lline}' and '{rline}'"
        )))
    }
}

fn skip_take<T: Iterator<Item = Info>, U: Iterator<Item = Info>>(
    lhs: &mut T,
    rhs: &mut U,
) -> Option<Result<MergeItem, MergeError>> {
    let linfo = lhs.next()?;
    let rinfo = rhs.next()?;
    if linfo.line[1..] == rinfo.line[1..] {
        Some(Ok(MergeItem::Single(rinfo)))
    } else {
        Some(Err(merge_err!(
            "skip_take: Mismatch between lines '{}' and '{}'",
            linfo.line,
            rinfo.line
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::Hand;
    use crate::info::iter_info;
    use crate::merge::merge_fn::merge_fn;
    use std::iter::repeat;

    struct Case<'a> {
        lines: (Vec<&'a str>, Vec<&'a str>),
        headers: ([usize; 4], [usize; 4]),
        expected: ([usize; 4], Vec<&'a str>),
    }

    impl<'a> Case<'a> {
        pub fn run(&'a mut self) {
            let result = merge_fn(
                &self.headers.0,
                iter_info(
                    &self.headers.0,
                    repeat(Hand::Left).zip(
                        self.lines
                            .0
                            .clone()
                            .into_iter()
                            .map(|s: &str| s.to_string()),
                    ),
                ),
                &self.headers.1,
                iter_info(
                    &self.headers.1,
                    repeat(Hand::Right).zip(
                        self.lines
                            .1
                            .clone()
                            .into_iter()
                            .map(|s: &str| s.to_string()),
                    ),
                ),
            );

            match result {
                Ok((header, data)) => {
                    let lines: Vec<_> =
                        data.into_iter().map(|(_, s)| s).collect();
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
