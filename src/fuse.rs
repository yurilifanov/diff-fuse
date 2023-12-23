pub mod core;
pub mod fuse_iter;
mod info_chain;
pub mod info_iter;
pub mod info_source;
pub mod line;
mod line_counter;

#[cfg(test)]
mod tests {
    use crate::fuse::info_chain::InfoChain;
    use crate::fuse::info_source::InfoSource;
    use crate::hunk::Hunk;

    #[test]
    fn case_1() {
        let left: Vec<_> = vec![
            "\
@@ -1,1 +1,1 @@
-1
+2",
            "\
@@ -2,1 +2,1 @@
-2
+3",
        ]
        .into_iter()
        .map(|s| Hunk::from_lines(&mut s.lines().peekable()).unwrap())
        .collect();

        let right: Vec<_> = vec![
            "\
@@ -1,3 +1,3 @@
 1
 2
 3",
        ]
        .into_iter()
        .map(|s| Hunk::from_lines(&mut s.lines().peekable()).unwrap())
        .collect();

        let mut liter = left.into_iter().peekable();
        let mut riter = right.into_iter().peekable();

        let mut chain = InfoChain::new(&mut liter, &mut riter).unwrap();

        loop {
            match chain.peek() {
                [None, None] => {
                    break;
                }
                _ => {
                    println!("{:?}", [chain.next_left(), chain.next_right()]);
                }
            }
        }
    }
}
