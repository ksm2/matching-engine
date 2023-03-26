use crate::model::{Order, OrderBookSide, OrderType, Side, Trade};

#[derive(Debug)]
pub struct Market {
    bids: OrderBookSide,
    asks: OrderBookSide,
}

impl Market {
    pub fn new() -> Self {
        let bids = OrderBookSide::new(true);
        let asks = OrderBookSide::new(false);
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
        let opposite_side = self.side_mut(!order.side);
        opposite_side.fill(order)
    }

    fn push_order(&mut self, order: Order) {
        let order_side = self.side_mut(order.side);
        order_side.push(order);
    }

    fn side_mut(&mut self, side: Side) -> &mut OrderBookSide {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
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
