// TODO: rename to Line
use crate::line_no::LineNo;

#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub line_no: LineNo,
    pub rank: usize, // lineno, but with a caveat, see InfoIter
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or('!')
    }
}

impl From<(String, LineNo, usize)> for Info {
    fn from(tuple: (String, LineNo, usize)) -> Info {
        let (line, line_no, rank) = tuple;
        Info {
            line,
            line_no,
            rank,
        }
    }
}

impl From<(&str, LineNo, usize)> for Info {
    fn from(tuple: (&str, LineNo, usize)) -> Info {
        let (line, line_no, rank) = tuple;
        (line.to_string(), line_no, rank).into()
    }
}

impl From<(&str, [usize; 2], usize)> for Info {
    fn from(tuple: (&str, [usize; 2], usize)) -> Info {
        let (line, line_no, rank) = tuple;
        (line.to_string(), line_no.into(), rank).into()
    }
}
