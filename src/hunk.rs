use crate::error::ParseError;
use crate::macros::parse_err;

#[derive(Debug)]
pub struct Hunk {
    _lines: Vec<String>,
    _header: [usize; 4],
}

fn split(string: &str) -> Vec<&str> {
    string.split(' ').flat_map(|s| s.split(',')).collect()
}

fn parse_usize(string: &&str) -> Result<usize, ParseError> {
    string
        .parse::<i64>()
        .map(|val| val.wrapping_abs() as usize)
        .map_err(|_| parse_err!("Hunk header: Could not parse i64 from {}", string))
}

fn parse_n<const N: usize>(values: &Vec<&str>) -> Result<[usize; N], ParseError> {
    values
        .iter()
        .flat_map(parse_usize)
        .collect::<Vec<usize>>()
        .try_into()
        .map_err(|_| parse_err!("Hunk header: Expected {} integers, got {:?}", N, values))
}

fn parse(values: Vec<&str>) -> Result<[usize; 4], ParseError> {
    match values.len() {
        2 => {
            let result = parse_n::<2>(&values)?;
            Ok([result[0], 1, result[1], 1])
        }
        3 => {
            let result = parse_n::<3>(&values)?;
            if values.get(1).map_or(false, |s| s.starts_with('+')) {
                return Ok([result[0], 1, result[1], result[2]]);
            }
            Ok([result[0], result[1], result[2], 1])
        }
        4 => parse_n::<4>(&values),
        _ => Err(parse_err!(
            "Hunk header: Unexpected number of fields - {:?}",
            values
        )),
    }
}

impl Hunk {
    fn parse_header(line: &&str) -> Result<[usize; 4], ParseError> {
        let fields = line
            .strip_prefix("@@ ")
            .map_or_else(|| None, |s| s.strip_suffix(" @@"))
            .map(split)
            .ok_or_else(|| parse_err!("Hunk: Could not parse header from '{}'", line))?;

        parse(fields)
    }

    pub fn parse(lines: &[&str]) -> Result<Hunk, ParseError> {
        let first = lines
            .get(0)
            .ok_or(parse_err!("Hunk: Could not fetch first line"))?;

        let _header: [usize; 4] = Hunk::parse_header(&first)?;

        let mut _lines: Vec<String> = vec![first.to_string()];
        for line in lines.iter().skip(1) {
            if !line.starts_with(['+', '-', ' ']) {
                break;
            }
            _lines.push(line.to_string());
        }

        Ok(Hunk { _lines, _header })
    }

    pub fn lines(&self) -> &Vec<String> {
        &self._lines
    }
}
