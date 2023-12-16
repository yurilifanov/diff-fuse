// TODO: rename to Line

#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub rank: usize, // lineno, but with a caveat, see InfoIter
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or('!')
    }
}

impl From<(&str, usize)> for Info {
    fn from(tuple: (&str, usize)) -> Info {
        Info {
            line: tuple.0.to_string(),
            rank: tuple.1,
        }
    }
}

impl From<(String, usize)> for Info {
    fn from(tuple: (String, usize)) -> Info {
        Info {
            line: tuple.0,
            rank: tuple.1,
        }
    }
}
