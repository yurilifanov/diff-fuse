use crate::fuse::line::Line;

pub trait InfoSource {
    fn peek(&mut self) -> [Option<&Line>; 2];
    fn next_right(&mut self) -> Option<Line>;
    fn next_left(&mut self) -> Option<Line>;
}
