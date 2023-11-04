use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub struct Info<'a> {
    pub line: &'a str,
    pub num: usize,
}

pub struct InfoIter<'a, T: Iterator<Item = &'a str> + Clone> {
    after: Peekable<LineIter<'a, T>>,
    before: Peekable<LineIter<'a, T>>,
}

impl<'a, T: Iterator<Item = &'a str> + Clone> InfoIter<'a, T> {
    pub fn new(header: &[usize; 4], iter: T) -> InfoIter<'a, T> {
        InfoIter {
            after: LineIter::<'a> {
                iter: iter.clone(),
                prefix: '+',
                num: header[2],
            }
            .peekable(),
            before: LineIter::<'a> {
                iter: iter,
                prefix: '-',
                num: header[0],
            }
            .peekable(),
        }
    }
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for InfoIter<'a, T> {
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
            (Some(_), None) => Some((self.after.next(), None)),
            (None, Some(_)) => Some((None, self.before.next())),
            _ => None,
        }
    }
}

struct LineIter<'a, T: Iterator<Item = &'a str> + Clone> {
    iter: T,
    prefix: char,
    num: usize,
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for LineIter<'a, T> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.iter.next()?;
            if line.starts_with([' ', self.prefix]) {
                let num = self.num;
                self.num += 1;
                return Some(Info { line, num });
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

        fn info<'a>(s: &'a str, n: usize) -> Option<Info<'a>> {
            Some(Info { line: s, num: n })
        }

        let header: [usize; 4] = [1, 0, 1, 0];

        let expected = [
            (info("+", 1), None),
            (info(" ", 2), info(" ", 1)),
            (info("+", 3), info("-", 2)),
            (info(" ", 4), info(" ", 3)),
            (info("+", 5), info("-", 4)),
            (info(" ", 6), info(" ", 5)),
            (None, info("-", 6)),
        ];

        let actual = InfoIter::new(&header, data.iter().map(|s| s.as_str()));
        for (act, exp) in actual.zip(expected.iter()) {
            assert_eq!(act, *exp);
        }
    }
}
