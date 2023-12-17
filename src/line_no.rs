#[derive(Clone, Debug, PartialEq)]
pub struct LineNo {
    pub nums: [i64; 2],
}

impl LineNo {
    pub fn new(left: i64, right: i64) -> LineNo {
        LineNo {
            nums: [left, right],
        }
    }

    pub fn bump(&mut self, line_prefix: char) {
        match line_prefix {
            '-' => {
                self.nums[0] += 1;
            }
            '+' => {
                self.nums[1] += 1;
            }
            ' ' => {
                self.nums[0] += 1;
                self.nums[1] += 1;
            }
            _ => {}
        }
    }
}

impl From<&[i64; 4]> for LineNo {
    fn from(header: &[i64; 4]) -> LineNo {
        LineNo::new(header[0], header[2])
    }
}

impl From<[i64; 2]> for LineNo {
    fn from(nums: [i64; 2]) -> LineNo {
        LineNo { nums }
    }
}

impl Default for LineNo {
    fn default() -> LineNo {
        LineNo::new(0, 0)
    }
}
