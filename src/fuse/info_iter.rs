use crate::fuse::info::Info;
use crate::line_no::LineNo;

use crate::hunk::{Header, Hunk};

type LineIter = std::vec::IntoIter<String>;

pub struct InfoIter {
    lines: LineIter,
    line_no: LineNo,
    rank: i64,
    kind: char,
}

impl InfoIter {
    pub fn left(lines: LineIter, header: &Header) -> InfoIter {
        let kind = '+';
        InfoIter {
            lines,
            line_no: (&header.fields).into(),
            rank: header.fields[2],
            kind,
        }
    }

    pub fn right(lines: LineIter, header: &Header) -> InfoIter {
        let kind = '-';
        InfoIter {
            lines,
            line_no: (&header.fields).into(),
            rank: header.fields[0],
            kind,
        }
    }
}

impl Iterator for InfoIter {
    type Item = Info;

    fn next(&mut self) -> Option<Info> {
        let line = self.lines.next()?;
        let info: Info =
            (line, self.line_no.clone(), self.rank.clone()).into();
        if info.line.starts_with([self.kind, ' ']) {
            self.rank += 1;
        }
        self.line_no.bump(info.prefix());
        Some(info)
    }
}

impl Default for InfoIter {
    fn default() -> InfoIter {
        InfoIter {
            lines: LineIter::default(),
            line_no: LineNo::default(),
            rank: 0,
            kind: '!',
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fuse::info::Info;
    use crate::fuse::info_iter::{InfoIter, LineIter};
    use crate::line_no::LineNo;

    fn split(line: &str) -> LineIter {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| line[i..j].to_string())
            .collect::<Vec<String>>()
            .into_iter()
    }

    fn test(actual: InfoIter, expected: Vec<(&str, [i64; 2], i64)>) {
        for (act, exp) in actual.zip(expected.into_iter()) {
            assert_eq!(act, Info::from(exp));
        }
    }

    fn test_left(
        header: [i64; 4],
        lines: LineIter,
        expected: Vec<(&str, [i64; 2], i64)>,
    ) {
        test(InfoIter::left(lines, &header.into()), expected);
    }

    fn test_right(
        header: [i64; 4],
        lines: LineIter,
        expected: Vec<(&str, [i64; 2], i64)>,
    ) {
        test(InfoIter::right(lines, &header.into()), expected);
    }

    #[test]
    fn case_1() {
        test_left(
            [1, 0, 1, 0],
            split("+ -+ +- -"),
            vec![
                ("+", [1, 1], 1),
                (" ", [1, 2], 2),
                ("-", [2, 3], 3),
                ("+", [3, 4], 3),
                (" ", [3, 5], 4),
                ("+", [4, 6], 5),
                ("-", [4, 5], 6),
                (" ", [5, 6], 6),
                ("-", [6, 6], 7),
            ],
        );
    }

    #[test]
    fn case_2() {
        test_right(
            [3, 0, 3, 0],
            split("+++- "),
            vec![
                ("+", [3, 3], 3),
                ("+", [3, 4], 3),
                ("+", [3, 5], 3),
                ("-", [4, 5], 3),
                (" ", [5, 6], 4),
            ],
        );
    }
}
