use crate::error::ParseError;
use crate::macros::parse_err;

const HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub struct Header<'a> {
    _lines: [&'a str; HEADER_SIZE],
    _file_name: &'a str,
}

impl Header<'_> {
    fn parse_file_name(line: &str) -> Result<&str, ParseError> {
        let pat = "Index: ";
        let pos = line.find(pat).ok_or_else(|| {
            parse_err!("Header: Could not extract file name from line '{}'", line)
        })?;
        Ok(&line[pos + pat.len()..])
    }

    pub fn parse<'a>(lines: &'a [&'a str]) -> Result<Header<'a>, ParseError> {
        let first_line = lines
            .get(0)
            .ok_or_else(|| parse_err!("Header: Could not fetch first line"))?;

        let file_name = Header::parse_file_name(first_line.to_owned())?;

        lines[..HEADER_SIZE]
            .try_into()
            .map(|&slice| Header {
                _lines: slice,
                _file_name: file_name,
            })
            .map_err(|err| parse_err!("Header: {}", err))
    }

    pub fn lines(&self) -> &[&str] {
        &self._lines
    }

    // pub fn file_name(&self) -> &str {
    //     &self._file_name
    // }
}
