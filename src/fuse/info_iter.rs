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

#[cfg(test)]
mod tests {
    use crate::fuse::info::Info;
    use crate::fuse::info_iter::{InfoIter, LineIter};

    fn split(line: &str) -> LineIter {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| line[i..j].to_string())
            .collect::<Vec<String>>()
            .into_iter()
    }

    fn test(actual: InfoIter, expected: Vec<(&str, usize)>) {
        for (act, exp) in actual.zip(expected.into_iter()) {
            assert_eq!(act, Info::from(exp));
        }
    }

    fn test_left(rank: usize, lines: LineIter, expected: Vec<(&str, usize)>) {
        test(InfoIter::left(lines, rank), expected);
    }

    fn test_right(rank: usize, lines: LineIter, expected: Vec<(&str, usize)>) {
        test(InfoIter::right(lines, rank), expected);
    }

    #[test]
    fn case_1() {
        test_left(
            1,
            split("+ -+ +- -"),
            vec![
                ("+", 1),
                (" ", 2),
                ("-", 3),
                ("+", 3),
                (" ", 4),
                ("+", 5),
                ("-", 6),
                (" ", 6),
                ("-", 7),
            ],
        );
    }

    #[test]
    fn case_2() {
        test_right(
            3,
            split("+++- "),
            vec![("+", 3), ("+", 3), ("+", 3), ("-", 3), (" ", 4)],
        );
    }
}
