use core::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub struct Info<'a> {
    pub line: &'a str,
    pub minus_num: usize,
    pub plus_num: usize,
}

impl Info<'_> {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or(' ')
    }

    pub fn cmp(&self, other: &Info) -> Ordering {
        match [self.prefix(), other.prefix()] {
            ['-', ' '] => Ordering::Equal,
            ['-', '-'] => self.minus_num.cmp(&other.minus_num),
            ['-', '+'] => self.minus_num.cmp(&other.plus_num),
            ['+', ' '] => self.plus_num.cmp(&other.minus_num),
            ['+', '-'] => self.plus_num.cmp(&other.minus_num),
            ['+', '+'] => self.plus_num.cmp(&other.plus_num),
            [' ', '-'] => self.plus_num.cmp(&other.minus_num),
            [' ', '+'] => self.plus_num.cmp(&other.plus_num),
            _ => self.plus_num.cmp(&other.minus_num),
        }
    }
}

pub struct InfoIter<'a, T: Iterator<Item = &'a str>> {
    line_iter: T,
    minus_num: usize,
    plus_num: usize,
}

impl<'a, T: Iterator<Item = &'a str>> InfoIter<'a, T> {
    pub fn new(header: &[usize; 4], iter: T) -> InfoIter<'a, T> {
        InfoIter {
            line_iter: iter,
            minus_num: header[0] - 1,
            plus_num: header[2] - 1,
        }
    }
}

fn pre_increment(reference: &mut usize) -> usize {
    *reference += 1;
    *reference
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for InfoIter<'a, T> {
    type Item = Info<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.line_iter.next()?;
        if line.starts_with('-') {
            Some(Info {
                line,
                minus_num: pre_increment(&mut self.minus_num),
                plus_num: self.plus_num,
            })
        } else if line.starts_with('+') {
            Some(Info {
                line,
                minus_num: self.minus_num,
                plus_num: pre_increment(&mut self.plus_num),
            })
        } else {
            Some(Info {
                line,
                minus_num: pre_increment(&mut self.minus_num),
                plus_num: pre_increment(&mut self.plus_num),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hunk::info_iter::{Info, InfoIter};
    type MockInfo<'a> = (&'a str, usize, usize);

    fn split(line: &str) -> impl Iterator<Item = &str> + Clone {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| &line[i..j])
    }

    fn test<'a, T: Iterator<Item = &'a str> + Clone>(
        header: [usize; 4],
        lines: T,
        expected: Vec<MockInfo>,
    ) {
        let actual = InfoIter::new(&header, lines);
        for (act, exp) in actual.zip(expected.iter()) {
            let (line, minus_num, plus_num) = *exp;
            assert_eq!(
                act,
                Info {
                    line,
                    minus_num,
                    plus_num
                }
            );
        }
    }

    #[test]
    fn case_1() {
        let header: [usize; 4] = [1, 0, 1, 0];
        let line = "+ -+ +- -";
        let expected: Vec<MockInfo> = vec![
            ("+", 0, 1),
            (" ", 1, 2),
            ("-", 2, 2),
            ("+", 2, 3),
            (" ", 3, 4),
            ("+", 3, 5),
            ("-", 4, 5),
            (" ", 5, 6),
            ("-", 6, 6),
        ];
        test(header, split(line), expected);
    }

    #[test]
    fn case_2() {
        let header: [usize; 4] = [3, 0, 1, 0];
        let line = "+++- ";
        let expected: Vec<MockInfo> = vec![
            ("+", 2, 1),
            ("+", 2, 2),
            ("+", 2, 3),
            ("-", 3, 3),
            (" ", 4, 4),
        ];
        test(header, split(line), expected);
    }
}
