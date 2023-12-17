// TODO: rename to Line
use crate::line_no::LineNo;

#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub line_no: LineNo,
    pub rank: i64, // lineno, but with a caveat, see InfoIter
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or('!')
    }
}

impl From<(String, LineNo, i64)> for Info {
    fn from(tuple: (String, LineNo, i64)) -> Info {
        let (line, line_no, rank) = tuple;
        Info {
            line,
            line_no,
            rank,
        }
    }
}

impl From<(&str, LineNo, i64)> for Info {
    fn from(tuple: (&str, LineNo, i64)) -> Info {
        let (line, line_no, rank) = tuple;
        (line.to_string(), line_no, rank).into()
    }
}

impl From<(&str, [i64; 2], i64)> for Info {
    fn from(tuple: (&str, [i64; 2], i64)) -> Info {
        let (line, line_no, rank) = tuple;
        (line.to_string(), line_no.into(), rank).into()
    }
}
