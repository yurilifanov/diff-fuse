use crate::error::ParseError;
use crate::macros::parse_err;

const HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub struct Header {
    _lines: Vec<String>,
    _file_name: String,
}

impl Header {
    fn parse_file_name(line: &str) -> Result<&str, ParseError> {
        let pat = "Index: ";
        let pos = line.find(pat).ok_or_else(|| {
            parse_err!(
                "Header: Could not extract file name from line '{}'",
                line
            )
        })?;
        Ok(&line[pos + pat.len()..])
    }

    pub fn parse(lines: &[&str]) -> Result<Header, ParseError> {
        let first_line = lines
            .get(0)
            .ok_or_else(|| parse_err!("Header: Could not fetch first line"))?;

        let _file_name =
            Header::parse_file_name(first_line.to_owned())?.to_string();

        let _lines: Vec<String> =
            lines[..HEADER_SIZE].iter().map(|s| s.to_string()).collect();

        if _lines.len() != HEADER_SIZE {
            return Err(parse_err!(
                "Header: Expected {} lines, got {}",
                HEADER_SIZE,
                _lines.len()
            ));
        }

        Ok(Header { _lines, _file_name })
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    pub fn file_name(&self) -> &str {
        &self._file_name
    }
}
