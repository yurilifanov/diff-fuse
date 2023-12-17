use crate::fuse::info::Info;
use crate::fuse::info_iter::InfoIter;
use crate::fuse::info_source::InfoSource;

use crate::error::MergeError;
use crate::hunk::{Header, Hunk};
use crate::macros::merge_err;
use std::iter::Peekable;

pub type HunkIter = std::vec::IntoIter<Hunk>;

struct Chain<'a, const T: char> {
    hunk_iter: &'a mut Peekable<HunkIter>,
    info_iter: Peekable<InfoIter>,
}

impl<'a, const T: char> Chain<'_, T> {
    pub fn new(mut hunk_iter: &'a mut Peekable<HunkIter>) -> Chain<'a, T> {
        let info_iter = match hunk_iter.next() {
            None => InfoIter::default().peekable(),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let iter = match T {
                    'L' => InfoIter::left(lines, &header),
                    _ => InfoIter::right(lines, &header),
                };
                iter.peekable()
            }
        };

        Chain {
            hunk_iter,
            info_iter,
        }
    }

    pub fn peek(&mut self, header: &Header) -> Option<&Info> {
        loop {
            if self.info_iter.peek().is_some() {
                return self.info_iter.peek();
            } else if let Some(peek) = self.hunk_iter.peek() {
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
                self.info_iter = match T {
                    'L' => InfoIter::left(lines, &header),
                    _ => InfoIter::right(lines, &header),
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
    lhunks: &'a mut Peekable<HunkIter>,
    rhunks: &'a mut Peekable<HunkIter>,
    linfo: Peekable<InfoIter>,
    rinfo: Peekable<InfoIter>,
    lheader: Header,
    rheader: Header,
}

impl<'a> InfoChain<'_> {
    pub fn new(
        mut lhunks: &'a mut Peekable<HunkIter>,
        mut rhunks: &'a mut Peekable<HunkIter>,
    ) -> Result<InfoChain<'a>, MergeError> {
        let (lheader, linfo) = match lhunks.next() {
            None => (Header::default(), InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let iter = InfoIter::left(lines, &header).peekable();
                (header, iter)
            }
        };

        let (rheader, rinfo) = match rhunks.next() {
            None => (Header::default(), InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let iter = InfoIter::right(lines, &header).peekable();
                (header, iter)
            }
        };

        Ok(InfoChain {
            lhunks,
            rhunks,
            linfo,
            rinfo,
            lheader,
            rheader,
        })
    }

    fn peek_right(
        mut info_iter: &'a mut Peekable<InfoIter>,
        mut hunk_iter: &'a mut Peekable<HunkIter>,
        header_in: &Header,
    ) -> (Option<Header>, Option<&'a Info>) {
        let mut header_out: Option<Header> = None;
        loop {
            if info_iter.peek().is_some() {
                return (header_out, info_iter.peek());
            } else if let Some(peek) = hunk_iter.peek() {
                if !header_in.overlaps(peek.header()) {
                    return (None, None);
                }
                let (header, lines) = hunk_iter.next().unwrap().unpack();
                let info = InfoIter::right(lines, &header);
                header_out = Some(header);
                *info_iter = info.peekable();
            } else {
                return (None, None);
            }
        }
    }

    fn peek_left(
        mut info_iter: &'a mut Peekable<InfoIter>,
        mut hunk_iter: &'a mut Peekable<HunkIter>,
        header_in: &Header,
    ) -> (Option<Header>, Option<&'a Info>) {
        let mut header_out: Option<Header> = None;
        loop {
            if info_iter.peek().is_some() {
                return (header_out, info_iter.peek());
            } else if let Some(peek) = hunk_iter.peek() {
                if !peek.header().overlaps(header_in) {
                    return (None, None);
                }
                let (header, lines) = hunk_iter.next().unwrap().unpack();
                let info = InfoIter::left(lines, &header);
                header_out = Some(header);
                *info_iter = info.peekable();
            } else {
                return (None, None);
            }
        }
    }
}

impl<'a> InfoSource for InfoChain<'_> {
    fn peek(&mut self) -> [Option<&Info>; 2] {
        let linfo = match Self::peek_left(
            &mut self.linfo,
            &mut self.lhunks,
            &self.rheader,
        ) {
            (Some(header), info) => {
                self.lheader = header;
                info
            }
            (_, info) => info,
        };

        let rinfo = match Self::peek_right(
            &mut self.rinfo,
            &mut self.rhunks,
            &self.lheader,
        ) {
            (Some(header), info) => {
                self.rheader = header;
                info
            }
            (_, info) => info,
        };

        [linfo, rinfo]
    }

    fn next_left(&mut self) -> Option<Info> {
        loop {
            if let Some(info) = self.linfo.next() {
                return Some(info);
            } else if let Some(peek) = self.lhunks.peek() {
                if !peek.header().overlaps(&self.rheader) {
                    return None;
                }
                let (header, lines) = self.lhunks.next()?.unpack();
                let info = InfoIter::left(lines, &header);
                self.lheader = header;
                self.linfo = info.peekable();
            } else {
                return None;
            }
        }
    }

    fn next_right(&mut self) -> Option<Info> {
        loop {
            if let Some(info) = self.rinfo.next() {
                return Some(info);
            } else if let Some(peek) = self.rhunks.peek() {
                if !self.lheader.overlaps(peek.header()) {
                    return None;
                }
                let (header, lines) = self.rhunks.next()?.unpack();
                let info = InfoIter::right(lines, &header);
                self.rheader = header;
                self.rinfo = info.peekable();
            } else {
                return None;
            }
        }
    }

    fn update_header(&mut self, lincrement: i64, rincrement: i64) {
        todo!()
    }
}
