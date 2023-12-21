// TODO: rename to Line
#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub rank: i64, // lineno, but with a caveat, see InfoIter
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or('!')
    }
}

impl From<(String, i64)> for Info {
    fn from(tuple: (String, i64)) -> Info {
        let (line, rank) = tuple;
        Info { line, rank }
    }
}

impl From<(&str, i64)> for Info {
    fn from(tuple: (&str, i64)) -> Info {
        let (line, rank) = tuple;
        (line.to_string(), rank).into()
    }
}
