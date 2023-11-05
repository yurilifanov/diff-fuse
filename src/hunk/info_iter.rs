use core::cmp::Ordering;
use core::iter::Filter;
use std::iter::Peekable;

type Info<'a> = (Option<&'a str>, Option<&'a str>, usize);

pub struct InfoIter<'a, T: Iterator<Item = &'a str> + Clone> {
    after: Peekable<LineIter<'a, T>>,
    before: Peekable<LineIter<'a, T>>,
    num_after: usize,
    num_before: usize,
}

impl<'a, T: Iterator<Item = &'a str> + Clone> InfoIter<'a, T> {
    pub fn new(header: &[usize; 4], iter: T) -> InfoIter<'a, T> {
        InfoIter {
            after: LineIter::<'a> {
                iter: iter.clone(),
                prefix: '+',
            }
            .peekable(),
            before: LineIter::<'a> {
                iter: iter,
                prefix: '-',
            }
            .peekable(),
            num_after: header[2],
            num_before: header[0],
        }
    }
}

fn post_increment(reference: &mut usize) -> usize {
    let prev: usize = *reference;
    *reference += 1;
    prev
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for InfoIter<'a, T> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.after.peek(), self.before.peek()) {
            (Some(a), Some(b)) => match self.num_after.cmp(&self.num_before) {
                Ordering::Greater => Some((
                    None,
                    self.before.next(),
                    post_increment(&mut self.num_before),
                )),
                Ordering::Less => Some((
                    self.after.next(),
                    None,
                    post_increment(&mut self.num_after),
                )),
                _ => {
                    self.num_before += 1;
                    Some((
                        self.after.next(),
                        self.before.next(),
                        post_increment(&mut self.num_after),
                    ))
                }
            },
            (Some(_), None) => Some((
                self.after.next(),
                None,
                post_increment(&mut self.num_after),
            )),
            (None, Some(_)) => Some((
                None,
                self.before.next(),
                post_increment(&mut self.num_after),
            )),
            _ => None,
        }
    }
}

struct LineIter<'a, T: Iterator<Item = &'a str> + Clone> {
    iter: T,
    prefix: char,
}

impl<'a, T: Iterator<Item = &'a str> + Clone> Iterator for LineIter<'a, T> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.iter.next()?;
            if line.starts_with([' ', self.prefix]) {
                return Some(line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::info_iter::{Info, InfoIter};

    fn split(line: &str) -> impl Iterator<Item = &str> + Clone {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| &line[i..j])
    }

    fn test<'a, T: Iterator<Item = &'a str> + Clone>(
        header: [usize; 4],
        lines: T,
        expected: Vec<Info>,
    ) {
        let actual = InfoIter::new(&header, lines);
        for (act, exp) in actual.zip(expected.iter()) {
            assert_eq!(act, *exp);
        }
    }

    #[test]
    fn case_1() {
        let header: [usize; 4] = [1, 0, 1, 0];
        let line = "+ -+ +- -";
        let expected: Vec<Info> = vec![
            (Some("+"), Some(" "), 1),
            (Some(" "), Some("-"), 2),
            (Some("+"), Some(" "), 3),
            (Some(" "), Some("-"), 4),
            (Some("+"), Some(" "), 5),
            (Some(" "), Some("-"), 6),
        ];
        test(header, split(line), expected);
    }

    #[test]
    fn case_2() {
        let header: [usize; 4] = [3, 0, 1, 0];
        let line = "+++- ";
        let expected: Vec<Info> = vec![
            (Some("+"), None, 1),
            (Some("+"), None, 2),
            (Some("+"), Some("-"), 3),
            (Some(" "), Some(" "), 4),
        ];
        test(header, split(line), expected);
    }
}
