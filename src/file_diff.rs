use crate::error::ParseError;
use crate::header::Header;
use crate::hunk::Hunk;
use crate::macros::debugln;

#[derive(Debug)]
pub struct FileDiff<'a> {
    header: Header<'a>,
    hunks: Vec<Hunk<'a>>,
}

impl FileDiff<'_> {
    pub fn parse<'a>(lines: &'a [&'a str]) -> Result<FileDiff<'a>, ParseError> {
        let header = Header::parse(lines)?;
        let mut view = &lines[header.lines().len()..];
        let mut hunks: Vec<Hunk> = Vec::new();

        loop {
            let hunk = Hunk::parse(view)?;
            debugln!("Got hunk {:?}", hunk);
            view = &view[hunk.lines().len()..];
            hunks.push(hunk);

            let predicate = |s: &&str| s.starts_with("Index: ");
            if view.get(0).map_or_else(|| true, predicate) {
                break;
            }
        }

        Ok(FileDiff { header, hunks })
    }
}
