use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct ParseErr {
    _msg: String,
}

impl ParseErr {
    pub fn from(msg: String) -> ParseErr {
        ParseErr { _msg: msg }
    }
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ParseError: {}", self._msg)
    }
}

impl Error for ParseErr {}

impl From<std::io::Error> for ParseErr {
    fn from(err: std::io::Error) -> ParseErr {
        ParseErr {
            _msg: format!("IOError: {err:?}"),
        }
    }
}

#[derive(Debug)]
pub struct MergeErr {
    _msg: String,
}

impl MergeErr {
    pub fn from(msg: String) -> MergeErr {
        MergeErr { _msg: msg }
    }
}

impl Display for MergeErr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "MergeError: {}", self._msg)
    }
}

impl Error for MergeErr {}
