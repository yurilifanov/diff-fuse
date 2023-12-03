use core::cmp::Ordering;

// Handedness would be more correct, but that's too long
#[derive(Clone, Debug, PartialEq)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    pub fn cmp(&self, other: &Hand) -> Ordering {
        match [self, other] {
            [Hand::Left, Hand::Right] => Ordering::Less,
            [Hand::Right, Hand::Left] => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::Hand;
    use core::cmp::Ordering;

    #[test]
    fn test_order() {
        assert_eq!(Ordering::Less, Hand::Left.cmp(&Hand::Right));
        assert_eq!(Ordering::Greater, Hand::Right.cmp(&Hand::Left));
    }
}
