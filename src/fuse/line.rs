#[derive(Debug, PartialEq)]
pub struct Line {
    pub line: String,
    pub rank: i64, // lineno, but with a caveat, see InfoIter
}

impl Line {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or('!')
    }
}

impl From<(String, i64)> for Line {
    fn from(tuple: (String, i64)) -> Line {
        let (line, rank) = tuple;
        Line { line, rank }
    }
}

impl From<(&str, i64)> for Line {
    fn from(tuple: (&str, i64)) -> Line {
        let (line, rank) = tuple;
        (line.to_string(), rank).into()
    }
}
