#[derive(Clone, Debug, PartialEq)]
pub struct LineNo {
    pub nums: [usize; 2],
}

impl LineNo {
    pub fn new(left: usize, right: usize) -> LineNo {
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

impl From<&[usize; 4]> for LineNo {
    fn from(header: &[usize; 4]) -> LineNo {
        LineNo::new(header[0], header[2])
    }
}

impl From<[usize; 2]> for LineNo {
    fn from(nums: [usize; 2]) -> LineNo {
        LineNo { nums }
    }
}

impl Default for LineNo {
    fn default() -> LineNo {
        LineNo::new(0, 0)
    }
}
