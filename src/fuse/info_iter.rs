use crate::fuse::info::Info;

use crate::hunk::Hunk;

type LineIter = std::vec::IntoIter<String>;

pub struct InfoIter {
    lines: LineIter,
    rank: usize,
    kind: char,
}

impl InfoIter {
    pub fn left(lines: LineIter, rank: usize) -> InfoIter {
        let kind = '+';
        InfoIter { lines, rank, kind }
    }

    pub fn right(lines: LineIter, rank: usize) -> InfoIter {
        let kind = '-';
        InfoIter { lines, rank, kind }
    }
}

impl Iterator for InfoIter {
    type Item = Info;

    fn next(&mut self) -> Option<Info> {
        let line = self.lines.next()?;
        let info: Info = (line, self.rank.clone()).into();
        if info.line.starts_with([self.kind, ' ']) {
            self.rank += 1;
        }
        Some(info)
    }
}

impl Default for InfoIter {
    fn default() -> InfoIter {
        InfoIter {
            lines: LineIter::default(),
            rank: 0,
            kind: '!',
        }
    }
}
