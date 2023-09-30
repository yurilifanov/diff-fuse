use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct ParseError {
    _msg: String,
}

impl ParseError {
    pub fn from(msg: String) -> ParseError {
        ParseError { _msg: msg }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ParseError: {}", self._msg)
    }
}

impl Error for ParseError {}
