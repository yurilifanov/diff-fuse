use crate::fuse::line::Line;
use crate::fuse::info_iter::InfoIter;
use crate::fuse::info_source::InfoSource;

use crate::error::MergeError;
use crate::hunk::{Header, Hunk};
use crate::macros::{debugln, merge_err};
use std::iter::Peekable;

pub type HunkIter = std::vec::IntoIter<Hunk>;

struct Chain<'a, const T: char> {
    hunk_iter: &'a mut Peekable<HunkIter>,
    info_iter: Peekable<InfoIter>,
    header: Header,
}

impl<'a, const T: char> Chain<'_, T> {
    pub fn new(hunk_iter: &'a mut Peekable<HunkIter>) -> Chain<'a, T> {
        let (header, info_iter) = match hunk_iter.next() {
            None => (Header::default(), InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let iter = match T {
                    'L' => InfoIter::left(lines, &header),
                    _ => InfoIter::right(lines, &header),
                };
                (header, iter.peekable())
            }
        };

        Chain {
            hunk_iter,
            info_iter,
            header,
        }
    }

    pub fn header(&mut self) -> Header {
        if self.info_iter.peek().is_some() {
            self.header.clone()
        } else {
            Header::default()
        }
    }

    pub fn peek(&mut self, header: &Header) -> Option<&Line> {
        loop {
            if self.info_iter.peek().is_some() {
                return self.info_iter.peek();
            } else if let Some(peek) = self.hunk_iter.peek() {
                match T {
                    'L' => {
                        if !peek.header().should_fuse(header) {
                            return None;
                        }
                    }
                    _ => {
                        if !header.should_fuse(peek.header()) {
                            return None;
                        }
                    }
                }

                let (header, lines) = self.hunk_iter.next().unwrap().unpack();
                debugln!("Chain<{T}>: new hunk {header}");

                self.header = header;

                self.info_iter = match T {
                    'L' => InfoIter::left(lines, &self.header),
                    _ => InfoIter::right(lines, &self.header),
                }
                .peekable();
            } else {
                return None;
            }
        }
    }

    pub fn next(&mut self, header: &Header) -> Option<Line> {
        if self.peek(header).is_some() {
            self.info_iter.next()
        } else {
            None
        }
    }
}

pub struct InfoChain<'a> {
    lchain: Chain<'a, 'L'>,
    rchain: Chain<'a, 'R'>,
}

impl<'a> InfoChain<'_> {
    pub fn new(
        lhunks: &'a mut Peekable<HunkIter>,
        rhunks: &'a mut Peekable<HunkIter>,
    ) -> Result<InfoChain<'a>, MergeError> {
        if let [Some(_), Some(_)] = [lhunks.peek(), rhunks.peek()] {
            Ok(InfoChain {
                lchain: Chain::<'a, 'L'>::new(lhunks),
                rchain: Chain::<'a, 'R'>::new(rhunks),
            })
        } else {
            Err(merge_err!(
                "InfoChain: at least one of the hunk iterators is empty"
            ))
        }
    }
}

impl<'a> InfoSource for InfoChain<'_> {
    fn peek(&mut self) -> [Option<&Line>; 2] {
        let lhdr = self.lchain.header();
        let rhdr = self.rchain.header();
        [self.lchain.peek(&rhdr), self.rchain.peek(&lhdr)]
    }

    fn next_left(&mut self) -> Option<Line> {
        self.lchain.next(&self.rchain.header())
    }

    fn next_right(&mut self) -> Option<Line> {
        self.rchain.next(&self.lchain.header())
    }
}
