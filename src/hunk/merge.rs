use crate::error::MergeError;
use crate::hunk::info_iter::{Info, InfoIter};
use crate::macros::merge_err;
use std::iter::Peekable;
use std::slice::Iter;

struct Merge<'a> {
    lines: Lines,
    lhs: Peekable<InfoIter<'a>>,
    rhs: Peekable<InfoIter<'a>>,
}

struct Lines {
    lines: Vec<String>,
}

impl Lines {
    fn take(&mut self, after: &Option<Info>, before: &Option<Info>) {}

    fn take_data(&mut self, data: Data) {
        println!("data -- {:?}", data);
    }
}

#[derive(Debug)]
enum Data<'a> {
    AddOrRemove(&'a Info<'a>),
    Change((&'a Info<'a>, &'a Info<'a>)),
}

enum AdvanceIter {
    Left(),
    Right(),
    Both(),
}

impl Merge<'_> {
    pub fn new<'a>(lhs: InfoIter<'a>, rhs: InfoIter<'a>) -> Merge<'a> {
        Merge {
            lines: Lines { lines: Vec::new() },
            lhs: lhs.peekable(),
            rhs: rhs.peekable(),
        }
    }

    fn assert_match(lhs: &Info, rhs: &Info) -> Result<(), MergeError> {
        if lhs.line[1..] == rhs.line[1..] {
            Ok(())
        } else {
            Err(merge_err!(
                "Expected a match between {:?} and {:?}",
                lhs,
                rhs
            ))
        }
    }

    fn choose<'a>(
        lafter: &'a Option<Info>,
        lbefore: &'a Option<Info>,
        rafter: &'a Option<Info>,
        rbefore: &'a Option<Info>,
    ) -> Result<(AdvanceIter, Option<Data<'a>>), MergeError> {
        // println!(
        //     "{:?} -- {:?} -- {:?} -- {:?}",
        //     lafter, lbefore, rafter, rbefore
        // );

        match (lafter, lbefore, rafter, rbefore) {
            (Some(la), Some(lb), None, None) => {
                // right empty
                Ok((AdvanceIter::Left(), Some(Data::Change::<'a>((la, lb)))))
            }
            (None, None, Some(ra), Some(rb)) => {
                // left empty
                Ok((AdvanceIter::Right(), Some(Data::Change::<'a>((ra, rb)))))
            }
            (Some(_), None, Some(ra), None) => {
                // both added a line
                // no rb, so this can only be outside of l-r overlap
                Ok((AdvanceIter::Right(), Some(Data::AddOrRemove::<'a>(ra))))
            }
            (Some(_), None, None, Some(_)) => {
                // left added, right removed
                Ok((AdvanceIter::Both(), None))
            }
            (None, Some(lb), Some(ra), None) => {
                /**
                 * left removed, right added, e.g.:
                 *  a            +0
                 * -b <--         a
                 *               +c <--
                 */
                if lb.line[1..] == ra.line[1..] {
                    Ok((AdvanceIter::Both(), None))
                } else {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::Change::<'a>((ra, lb))),
                    ))
                }
            }
            (None, Some(_), None, Some(rb)) => {
                // both removed a line
                // no la, so this can only be outside of l-r overlap
                Ok((AdvanceIter::Right(), Some(Data::AddOrRemove::<'a>(rb))))
            }
            (None, Some(lb), Some(ra), Some(rb)) => {
                // left removed, right changed
                Ok((AdvanceIter::Left(), Some(Data::AddOrRemove::<'a>(lb))))
            }
            (Some(la), None, Some(ra), Some(rb)) => {
                // left added, right changed
                if ra.line.starts_with('+') {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::AddOrRemove::<'a>(ra)),
                    ))
                } else {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::AddOrRemove::<'a>(la)),
                    ))
                }
            }
            (Some(la), Some(lb), Some(ra), None) => {
                // left changed, right added
                Ok((AdvanceIter::Left(), Some(Data::Change::<'a>((la, lb)))))
            }
            (Some(la), Some(lb), None, Some(rb)) => {
                // left changed, right removed
                if lb.line.starts_with('-') {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::AddOrRemove::<'a>(lb)),
                    ))
                } else {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::AddOrRemove::<'a>(rb)),
                    ))
                }
            }
            (Some(la), Some(lb), Some(ra), Some(rb)) => {
                // both changed
                if lb.line.starts_with('-') && ra.line.starts_with('+') {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::Change::<'a>((ra, lb))),
                    ))
                } else if lb.line.starts_with('-') {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::Change::<'a>((la, lb))),
                    ))
                } else {
                    Ok((
                        AdvanceIter::Both(),
                        Some(Data::Change::<'a>((ra, rb))),
                    ))
                }
            }
            unexpected => Err(merge_err!("Unexpected case: {:?}", unexpected)),
        }
    }

    pub fn call<'a>(&'a mut self) -> Result<(), MergeError> {
        let nil: (Option<Info>, Option<Info>) = (None, None);
        while self.lhs.peek().is_some() || self.rhs.peek().is_some() {
            let (la, lb) = self.lhs.peek().unwrap_or(&nil);
            let (ra, rb) = self.rhs.peek().unwrap_or(&nil);

            // fun times with:
            // cannot borrow `*self` as mutable more than once at a time
            if let (Some(left), Some(right)) = (la, rb) {
                if left.num < right.num {
                    self.lines.take(la, lb);
                    self.lhs.next();
                    continue;
                }
                if left.num > right.num {
                    self.lines.take(ra, rb);
                    self.rhs.next();
                    continue;
                }
                Self::assert_match(left, right)?;
            }

            let (action, data) = Self::choose(la, lb, ra, rb)?;
            if let Some(d) = data {
                self.lines.take_data(d);
            }
            match action {
                AdvanceIter::Left() => {
                    self.lhs.next();
                }
                AdvanceIter::Right() => {
                    self.rhs.next();
                }
                AdvanceIter::Both() => {
                    self.lhs.next();
                    self.rhs.next();
                }
            }
        }

        // let mut lhs = self.lhs_iter.next();
        // let mut rhs = self.rhs_iter.next();

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::hunk::info_iter::InfoIter;
    use crate::hunk::merge::Merge;

    #[test]
    fn test_merge() {
        let hleft: [usize; 4] = [1, 6, 1, 8];
        let hright: [usize; 4] = [1, 8, 1, 8];

        let left: Vec<_> =
            vec!["+1", "+2", " a", "-b", " c", "-d", "+D", " e", " f", "+3"]
                .iter()
                .map(|s| s.to_string())
                .collect();

        // for info in InfoIter::new(&hleft, left.iter()) {
        //     println!("{:?}", info);
        // }

        let right: Vec<_> = vec![
            "+0", " 1", "-2", "-a", "+A", " c", "-D", " e", " f", "+2", " 3",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        // for info in InfoIter::new(&hright, right.iter()) {
        //     println!("{:?}", info);
        // }

        let expected: Vec<_> = vec![
            "-a", "-b", "+0", "+1", "+A", " c", "-d", " e", " f", "+2", "+3",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let mut merge = Merge::new(
            InfoIter::new(&hleft, left.iter()),
            InfoIter::new(&hright, right.iter()),
        );

        merge.call();
    }
}
