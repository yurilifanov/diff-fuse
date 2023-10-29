use crate::macros::warnln;
use std::iter::Peekable;
use std::slice::Iter;

pub fn iter_info<'a>(
    iter: Iter<'a, String>,
    header: &[usize; 4],
) -> impl Iterator<Item = Info<'a>> {
    LineIter {
        after: filter_info(iter.clone(), '+', header[2]),
        before: filter_info(iter, '-', header[0]),
    }
}

pub struct LineIter<'a, T: Iterator<Item = LineInfo<'a>>> {
    after: Peekable<T>,
    before: Peekable<T>,
}

#[derive(Debug, PartialEq)]
pub struct LineInfo<'a> {
    line: &'a str,
    index: usize,
    num: usize,
}

fn filter_info<'a>(
    iter: Iter<'a, String>,
    prefix: char,
    offset: usize,
) -> Peekable<impl Iterator<Item = LineInfo<'a>>> {
    let mut num: usize = offset;
    iter.enumerate()
        .filter_map(move |args| transform(&mut num, prefix, args))
        .peekable()
}

fn transform<'a>(
    _num: &mut usize,
    prefix: char,
    args: (usize, &'a String),
) -> Option<LineInfo<'a>> {
    if !args.1.is_empty() && !args.1.starts_with([' ', prefix]) {
        return None;
    }
    let num = *_num;
    *_num += 1;
    Some(LineInfo {
        line: args.1,
        index: args.0,
        num,
    })
}

type Info<'a> = (Option<LineInfo<'a>>, Option<LineInfo<'a>>);

impl<'a, T: Iterator<Item = LineInfo<'a>>> Iterator for LineIter<'a, T> {
    type Item = Info<'a>;

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

#[cfg(test)]
mod line_info {
    use crate::hunk::line_iter::{iter_info, LineInfo};

    #[test]
    fn test_iter_info() {
        let string = "+ -+ +- -";
        let data: Vec<String> =
            string.chars().map(|c| c.to_string()).collect();

        fn li<'a>(s: &'a str, i: usize, n: usize) -> Option<LineInfo<'a>> {
            Some(LineInfo {
                line: s,
                index: i,
                num: n,
            })
        }

        let header: [usize; 4] = [1, 0, 1, 0];

        let expected = [
            (li("+", 0, 1), None),
            (li(" ", 1, 2), li(" ", 1, 1)),
            (li("+", 3, 3), li("-", 2, 2)),
            (li(" ", 4, 4), li(" ", 4, 3)),
            (li("+", 5, 5), li("-", 6, 4)),
            (li(" ", 7, 6), li(" ", 7, 5)),
            (None, li("-", 8, 6)),
        ];

        let actual = iter_info(data.iter(), &header);
        for (act, exp) in actual.zip(expected.iter()) {
            assert_eq!(act, *exp);
        }
    }
}
