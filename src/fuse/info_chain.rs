use crate::fuse::info::Info;
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
    pub fn new(mut hunk_iter: &'a mut Peekable<HunkIter>) -> Chain<'a, T> {
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
        } else if let Some(hunk) = self.hunk_iter.peek() {
            hunk.header().clone()
        } else {
            Header::default()
        }
    }

    pub fn peek(&mut self, header: &Header) -> Option<&Info> {
        loop {
            if self.info_iter.peek().is_some() {
                return self.info_iter.peek();
            } else if let Some(peek) = self.hunk_iter.peek() {
                println!("{T} -- {header} vs {}", peek.header());
                match T {
                    'L' => {
                        if !peek.header().overlaps(header) {
                            return None;
                        }
                    }
                    _ => {
                        if !header.overlaps(peek.header()) {
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

    pub fn next(&mut self, header: &Header) -> Option<Info> {
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
        mut lhunks: &'a mut Peekable<HunkIter>,
        mut rhunks: &'a mut Peekable<HunkIter>,
    ) -> Result<InfoChain<'a>, MergeError> {
        if let [Some(lhs), Some(rhs)] = [lhunks.peek(), rhunks.peek()] {
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
    fn peek(&mut self) -> [Option<&Info>; 2] {
        let lhdr = self.lchain.header();
        let rhdr = self.rchain.header();
        let tmp = [self.lchain.peek(&lhdr), self.rchain.peek(&rhdr)];
        println!("Chain peek: {tmp:?}");
        tmp
    }

    fn next_left(&mut self) -> Option<Info> {
        self.lchain.next(&self.rchain.header())
    }

    fn next_right(&mut self) -> Option<Info> {
        self.rchain.next(&self.lchain.header())
    }
}
