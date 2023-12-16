use crate::fuse::info::Info;

pub trait InfoSource {
    fn peek(&mut self) -> [Option<&Info>; 2];
    fn next_right(&mut self) -> Option<Info>;
    fn next_left(&mut self) -> Option<Info>;
}
