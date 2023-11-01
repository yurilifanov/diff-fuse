use crate::error::MergeError;
use crate::hunk::info_iter::{Info, InfoIter};
use crate::macros::merge_err;
use std::iter::Peekable;

struct MergeIter<'a, T: Iterator<Item = &'a str> + Clone> {
    lhs: Peekable<InfoIter<'a, T>>,
    rhs: Peekable<InfoIter<'a, T>>,
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

            match (lafter, lbefore, rafter, rbefore) {
                // la, lb, ra, rb
                (Some(_), Some(_), None, None) => {
                    // rhs empty
                    return Some(MergeItem::from(self.lhs.next()));
                }
                (None, None, Some(_), Some(_)) => {
                    // lhs empty
                    return Some(MergeItem::from(self.rhs.next()));
                }
                (Some(_), None, Some(_), None) => {
                    // both added a line
                    // no rb, so this can only be outside of lhs-rhs overlap
                    return Some(MergeItem::from(self.rhs.next()));
                }
                (Some(_), None, None, Some(_)) => {
                    // lhs added - rhs removed, so skip
                    self.lhs.next();
                    self.rhs.next();
                }
                (None, Some(lb), Some(ra), None) => {
                    // lhs removed, rhs added
                    if lb.line[1..] == ra.line[1..] {
                        // change has been reverted, so skip
                        self.lhs.next();
                        self.rhs.next();
                    } else {
                        // keep both
                        return Some(MergeItem::from((
                            self.lhs.next(),
                            self.rhs.next(),
                        )));
                    }
                }
                (None, Some(_), None, Some(_)) => {
                    // both removed a line
                    // no la, so this can only be outside of lhs-rhs overlap
                    return Some(MergeItem::from(self.rhs.next()));
                }
                (None, Some(_), Some(_), Some(_)) => {
                    // lhs removed, rhs maybe changed
                    return Some(MergeItem::from(self.lhs.next()));
                }
                (Some(_), None, Some(ra), Some(_)) => {
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
                (Some(_), Some(_), Some(_), None) => {
                    // lhs changed, rhs added
                    return Some(MergeItem::from(self.lhs.next()));
                }
                (Some(_), Some(lb), None, Some(_)) => {
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
                (Some(_), Some(lb), Some(ra), Some(_)) => {
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
                unexpected => {
                    return Some(MergeItem::Err(merge_err!(
                        "Unexpected case: {:?}",
                        unexpected
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
    use crate::hunk::merge_iter::{MergeItem, MergeIter};

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

        pub fn merge_iter(
            &'a self,
        ) -> MergeIter<'a, std::vec::IntoIter<&str>> {
            MergeIter {
                lhs: self.info_iter(0).peekable(),
                rhs: self.info_iter(1).peekable(),
            }
        }
    }

    #[test]
    fn test_merge() {
        let case = Case {
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
        };

        for item in case.merge_iter() {
            println!("{:?}", item);
            if let MergeItem::Err(_) = item {
                println!("Error");
                break;
            }
        }
    }
}
