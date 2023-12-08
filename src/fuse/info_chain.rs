use crate::fuse::info::Info;
use crate::fuse::info_iter::InfoIter;

use crate::error::MergeError;
use crate::hunk::{header, Hunk};
use crate::macros::merge_err;
use std::iter::Peekable;

type HunkIter = std::vec::IntoIter<Hunk>;
type Header = [usize; 4];

fn get_rank(header_field: usize, offset: &i64) -> Result<usize, MergeError> {
    Ok(0)
}

pub struct InfoChain<'a> {
    lhunks: Peekable<HunkIter>,
    rhunks: Peekable<HunkIter>,
    linfo: Peekable<InfoIter>,
    rinfo: Peekable<InfoIter>,
    lheader: Header,
    rheader: Header,
    offset: &'a mut i64,
}

impl<'a> InfoChain<'_> {
    pub fn new(
        mut lhunks: Peekable<HunkIter>,
        mut rhunks: Peekable<HunkIter>,
        offset: &'a mut i64,
    ) -> Result<InfoChain<'a>, MergeError> {
        let (lheader, linfo) = match lhunks.next() {
            None => ([0usize, 0, 0, 0], InfoIter::default().peekable()),
            Some(hunk) => {
                let (header, lines) = hunk.unpack();
                let rank = get_rank(header[2], &*offset)?;
                (header, InfoIter::left(lines, rank).peekable())
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
            offset,
        })
    }

    pub fn peek(&mut self) -> Result<[Option<&Info>; 2], MergeError> {
        let linfo = match Self::peek_left(
            &mut self.linfo,
            &mut self.lhunks,
            &self.rheader,
            &*self.offset,
        )? {
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
            (offset, Some(header), info) => {
                *self.offset += offset;
                self.rheader = header;
                info
            }
            (offset, _, info) => {
                *self.offset += offset;
                info
            }
        };

        Ok([linfo, rinfo])
    }

    pub fn next_left(&mut self) -> Result<Option<Info>, MergeError> {
        loop {
            if let Some(info) = self.linfo.next() {
                return Ok(Some(info));
            } else if let Some(hunk) = self.lhunks.next() {
                if !header::overlap(hunk.header(), &self.rheader) {
                    return Ok(None);
                }
                let (header, lines) = hunk.unpack();
                let rank = get_rank(header[2], &*self.offset)?;
                let info = InfoIter::left(lines, rank);
                self.lheader = header;
                self.linfo = info.peekable();
            } else {
                return Ok(None);
            }
        }
    }

    pub fn next_right(&mut self) -> Option<Info> {
        loop {
            if let Some(info) = self.rinfo.next() {
                return Some(info);
            } else if let Some(hunk) = self.rhunks.next() {
                if !header::overlap(&self.lheader, hunk.header()) {
                    return None;
                }
                *self.offset += hunk.offset();
                let (header, lines) = hunk.unpack();
                let info = InfoIter::right(lines, header[0]);
                self.rheader = header;
                self.rinfo = info.peekable();
            } else {
                return None;
            }
        }
    }

    fn peek_right(
        mut info_iter: &'a mut Peekable<InfoIter>,
        mut hunk_iter: &'a mut Peekable<HunkIter>,
        header_in: &Header,
    ) -> (i64, Option<Header>, Option<&'a Info>) {
        let mut ofs: i64 = 0;
        let mut header_out: Option<Header> = None;
        loop {
            if info_iter.peek().is_some() {
                return (ofs, header_out, info_iter.peek());
            } else if let Some(hunk) = hunk_iter.next() {
                if !header::overlap(header_in, hunk.header()) {
                    return (0, None, None);
                }
                ofs += hunk.offset();
                let (header, lines) = hunk.unpack();
                let info = InfoIter::right(lines, header[0]);
                header_out = Some(header);
                *info_iter = info.peekable();
            } else {
                return (0, None, None);
            }
        }
    }

    fn peek_left(
        mut info_iter: &'a mut Peekable<InfoIter>,
        mut hunk_iter: &'a mut Peekable<HunkIter>,
        header_in: &Header,
        offset: &i64,
    ) -> Result<(Option<Header>, Option<&'a Info>), MergeError> {
        let mut header_out: Option<Header> = None;
        loop {
            if info_iter.peek().is_some() {
                return Ok((header_out, info_iter.peek()));
            } else if let Some(hunk) = hunk_iter.next() {
                if !header::overlap(hunk.header(), header_in) {
                    return Ok((None, None));
                }
                let (header, lines) = hunk.unpack();
                let rank = get_rank(header[2], offset)?;
                let info = InfoIter::left(lines, rank);
                header_out = Some(header);
                *info_iter = info.peekable();
            } else {
                return Ok((None, None));
            }
        }
    }
}
