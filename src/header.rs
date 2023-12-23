use crate::error::ParseErr;
use crate::macros::parse_err;

#[derive(Debug, Clone)]
pub struct Header {
    _lines: Vec<String>,
    _file_name: String,
}

fn get_line<'a, T: Iterator<Item = &'a str>>(
    lines: &mut T,
) -> Result<String, ParseErr> {
    lines
        .next()
        .map(|s| s.to_string())
        .ok_or(parse_err!("Header: Could not get line"))
}

fn get_file_name(line: &String) -> Result<String, ParseErr> {
    line.strip_prefix("Index: ")
        .map(|s| s.to_string())
        .ok_or(parse_err!("Header: Unexpected suffix in '{line}'"))
}

impl Header {
    pub fn from_lines<'a, T: Iterator<Item = &'a str>>(
        lines: &mut T,
    ) -> Result<Header, ParseErr> {
        let _lines: Vec<_> = vec![
            get_line(lines)?,
            get_line(lines)?,
            get_line(lines)?,
            get_line(lines)?,
        ];

        let _file_name = _lines
            .get(0)
            .map(get_file_name)
            .ok_or(parse_err!("Header: Missing first line"))??;

        Ok(Header { _lines, _file_name })
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }

    pub fn file_name(&self) -> &str {
        &self._file_name
    }
}
