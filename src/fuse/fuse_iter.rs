use crate::fuse::core::fuse;
use crate::fuse::info_chain::{HunkIter, InfoChain};

use crate::error::MergeError;
use crate::hunk::Hunk;
use crate::macros::{debugln, merge_err};

use core::cmp::Ordering;
use std::iter::Peekable;

pub fn fuse_iter(
    lhunks: Vec<Hunk>,
    rhunks: Vec<Hunk>,
) -> impl Iterator<Item = Result<Hunk, MergeError>> {
    // A file diff is an ordered set X[i], i >= 0 of non-overlapping hunks.
    // Consider two file diffs, X and Y.
    //
    // If X[i].overlaps(Y[j]) is false, then:
    //   1. X[i].overlaps(Y[k]) is false for k > j
    //   2. X[i].overlaps(X[l].merge(Y[k])) is false for l > i and k >= j
    //
    // To see why point 2. applies consider two overlapping hunks x & y.
    // For headers rx, ry and rxy of x, y amd x.merge(y) respectively:
    //   1. rxy[0] = min(rx[0], ry[0])
    //   2. rxy[2] = min(rx[2], ry[2])
    //
    // So if hunk z doesn't overlap x or y, it's clear that:
    //   1. rz[0] + rz[1] < min(rx[0], ry[0])
    //   2. rz[2] + rz[3] < min(rx[2], ry[2])
    debugln!(
        "Fusing file diffs with {} and {} hunks",
        lhunks.len(),
        rhunks.len()
    );

    let mut liter = lhunks.into_iter().peekable();
    let mut riter = rhunks.into_iter().peekable();
    let mut offset = 0i64;
    std::iter::from_fn(move || -> Option<Result<Hunk, MergeError>> {
        match [liter.peek(), riter.peek()] {
            [None, None] => None,
            [None, Some(rhs)] => {
                debugln!("fuse_iter: right -- {rhs}");
                let hunk = riter.next()?;
                offset += hunk.offset();
                Some(Ok(hunk))
            }
            [Some(lhs), None] => {
                debugln!("fuse_iter: left -- {lhs}, offset -- {offset}");
                Some(liter.next()?.with_offset(offset))
            }
            [Some(lhs), Some(rhs)] => {
                if !lhs.overlaps(rhs) {
                    if lhs.cmp(rhs) == Ordering::Less {
                        debugln!(
                            "fuse_iter: left -- {lhs}, offset -- {offset}"
                        );
                        Some(liter.next()?.with_offset(offset))
                    } else {
                        debugln!("fuse_iter: right -- {rhs}");
                        offset += rhs.offset();
                        Some(Ok(riter.next()?))
                    }
                } else {
                    debugln!("fuse_iter: Merging {lhs} and {rhs}");
                    match fuse_overlapping(&mut liter, &mut riter) {
                        Ok(hunk) => {
                            offset += hunk.offset();
                            Some(Ok(hunk))
                        }
                        err => Some(err),
                    }
                }
            }
        }
    })
}

fn fuse_overlapping(
    lhunks: &mut Peekable<HunkIter>,
    rhunks: &mut Peekable<HunkIter>,
) -> Result<Hunk, MergeError> {
    let header = if let (Some(l), Some(r)) = (lhunks.peek(), rhunks.peek()) {
        l.header().fuse(r.header())
    } else {
        return Err(merge_err!("fuse_overlapping: peek returned None"));
    };
    fuse(header, InfoChain::new(lhunks, rhunks)?)
}
