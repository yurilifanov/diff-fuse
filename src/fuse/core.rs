use crate::fuse::info_source::InfoSource;
use crate::fuse::line::Line;
use crate::fuse::line_counter::LineCounter;

use crate::error::MergeErr;
use crate::hunk::{Header, Hunk};
use crate::macros::merge_err;

use core::cmp::Ordering;

pub fn fuse<T: InfoSource>(
    mut header: Header,
    source: T,
) -> Result<Hunk, MergeErr> {
    let mut counter = LineCounter::default();
    let mut data: Vec<((i64, i64), Line)> = Vec::new();
    let mut drain = Drain::<T> { source };

    while let Some(item) = drain.next() {
        match item? {
            FuseItem::Single(info) => {
                data.push((counter.update(&info)?, info));
            }
            FuseItem::Pair(linfo, rinfo) => {
                data.push((counter.update(&linfo)?, linfo));
                data.push((counter.update(&rinfo)?, rinfo));
            }
            FuseItem::None => {}
        }
    }

    counter.update_header(&mut header);

    Ok(Hunk::new(
        header,
        sort(data)?.into_iter().map(|(_, info)| info.line).collect(),
    ))
}

enum FuseItem {
    None,
    Single(Line),
    Pair(Line, Line),
}

type DrainItem = Option<Result<FuseItem, MergeErr>>;

struct Drain<T: InfoSource> {
    source: T,
}

impl<T: InfoSource> Drain<T> {
    pub fn next(&mut self) -> DrainItem {
        match self.source.peek() {
            [None, None] => None,
            [None, Some(_)] => self.take_right(),
            [Some(_), None] => self.take_left(),
            [Some(linfo), Some(rinfo)] => match linfo.rank.cmp(&rinfo.rank) {
                Ordering::Less => self.take_left(),
                Ordering::Greater => self.take_right(),
                _ => {
                    let index = [linfo.prefix(), rinfo.prefix()];
                    self.choose(index)
                }
            },
        }
    }

    fn choose(&mut self, index: [char; 2]) -> DrainItem {
        match index {
            [' ', ' '] => self.skip_take_right(),
            ['+', ' '] => self.skip_take_left(),
            ['-', ' '] => self.take_left(),

            [' ', '+'] => self.take_right(),
            ['+', '+'] => self.take_right(),
            ['-', '+'] => {
                let left = self.source.next_left()?;
                let mut right = self.source.next_right()?;
                if left.line[1..] == right.line[1..] {
                    right.line.replace_range(0..1, " ");
                    Some(Ok(FuseItem::Single(right)))
                } else {
                    Some(Ok(FuseItem::Pair(left, right)))
                }
            }

            [' ', '-'] => self.skip_take_right(),
            ['+', '-'] => self.skip(),
            ['-', '-'] => self.take_left(),

            _ => {
                let left = self.source.next_left()?;
                let right = self.source.next_right()?;
                Some(Err(merge_err!(
                    "Unexpected prefixes on lines -- '{:?}' and '{:?}'",
                    left,
                    right
                )))
            }
        }
    }

    fn take_left(&mut self) -> DrainItem {
        Self::take(self.source.next_left())
    }

    fn take_right(&mut self) -> DrainItem {
        Self::take(self.source.next_right())
    }

    fn skip(&mut self) -> DrainItem {
        let left = self.source.next_left()?;
        let right = self.source.next_right()?;
        if left.line[1..] == right.line[1..] {
            Some(Ok(FuseItem::None))
        } else {
            Some(Err(merge_err!(
                "skip: Mismatch between lines -- '{left:?}' and '{right:?}'"
            )))
        }
    }

    fn skip_take_left(&mut self) -> DrainItem {
        let left = self.source.next_left()?;
        let right = self.source.next_right()?;
        if left.line[1..] == right.line[1..] {
            Self::take(Some(left))
        } else {
            Some(Err(merge_err!(
                "skip_take: Mismatch between lines -- '{:?}' and '{:?}'",
                left,
                right
            )))
        }
    }

    fn skip_take_right(&mut self) -> DrainItem {
        let left = self.source.next_left()?;
        let right = self.source.next_right()?;
        if left.line[1..] == right.line[1..] {
            Self::take(Some(right))
        } else {
            Some(Err(merge_err!(
                "skip_take: Mismatch between lines -- '{:?}' and '{:?}'",
                left,
                right
            )))
        }
    }

    fn take(info: Option<Line>) -> DrainItem {
        Some(Ok(FuseItem::Single(info?)))
    }
}

fn sort(
    mut data: Vec<((i64, i64), Line)>,
) -> Result<Vec<((i64, i64), Line)>, MergeErr> {
    let mut err: Option<MergeErr> = None;
    let mut update_err = |e: MergeErr| {
        if err.is_none() {
            err = Some(e);
        }
    };

    // each group can only have one line with prefix ' ', so order:
    // - within each group
    //     - ' ' line, if any, first
    //     - '-' lines, if any, follow
    //     - '+' lines, if any, last
    // - keep the order of lines according to their index
    // - keep the group order according to the group index
    data.sort_unstable_by(
        |((lhs_group, lhs_index), linfo), ((rhs_group, rhs_index), rinfo)| {
            if lhs_group != rhs_group {
                return lhs_group.cmp(rhs_group);
            }

            let lhs_prefix = if let Some(val) = linfo.line.chars().nth(0) {
                val
            } else {
                update_err(merge_err!("Empty line in sort"));
                ' '
            };

            let rhs_prefix = if let Some(val) = rinfo.line.chars().nth(0) {
                val
            } else {
                update_err(merge_err!("Empty line in sort"));
                ' '
            };

            if lhs_prefix == rhs_prefix {
                return lhs_index.cmp(rhs_index);
            }

            match (lhs_prefix, rhs_prefix) {
                ('+', '-') => Ordering::Greater,
                ('-', '+') => Ordering::Less,
                (' ', _) => Ordering::Less,
                (_, ' ') => Ordering::Greater,
                _ => {
                    update_err(merge_err!(
                        "sort: Unexpected line prefixes: {}, {}",
                        lhs_prefix,
                        rhs_prefix
                    ));
                    Ordering::Equal
                }
            }
        },
    );

    err.map(Err).unwrap_or(Ok(data))
}
