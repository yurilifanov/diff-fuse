use crate::error::MergeError;
use crate::fuse::line::Line;
use crate::hunk::Header;
use crate::macros::merge_err;

pub struct LineCounter {
    num_added: i64,
    num_removed: i64,
    total_added: i64,
    total_removed: i64,
    total_unchanged: i64,
}

impl Default for LineCounter {
    fn default() -> LineCounter {
        LineCounter {
            num_added: 0,
            num_removed: 0,
            total_added: 0,
            total_removed: 0,
            total_unchanged: 0,
        }
    }
}

impl LineCounter {
    pub fn update(&mut self, info: &Line) -> Result<(i64, i64), MergeError> {
        match info.prefix() {
            '-' => {
                self.num_removed += 1;
                self.total_removed += 1;
                Ok((self.total_unchanged, self.num_removed))
            }
            '+' => {
                self.num_added += 1;
                self.total_added += 1;
                Ok((self.total_unchanged, self.num_added))
            }
            ' ' => {
                self.num_added = 0;
                self.num_removed = 0;
                self.total_unchanged += 1;
                Ok((self.total_unchanged, 1))
            }
            prefix => Err(merge_err!(
                "LineCounter: Unexpected line prefix {}",
                prefix
            )),
        }
    }

    pub fn update_header(&self, header: &mut Header) {
        header.fields[1] = self.total_removed + self.total_unchanged;
        header.fields[3] = self.total_added + self.total_unchanged;

        match [header.fields[1], header.fields[3]] {
            [0, 0] => {
                header.fields[0] = 0;
                header.fields[2] = 0;
            }
            [_, 0] => {
                header.fields[2] = header.fields[0] - 1;
            }
            [0, _] => {
                header.fields[0] = header.fields[2] - 1;
            }
            _ => {}
        }
    }
}
