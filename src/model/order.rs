use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::model::Side;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub side: Side,
    pub status: OrderStatus,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled: Decimal,
}

impl Order {
    pub fn open(side: Side, price: Decimal, quantity: Decimal) -> Self {
        let status = OrderStatus::Open;
        let filled = Decimal::ZERO;
        Self {
            side,
            status,
            price,
            quantity,
            filled,
        }
    }

    pub fn crosses(&self, other: &Self) -> bool {
        if self.side == other.side {
            return false;
        }

        match self.side {
            Side::Buy => self.price >= other.price,
            Side::Sell => self.price <= other.price,
        }
    }

    pub fn fill(&mut self, qty: Decimal) -> Decimal {
        let remaining = self.quantity - self.filled;
        let used = Decimal::min(qty, remaining);

        if used == remaining {
            self.status = OrderStatus::Filled;
            self.filled = self.quantity;
        } else {
            self.status = OrderStatus::PartiallyFilled;
            self.filled += used;
        }

        used
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.side != other.side {
            return None;
        }

        match self.side {
            Side::Buy => Some(self.price.cmp(&other.price)),
            Side::Sell => Some(other.price.cmp(&self.price)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_not_order_two_different_orders() {
        let o1 = Order::open(Side::Buy, dec!(12), dec!(500));
        let o2 = Order::open(Side::Sell, dec!(11), dec!(600));

        assert_eq!(o1.partial_cmp(&o2), None);
    }

    #[test]
    fn should_compare_two_bids() {
        let o1 = Order::open(Side::Buy, dec!(12), dec!(500));
        let o2 = Order::open(Side::Buy, dec!(11), dec!(600));

        assert!(o1.gt(&o2));
        assert!(o2.lt(&o1));
    }

    #[test]
    fn should_compare_two_asks() {
        let o1 = Order::open(Side::Sell, dec!(12), dec!(500));
        let o2 = Order::open(Side::Sell, dec!(11), dec!(600));

        assert!(o1.lt(&o2));
        assert!(o2.gt(&o1));
    }

    #[test]
    fn should_not_cross_order_of_same_side() {
        let o1 = Order::open(Side::Buy, dec!(12), dec!(500));
        let o2 = Order::open(Side::Buy, dec!(11), dec!(600));
        let o3 = Order::open(Side::Sell, dec!(12), dec!(500));
        let o4 = Order::open(Side::Sell, dec!(11), dec!(600));

        assert!(!o1.crosses(&o2));
        assert!(!o3.crosses(&o4));
    }

    #[test]
    fn should_cross_a_bid_with_an_equal_ask() {
        let o1 = Order::open(Side::Buy, dec!(12), dec!(500));
        let o2 = Order::open(Side::Sell, dec!(12), dec!(500));

        assert!(o1.crosses(&o2));
        assert!(o2.crosses(&o1));
    }

    #[test]
    fn should_cross_a_bid_with_a_lower_ask() {
        let o1 = Order::open(Side::Buy, dec!(12), dec!(500));
        let o2 = Order::open(Side::Sell, dec!(11), dec!(500));

        assert!(o1.crosses(&o2));
    }

    #[test]
    fn should_cross_an_ask_with_a_higher_bid() {
        let o1 = Order::open(Side::Sell, dec!(12), dec!(500));
        let o2 = Order::open(Side::Buy, dec!(15), dec!(500));

        assert!(o1.crosses(&o2));
    }

    #[test]
    fn should_be_partially_filled() {
        let mut o = Order::open(Side::Buy, dec!(42), dec!(500));

        let used = o.fill(dec!(200));
        assert_eq!(used, dec!(200));
        assert_eq!(o.filled, dec!(200));
        assert_eq!(o.status, OrderStatus::PartiallyFilled);
    }

    #[test]
    fn should_be_filled() {
        let mut o = Order::open(Side::Buy, dec!(42), dec!(200));

        let used = o.fill(dec!(500));
        assert_eq!(used, dec!(200));
        assert_eq!(o.filled, dec!(200));
        assert_eq!(o.status, OrderStatus::Filled);
    }
}
