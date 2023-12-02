use core::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub struct Info {
    pub line: String,
    pub rank: usize,
}

impl Info {
    pub fn prefix(&self) -> char {
        self.line.chars().nth(0).unwrap_or(' ')
    }

    pub fn cmp(&self, other: &Info) -> Ordering {
        // FIXME
        Ordering::Less
    }
}

impl From<(&str, usize)> for Info {
    fn from(tuple: (&str, usize)) -> Info {
        Info {
            line: tuple.0.to_string(),
            rank: tuple.1,
        }
    }
}

pub enum InfoType {
    Minus(),
    Plus(),
}

pub fn iter_info<T: Iterator<Item = String>>(
    header: &[usize; 4],
    mut iter: T,
    info_type: InfoType,
) -> impl Iterator<Item = Info> {
    let (prefix, mut rank) = match info_type {
        InfoType::Minus() => ("-", header[2]),
        InfoType::Plus() => ("+", header[0]),
    };
    std::iter::from_fn(move || -> Option<Info> {
        let line = iter.next()?;
        if line.starts_with(prefix) {
            Some(Info { line, rank })
        } else {
            let result = Some(Info { line, rank });
            rank += 1;
            result
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hunk::iter_info::{iter_info, Info, InfoType};
    fn split(line: &str) -> impl Iterator<Item = String> + '_ {
        line.char_indices()
            .zip(line.char_indices().skip(1).chain(Some((line.len(), ' '))))
            .map(move |((i, _), (j, _))| line[i..j].to_string())
    }

    fn test(
        header: [usize; 4],
        data: &str,
        info_type: InfoType,
        expected: Vec<(&str, usize)>,
    ) {
        let actual = iter_info(&header, split(data), info_type);
        for (act, exp) in actual.zip(expected.into_iter()) {
            assert_eq!(act, Info::from(exp));
        }
    }

    #[test]
    fn case_1() {
        test(
            [1, 0, 1, 0],
            "+ -+ +- -",
            InfoType::Minus(),
            vec![
                ("+", 1),
                (" ", 2),
                ("-", 3),
                ("+", 3),
                (" ", 4),
                ("+", 5),
                ("-", 6),
                (" ", 6),
                ("-", 7),
            ],
        );
    }

    #[test]
    fn case_2() {
        test(
            [3, 0, 1, 0],
            "+++- ",
            InfoType::Plus(),
            vec![("+", 3), ("+", 3), ("+", 3), ("-", 3), (" ", 4)],
        );
    }
}
