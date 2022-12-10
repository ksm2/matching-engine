use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::side::Side;

use super::OrderType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled: Decimal,
    pub created_at: u128,
}

impl Order {
    pub fn open(
        id: OrderId,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let status = OrderStatus::Open;
        let filled = Decimal::ZERO;
        let now = SystemTime::now();
        let created_at = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Self {
            id,
            side,
            order_type,
            status,
            price,
            quantity,
            filled,
            created_at,
        }
    }

    pub fn unfilled(&self) -> Decimal {
        self.quantity - self.filled
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

    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.side != other.side {
            return None;
        }

        Some(self.cmp(other))
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.created_at.cmp(&other.created_at),
            ordering => match self.side {
                Side::Buy => ordering,
                // the reverse ordering is used to construct min heap for sell orders
                Side::Sell => ordering.reverse(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::{thread, time};

    #[test]
    fn should_not_order_two_different_orders() {
        let o1 = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(12), dec!(500));
        let o2 = Order::open(
            OrderId(2),
            Side::Sell,
            OrderType::Limit,
            dec!(11),
            dec!(600),
        );

        assert_eq!(o1.partial_cmp(&o2), None);
    }

    #[test]
    fn should_compare_two_bids() {
        let o1 = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(12), dec!(500));
        let o2 = Order::open(OrderId(2), Side::Buy, OrderType::Limit, dec!(11), dec!(600));

        assert!(o1.gt(&o2));
        assert!(o2.lt(&o1));
    }

    #[test]
    fn should_compare_two_asks() {
        let o1 = Order::open(
            OrderId(1),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(500),
        );
        let o2 = Order::open(
            OrderId(2),
            Side::Sell,
            OrderType::Limit,
            dec!(11),
            dec!(600),
        );

        assert!(o1.lt(&o2));
        assert!(o2.gt(&o1));
    }

    #[test]
    fn should_compare_two_asks_with_different_creation_time() {
        let o1 = Order::open(
            OrderId(1),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(600),
        );
        let one_ms = time::Duration::from_millis(1);
        thread::sleep(one_ms);
        let o2 = Order::open(
            OrderId(2),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(600),
        );

        assert!(o1.lt(&o2));
        assert!(o2.gt(&o1));
    }

    #[test]
    fn should_not_cross_order_of_same_side() {
        let o1 = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(12), dec!(500));
        let o2 = Order::open(OrderId(2), Side::Buy, OrderType::Limit, dec!(11), dec!(600));
        let o3 = Order::open(
            OrderId(3),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(500),
        );
        let o4 = Order::open(
            OrderId(4),
            Side::Sell,
            OrderType::Limit,
            dec!(11),
            dec!(600),
        );

        assert!(!o1.crosses(&o2));
        assert!(!o3.crosses(&o4));
    }

    #[test]
    fn should_cross_a_bid_with_an_equal_ask() {
        let o1 = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(12), dec!(500));
        let o2 = Order::open(
            OrderId(2),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(500),
        );

        assert!(o1.crosses(&o2));
        assert!(o2.crosses(&o1));
    }

    #[test]
    fn should_cross_a_bid_with_a_lower_ask() {
        let o1 = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(12), dec!(500));
        let o2 = Order::open(
            OrderId(2),
            Side::Sell,
            OrderType::Limit,
            dec!(11),
            dec!(500),
        );

        assert!(o1.crosses(&o2));
    }

    #[test]
    fn should_cross_an_ask_with_a_higher_bid() {
        let o1 = Order::open(
            OrderId(1),
            Side::Sell,
            OrderType::Limit,
            dec!(12),
            dec!(500),
        );
        let o2 = Order::open(OrderId(2), Side::Buy, OrderType::Limit, dec!(15), dec!(500));

        assert!(o1.crosses(&o2));
    }

    #[test]
    fn should_be_partially_filled() {
        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(42), dec!(500));

        let used = o.fill(dec!(200));
        assert_eq!(used, dec!(200));
        assert_eq!(o.filled, dec!(200));
        assert_eq!(o.status, OrderStatus::PartiallyFilled);
    }

    #[test]
    fn should_be_filled() {
        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(42), dec!(200));

        let used = o.fill(dec!(500));
        assert_eq!(used, dec!(200));
        assert_eq!(o.filled, dec!(200));
        assert_eq!(o.status, OrderStatus::Filled);
    }
}
