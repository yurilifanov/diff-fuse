use crate::fuse::info::Info;
use crate::fuse::info_iter::InfoIter;
use crate::fuse::info_source::InfoSource;

use crate::error::MergeError;
use crate::hunk::{header, Hunk};
use crate::macros::merge_err;
use std::iter::Peekable;

pub type HunkIter = std::vec::IntoIter<Hunk>;
type Header = [usize; 4];

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
            None => ([0usize, 0, 0, 0], InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                (header, InfoIter::left(lines, header[2]).peekable())
            }
        };

        let (rheader, rinfo) = match rhunks.next() {
            None => ([0usize, 0, 0, 0], InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let rank = header[0];
                (header, InfoIter::right(lines, rank).peekable())
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
        // let mut ofs: i64 = 0;
        let mut header_out: Option<Header> = None;
        loop {
            if info_iter.peek().is_some() {
                return (header_out, info_iter.peek());
            } else if let Some(hunk) = hunk_iter.next() {
                if !header::overlap(header_in, hunk.header()) {
                    // return (0, None, None);
                    return (None, None);
                }
                let (header, lines) = hunk.unpack();
                let info = InfoIter::right(lines, header[0]);
                header_out = Some(header);
                *info_iter = info.peekable();
            } else {
                // return (0, None, None);
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
            } else if let Some(hunk) = hunk_iter.next() {
                if !header::overlap(hunk.header(), header_in) {
                    return (None, None);
                }
                let (header, lines) = hunk.unpack();
                let info = InfoIter::left(lines, header[2]);
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
            } else if let Some(hunk) = self.lhunks.next() {
                if !header::overlap(hunk.header(), &self.rheader) {
                    return None;
                }
                let (header, lines) = hunk.unpack();
                let info = InfoIter::left(lines, header[2]);
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
            } else if let Some(hunk) = self.rhunks.next() {
                if !header::overlap(&self.lheader, hunk.header()) {
                    return None;
                }
                let (header, lines) = hunk.unpack();
                let info = InfoIter::right(lines, header[0]);
                self.rheader = header;
                self.rinfo = info.peekable();
            } else {
                return None;
            }
        }
    }
}
