use crate::error::MergeError;
use crate::hand::Hand;
use crate::hunk::handed::{HandedHunk, Mergeable};
use crate::hunk::{header, Hunk};
use crate::info::Info;
use crate::macros::debugln;
use crate::macros::merge_err;
use core::cmp::Ordering;
use std::iter::Peekable;

use crate::merge::merge_fn::merge_fn;

pub fn merge_overlapping<
    'a,
    T: Iterator<Item = Hunk>,
    U: Iterator<Item = Hunk>,
>(
    mut liter: &'a mut Peekable<T>,
    mut riter: &'a mut Peekable<U>,
) -> Result<Hunk, MergeError> {
    let (mut lheader, mut linfo) = liter
        .next()
        .ok_or(merge_err!("Expected hunk in left iterator"))?
        .into_info(Hand::Left);

    let (mut rheader, mut rinfo) = riter
        .next()
        .ok_or(merge_err!("Expected hunk in right iterator"))?
        .into_info(Hand::Right);

    // BUG: the two lambdas end up with local copies of the headers, so
    // one cannot notify the other. To fix this, might be better to switch
    // to pull model:

    // struct InfoSource<T: Iterator<Item = Hunk>, U: Iterator<Item = Hunk>> {
    //     lhunks: T,
    //     rhunks: U,
    //     linfo: InfoIter, // should implement InfoIter::empty()
    //     rinfo: InfoIter,
    // }

    // impl InfoSource {
    //     pub fn new<T: Iterator<Item = Hunk>, U: Iterator<Item = Hunk>>(
    //     ) -> InfoSource {
    //         // TODO: adjust left header
    //         todo!()
    //     }

    //     pub fn peek_left(&self) -> Option<&Info> {
    //         // TODO: adjust left header
    //         todo!()
    //     }

    //     pub fn peek_right(&self) -> Option<&Info> {
    //         // TODO: adjust left header
    //         todo!()
    //     }

    //     pub fn next_left(&mut self) -> Option<Info> {
    //         todo!()
    //     }

    //     pub fn next_right(&mut self) -> Option<Info> {
    //         todo!()
    //     }
    // }

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
        // println!("R -- {lh_ref:?}");
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

    let (header, lines) =
        merge_fn(&lheader.clone(), left, &rheader.clone(), right)?;

    Ok(Hunk::new(
        header,
        lines.into_iter().map(|(_, s)| s).collect(),
    ))
}

pub fn merge_iter<T: Iterator<Item = Hunk>, U: Iterator<Item = Hunk>>(
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
                    Some(merge_overlapping(&mut liter, &mut riter))
                }
            }
        }
    })
}
