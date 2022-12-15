use crate::model::{Order, OrderType, Side, Trade};
use log::debug;
use std::collections::BinaryHeap;
use std::ops::DerefMut;

#[derive(Debug)]
pub struct Market {
    bids: BinaryHeap<Order>,
    asks: BinaryHeap<Order>,
}

impl Market {
    pub fn new() -> Self {
        let bids = BinaryHeap::new();
        let asks = BinaryHeap::new();
        Self { bids, asks }
    }

    pub fn push(&mut self, order: &mut Order) -> Vec<Trade> {
        let trades = self.fill_order(order);
        if !order.is_filled() && order.order_type == OrderType::Limit {
            self.push_order(order.clone());
        }
        trades
    }

    fn fill_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let opposite_orders = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let mut trades = Vec::new();

        while !order.is_filled() {
            let filled = {
                let mut peek_other = match opposite_orders.peek_mut() {
                    None => break,
                    Some(o) => o,
                };
                let other = peek_other.deref_mut();

                if !order.can_be_filled_by(other) {
                    break;
                }

                let trade = Self::execute_trade(order, other);
                trades.push(trade);
                other.is_filled()
            };

            if filled {
                opposite_orders.pop();
            }
        }

        trades
    }

    fn execute_trade(order: &mut Order, other: &mut Order) -> Trade {
        let (buy_order_id, sell_order_id) = match order.side {
            Side::Buy => (order.id, other.id),
            Side::Sell => (other.id, order.id),
        };

        let used_qty = other.fill(order.unfilled());
        order.fill(used_qty);
        debug!("Filled bid at {}", other.price);

        Trade::new(other.price, used_qty, buy_order_id, sell_order_id)
    }

    fn push_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => self.bids.push(order),
            Side::Sell => self.asks.push(order),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{OrderId, OrderStatus, OrderType};
    use rust_decimal_macros::dec;

    #[test]
    fn should_create_new_bid() {
        let mut market = Market::new();

        let mut o = Order::open_limit(OrderId(1), Side::Buy, dec!(10), dec!(100));
        market.push(&mut o);

        assert_eq!(market.asks.len(), 0);
        assert_eq!(market.bids.len(), 1);
        let bid = market.bids.peek().unwrap();
        assert_eq!(bid.price, dec!(10));
        assert_eq!(bid.quantity, dec!(100));
    }

    #[test]
    fn should_create_new_ask() {
        let mut market = Market::new();

        let mut o = Order::open_limit(OrderId(1), Side::Sell, dec!(10), dec!(100));

        let trades = market.push(&mut o);
        assert!(trades.is_empty());

        assert_eq!(market.bids.len(), 0);
        assert_eq!(market.asks.len(), 1);

        let ask = market.asks.peek().unwrap();
        assert_eq!(ask, &o);
    }

    #[test]
    fn should_create_ask_and_fill_it_with_another_order() {
        let mut market = Market::new();

        let mut o = Order::open(
            OrderId(1),
            Side::Sell,
            OrderType::Limit,
            dec!(10),
            dec!(100),
        );
        market.push(&mut o);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        market.push(&mut o);

        assert_eq!(market.bids.len(), 0);
        assert_eq!(market.asks.len(), 0);
    }

    #[test]
    fn should_handle_partial_fill() {
        let mut matcher = Market::new();

        let mut o = Order::open_limit(OrderId(1), Side::Sell, dec!(10), dec!(100));
        matcher.push(&mut o);

        let mut o = Order::open_limit(OrderId(1), Side::Buy, dec!(10), dec!(145));
        matcher.push(&mut o);

        assert!(matcher.asks.is_empty());
        assert!(!matcher.bids.is_empty());
        let bid = matcher.bids.peek().unwrap().clone();
        assert_eq!(
            matcher.bids.into_vec(),
            vec![Order {
                id: OrderId(1),
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::PartiallyFilled,
                price: dec!(10),
                quantity: dec!(145),
                filled: dec!(100),
                created_at: bid.created_at,
            }],
        );
    }

    #[test]
    fn should_handle_market_order() {
        let mut market = Market::new();

        let mut o = Order::open_limit(OrderId(1), Side::Sell, dec!(10), dec!(100));
        market.push(&mut o);

        let mut o = Order::open_market(OrderId(1), Side::Buy, dec!(145));
        market.push(&mut o);

        assert!(market.asks.is_empty());
        assert!(market.bids.is_empty());
    }
}
