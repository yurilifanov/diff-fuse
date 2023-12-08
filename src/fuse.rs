mod info;
mod info_chain;
mod info_iter;

use info::Info;
use info_chain::InfoChain;
use info_iter::InfoIter;

#[cfg(test)]
mod tests {
    use crate::fuse::{InfoChain, InfoIter};
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

        let mut offset = 0;

        let mut chain = InfoChain::new(
            left.into_iter().peekable(),
            right.into_iter().peekable(),
            &mut offset,
        )
        .unwrap();

        loop {
            match chain.peek().unwrap() {
                [None, None] => {
                    break;
                }
                _ => {
                    println!(
                        "{:?}",
                        [chain.next_left().unwrap(), chain.next_right()]
                    );
                }
            }
        }
    }
}
