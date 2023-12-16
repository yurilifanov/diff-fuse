use crate::fuse::info::Info;
use crate::fuse::info_iter::InfoIter;
use crate::fuse::info_source::InfoSource as Trait;

use crate::hunk::Hunk;

use std::iter::Peekable;

pub struct InfoSource {
    left: Peekable<InfoIter>,
    right: Peekable<InfoIter>,
}

impl InfoSource {
    pub fn new(left: Hunk, right: Hunk) -> InfoSource {
        let (lheader, llines) = left.unpack();
        let (rheader, rlines) = right.unpack();
        InfoSource {
            left: InfoIter::left(llines, lheader[2]).peekable(),
            right: InfoIter::right(rlines, rheader[0]).peekable(),
        }
    }
}

impl Trait for InfoSource {
    fn peek(&mut self) -> [Option<&Info>; 2] {
        [self.left.peek(), self.right.peek()]
    }

    fn next_right(&mut self) -> Option<Info> {
        self.right.next()
    }

    fn next_left(&mut self) -> Option<Info> {
        self.left.next()
    }
}
