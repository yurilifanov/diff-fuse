use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

#[derive(Debug, PartialEq)]
pub struct Info<'a> {
    pub line: &'a str,
    pub index: usize,
    pub num: usize,
}

pub struct InfoIter<'a> {
    after: Peekable<LineIter<'a>>,
    before: Peekable<LineIter<'a>>,
}

impl<'a> InfoIter<'a> {
    pub fn new(header: &[usize; 4], iter: Iter<'a, String>) -> InfoIter<'a> {
        InfoIter {
            after: LineIter::<'a> {
                iter: iter.clone().enumerate(),
                prefix: '+',
                num: header[2],
            }
            .peekable(),
            before: LineIter::<'a> {
                iter: iter.enumerate(),
                prefix: '-',
                num: header[0],
            }
            .peekable(),
        }
    }
}

impl<'a> Iterator for InfoIter<'a> {
    type Item = (Option<Info<'a>>, Option<Info<'a>>);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.after.peek(), self.before.peek()) {
            (Some(a), Some(b)) => {
                match (a.line.starts_with(' '), b.line.starts_with(' ')) {
                    (true, false) => Some((None, self.before.next())),
                    (false, true) => Some((self.after.next(), None)),
                    _ => Some((self.after.next(), self.before.next())),
                }
            }
            (Some(a), None) => Some((self.after.next(), None)),
            (None, Some(b)) => Some((None, self.before.next())),
            _ => None,
        }
    }
}

struct LineIter<'a> {
    iter: Enumerate<Iter<'a, String>>,
    prefix: char,
    num: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (index, line): (usize, &String) = self.iter.next()?;
            if line.starts_with([' ', self.prefix]) {
                let num = self.num;
                self.num += 1;
                return Some(Info { line, index, num });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::info_iter::{Info, InfoIter};

    #[test]
    fn test_info_iter() {
        let string = "+ -+ +- -";

        let data: Vec<String> =
            string.chars().map(|c| c.to_string()).collect();

        fn info<'a>(s: &'a str, i: usize, n: usize) -> Option<Info<'a>> {
            Some(Info {
                line: s,
                index: i,
                num: n,
            })
        }

        let header: [usize; 4] = [1, 0, 1, 0];

        let expected = [
            (info("+", 0, 1), None),
            (info(" ", 1, 2), info(" ", 1, 1)),
            (info("+", 3, 3), info("-", 2, 2)),
            (info(" ", 4, 4), info(" ", 4, 3)),
            (info("+", 5, 5), info("-", 6, 4)),
            (info(" ", 7, 6), info(" ", 7, 5)),
            (None, info("-", 8, 6)),
        ];

        let actual = InfoIter::new(&header, data.iter());
        for (act, exp) in actual.zip(expected.iter()) {
            assert_eq!(act, *exp);
        }
    }
}
