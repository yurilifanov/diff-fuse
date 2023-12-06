use crate::error::MergeError;
use crate::hand::Hand;
use crate::hunk::handed::{HandedHunk, Mergeable};
use crate::hunk::{header, Hunk};
use crate::info::Info;
use crate::macros::debugln;
use core::cmp::Ordering;
use std::iter::Peekable;

pub fn merge_iter<T: Iterator<Item = HandedHunk>>(
    mut iter: Peekable<T>,
) -> impl Iterator<Item = Result<Hunk, MergeError>> {
    std::iter::from_fn(move || -> Option<Result<Hunk, MergeError>> {
        let next = iter.next()?;
        if let Some(peek) = iter.peek() {
            if !next.overlaps(peek) {
                return Some(Ok(next.into()));
            }

            debugln!("Merging hunks {next} and {peek}");
            let mut merge = match next.merge(iter.next()?) {
                Ok(m) => m,
                Err(err) => {
                    return Some(Err(err));
                }
            };

            while let Some(peek) = iter.peek() {
                if !peek.overlaps(&merge) {
                    return Some(Ok(merge.into()));
                }

                debugln!("Merging hunks {merge} (merged) and {peek}");
                println!("{merge:?}");
                merge = match iter.next()?.merge(merge) {
                    Ok(m) => m,
                    Err(err) => {
                        return Some(Err(err));
                    }
                };
            }

            return Some(Ok(merge.into()));
        }
        Some(Ok(next.into()))
    })
}

pub fn info_chain<'a, T: Iterator<Item = Hunk>, U: Iterator<Item = Hunk>>(
    mut liter: &'a mut Peekable<T>,
    mut riter: &'a mut Peekable<U>,
) -> Option<impl Iterator<Item = (Option<Info>, Option<Info>)> + 'a> {
    let (mut lheader, mut linfo) = liter.next()?.into_info(Hand::Left);
    let (mut rheader, mut rinfo) = riter.next()?.into_info(Hand::Right);

    // TODO: adjust lheader

    Some(std::iter::from_fn(
        move || -> Option<(Option<Info>, Option<Info>)> {
            let mut li = linfo.next();
            while li.is_none() {
                if let Some(hunk) = liter.peek() {
                    if header::overlap(hunk.header(), &rheader) {
                        // TODO: adjust lheader

                        (lheader, linfo) = liter.next()?.into_info(Hand::Left);
                        li = linfo.next();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let mut ri = rinfo.next();
            while ri.is_none() {
                if let Some(hunk) = riter.peek() {
                    if header::overlap(&rheader, hunk.header()) {
                        (rheader, rinfo) =
                            riter.next()?.into_info(Hand::Right);
                        ri = rinfo.next();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if li.is_none() && ri.is_none() {
                None
            } else {
                Some((li, ri))
            }
        },
    ))
}

pub fn into_info_chains<
    'a,
    T: Iterator<Item = Hunk>,
    U: Iterator<Item = Hunk>,
>(
    mut liter: &'a mut Peekable<T>,
    mut riter: &'a mut Peekable<U>,
) -> Option<(
    impl Iterator<Item = Info> + 'a,
    impl Iterator<Item = Info> + 'a,
)> {
    let (mut lheader, mut linfo) = liter.next()?.into_info(Hand::Left);
    let (mut rheader, mut rinfo) = riter.next()?.into_info(Hand::Right);

    // TODO: adjust lheader

    let left = std::iter::from_fn(move || -> Option<Info> {
        let mut li = linfo.next();
        while li.is_none() {
            if let Some(hunk) = liter.peek() {
                if header::overlap(hunk.header(), &rheader) {
                    // TODO: adjust lheader

                    (lheader, linfo) = liter.next()?.into_info(Hand::Left);
                    li = linfo.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        li
    });

    let right = std::iter::from_fn(move || -> Option<Info> {
        let mut ri = rinfo.next();
        while ri.is_none() {
            if let Some(hunk) = riter.peek() {
                if header::overlap(&lheader, hunk.header()) {
                    // TODO: adjust lheader

                    (rheader, rinfo) = riter.next()?.into_info(Hand::Right);
                    ri = rinfo.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        ri
    });

    Some((left, right))
}

pub fn merge_iter2<T: Iterator<Item = Hunk>, U: Iterator<Item = Hunk>>(
    mut liter: Peekable<T>,
    mut riter: Peekable<U>,
) -> impl Iterator<Item = Result<Hunk, MergeError>> {
    std::iter::from_fn(move || -> Option<Result<Hunk, MergeError>> {
        match [liter.peek(), riter.peek()] {
            [None, None] => None,
            [None, Some(_)] => Some(Ok(riter.next()?)),
            [Some(_), None] => Some(Ok(liter.next()?)),
            [Some(lhs), Some(rhs)] => {
                if !lhs.overlaps(rhs) {
                    if lhs.cmp(rhs) == Ordering::Less {
                        Some(Ok(liter.next()?))
                    } else {
                        Some(Ok(riter.next()?))
                    }
                } else {
                    let (mut lchain, mut rchain) =
                        into_info_chains(&mut liter, &mut riter)?;

                    loop {
                        let a = lchain.next()?;
                        let b = rchain.next()?;
                    }

                    // let mut lheader = lhs.header().clone();
                    // let mut rheader = rhs.header().clone();

                    // let mut lhunks = std::iter::from_fn(|| -> Option<Hunk> {
                    //     let peek = liter.peek()?;
                    //     if header::overlap(peek.header(), &rheader) {
                    //         liter.next()
                    //     } else {
                    //         None
                    //     }
                    // });

                    // let mut rhunks = std::iter::from_fn(|| -> Option<Hunk> {
                    //     let peek = riter.peek()?;
                    //     if header::overlap(&lheader, peek.header()) {
                    //         riter.next()
                    //     } else {
                    //         None
                    //     }
                    // });

                    // let (_, mut linfo) = lhunks.next()?.into_info(Hand::Left);
                    // let (_, mut rinfo) = rhunks.next()?.into_info(Hand::Right);

                    // let mut li = std::iter::from_fn(|| -> Option<Info> {
                    //     loop {
                    //         if let Some(info) = linfo.next() {
                    //             return Some(info);
                    //         }
                    //         if let Some(hunk) = lhunks.next() {
                    //             lheader = hunk.header().clone();
                    //             let (_, i) = hunk.into_info(Hand::Left);
                    //             linfo = i;
                    //         } else {
                    //             return None;
                    //         }
                    //     }
                    // });

                    // let mut ri = std::iter::from_fn(|| -> Option<Info> {
                    //     loop {
                    //         if let Some(info) = rinfo.next() {
                    //             return Some(info);
                    //         }
                    //         if let Some(hunk) = rhunks.next() {
                    //             rheader = hunk.header().clone();
                    //             let (_, i) = hunk.into_info(Hand::Right);
                    //             linfo = i;
                    //         } else {
                    //             return None;
                    //         }
                    //     }
                    // });

                    todo!()
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::file_diff::merge_iter;
    use crate::hunk::Hunk;

    #[test]
    fn case_1() {
        let lhunks = vec![
            Hunk::new(
                [1, 2, 1, 2],
                vec![" a", " b"].iter().map(|s| s.to_string()).collect(),
            ),
            Hunk::new(
                [3, 2, 3, 2],
                vec![" c", " d"].iter().map(|s| s.to_string()).collect(),
            ),
            Hunk::new(
                [10, 2, 10, 2],
                vec![" c", " d"].iter().map(|s| s.to_string()).collect(),
            ),
        ];

        let rhunks = vec![
            Hunk::new(
                [1, 5, 1, 5],
                vec![" 1", " 2", " 3", " 4", " 5"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            Hunk::new(
                [10, 2, 10, 2],
                vec![" c", " d"].iter().map(|s| s.to_string()).collect(),
            ),
        ];

        let mut liter = lhunks.into_iter().peekable();
        let mut riter = rhunks.into_iter().peekable();

        if let Some((mut lchain, mut rchain)) =
            merge_iter::into_info_chains(&mut liter, &mut riter)
        {
            loop {
                let a = lchain.next();
                let b = rchain.next();
                if a.is_none() && b.is_none() {
                    break;
                }
                println!("{a:?}, {b:?}");
            }
        }

        for h in liter {
            println!("left -- {h}");
        }

        for h in riter {
            println!("right -- {h}");
        }
    }
}
