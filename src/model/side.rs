use serde::{Deserialize, Serialize};
use std::ops::Not;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_invert_side() {
        assert_eq!(!Side::Buy, Side::Sell);
        assert_eq!(!Side::Sell, Side::Buy);
    }
}
