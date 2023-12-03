use crate::error::MergeError;
use crate::hand::Hand;
use crate::hunk::{header, Hunk};
use crate::info::Info;
use crate::macros::merge_err;
use crate::merge::Merge;
use std::iter::repeat;

pub struct HandedHunk {
    hand: Hand,
    hunk: Hunk,
}

impl HandedHunk {
    fn into_info(mut self) -> ([usize; 4], impl Iterator<Item = Info>) {
        self.hunk.into_info(self.hand)
    }
}

impl From<HandedHunk> for Hunk {
    fn from(hunk: HandedHunk) -> Hunk {
        hunk.hunk
    }
}

impl From<(Hand, Hunk)> for HandedHunk {
    fn from(input: (Hand, Hunk)) -> HandedHunk {
        HandedHunk {
            hand: input.0,
            hunk: input.1,
        }
    }
}

pub trait Mergeable<T> {
    fn overlaps(&self, other: &T) -> bool;
    fn merge(self, other: T) -> Result<Merge, MergeError>;
}

impl Mergeable<HandedHunk> for HandedHunk {
    fn overlaps(&self, other: &HandedHunk) -> bool {
        match [&self.hand, &other.hand] {
            [Hand::Left, Hand::Right] => {
                header::overlap(self.hunk.header(), other.hunk.header())
            }
            [Hand::Right, Hand::Left] => {
                header::overlap(other.hunk.header(), self.hunk.header())
            }
            _ => false,
        }
    }

    fn merge(mut self, other: HandedHunk) -> Result<Merge, MergeError> {
        if self.hand == other.hand {
            return Err(merge_err!("Cannot merge same-handed hunks"));
        }
        let (lheader, liter) = self.into_info();
        let (rheader, riter) = other.into_info();
        Merge::new(&lheader, liter, &rheader, riter)
    }
}

impl Mergeable<Merge> for HandedHunk {
    fn overlaps(&self, other: &Merge) -> bool {
        match &self.hand {
            Hand::Left => header::overlap(self.hunk.header(), other.header()),
            Hand::Right => header::overlap(other.header(), self.hunk.header()),
            _ => false,
        }
    }

    fn merge(mut self, other: Merge) -> Result<Merge, MergeError> {
        let hand = self.hand.clone();
        let (lheader, liter) = self.into_info();
        let (rheader, riter) = other.into_info(hand);
        Merge::new(&lheader, liter, &rheader, riter)
    }
}

impl std::fmt::Display for HandedHunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hunk)
    }
}
