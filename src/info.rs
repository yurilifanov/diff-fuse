use crate::hand::Hand;
use core::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub rank: usize,
    pub hand: Hand,
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or(' ')
    }

    pub fn data(mut self) -> (Hand, String) {
        (self.hand, self.line)
    }
}

impl From<(&str, usize, Hand)> for Info {
    fn from(tuple: (&str, usize, Hand)) -> Info {
        Info {
            line: tuple.0.to_string(),
            rank: tuple.1,
            hand: tuple.2,
        }
    }
}

pub fn iter_info<T: Iterator<Item = (Hand, String)>>(
    header: &[usize; 4],
    mut iter: T,
) -> impl Iterator<Item = Info> {
    let (mut lrank, mut rrank) = (header[2], header[0]);
    std::iter::from_fn(move || -> Option<Info> {
        let (hand, line) = iter.next()?;
        let info = match hand {
            Hand::Left => Info {
                line,
                rank: lrank,
                hand,
            },
            Hand::Right => Info {
                line,
                rank: rrank,
                hand,
            },
        };
        if info.line.starts_with('-') {
            rrank += 1;
        } else if info.line.starts_with('+') {
            lrank += 1;
        } else {
            lrank += 1;
            rrank += 1;
        }
        Some(info)
    })
}

pub fn iter_info_X<T: Iterator<Item = (Hand, String)>>(
    header: &[usize; 4],
    h: Hand,
    mut iter: T,
) -> impl Iterator<Item = Info> {
    let (mut lrank, mut rrank) = (header[2], header[0]);
    std::iter::from_fn(move || -> Option<Info> {
        match h {
            Hand::Right => {
                let (hand, line) = iter.next()?;
                let info = Info {
                    line,
                    rank: lrank,
                    hand,
                };
                if info.line.starts_with('-') {
                    rrank += 1;
                } else if info.line.starts_with('+') {
                    lrank += 1;
                } else {
                    lrank += 1;
                    rrank += 1;
                }
                Some(info)
            }
            Hand::Left => {
                let (hand, line) = iter.next()?;
                let info = Info {
                    line,
                    rank: rrank,
                    hand,
                };
                if info.line.starts_with('-') {
                    rrank += 1;
                } else if info.line.starts_with('+') {
                    lrank += 1;
                } else {
                    lrank += 1;
                    rrank += 1;
                }
                Some(info)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hand::Hand;
    use crate::info::{iter_info, Info};

    fn split(line: &str) -> impl Iterator<Item = String> + '_ {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| line[i..j].to_string())
    }

    fn test(
        header: [usize; 4],
        data: &str,
        hand: Hand,
        expected: Vec<(&str, usize, Hand)>,
    ) {
        let actual =
            iter_info(&header, std::iter::repeat(hand).zip(split(data)));
        for (act, exp) in actual.zip(expected.into_iter()) {
            assert_eq!(act, Info::from(exp));
        }
    }

    #[test]
    fn case_1() {
        test(
            [1, 0, 1, 0],
            "+ -+ +- -",
            Hand::Left,
            vec![
                ("+", 1, Hand::Left),
                (" ", 2, Hand::Left),
                ("-", 3, Hand::Left),
                ("+", 3, Hand::Left),
                (" ", 4, Hand::Left),
                ("+", 5, Hand::Left),
                ("-", 6, Hand::Left),
                (" ", 6, Hand::Left),
                ("-", 7, Hand::Left),
            ],
        );
    }

    #[test]
    fn case_2() {
        test(
            [3, 0, 1, 0],
            "+++- ",
            Hand::Right,
            vec![
                ("+", 3, Hand::Right),
                ("+", 3, Hand::Right),
                ("+", 3, Hand::Right),
                ("-", 3, Hand::Right),
                (" ", 4, Hand::Right),
            ],
        );
    }
}
