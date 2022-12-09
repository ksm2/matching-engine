use log::{debug, info};
use std::sync::Arc;
use rust_decimal_macros::dec;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::model::{MessagePort, OpenOrder, Order, OrderId, OrderType, Side, State, Trade};

pub fn matcher(
    rt: &Runtime,
    mut rx: Receiver<MessagePort<OpenOrder, Order>>,
    ob: Arc<RwLock<State>>,
) {
    let mut id = 0_u64;
    let mut matcher = Matcher::new(rt, ob);

    info!("Matcher is listening for commands");
    while let Some(message) = rt.block_on(rx.recv()) {
        id += 1;

        debug!("Processing {:?}", message.req);

        let mut order = Order::open(
            OrderId(id),
            message.side,
            message.order_type,
            message.price,
            message.quantity,
        );
        matcher.process(&mut order);
        matcher.remove_filled_orders(order.side);

        message.reply(order).unwrap();
    }

    info!("Matcher stopped listening for commands");
}

#[derive(Debug)]
struct Matcher<'a> {
    rt: &'a Runtime,
    state: Arc<RwLock<State>>,
    bids: Vec<Order>,
    asks: Vec<Order>,
}

impl<'a> Matcher<'a> {
    pub fn new(rt: &'a Runtime, state: Arc<RwLock<State>>) -> Self {
        Self {
            rt,
            state,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn process(&mut self, order: &mut Order) {
        let mut state = self.rt.block_on(self.state.write());

        let opposite_orders = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        for other_order in opposite_orders.iter_mut() {
            if order.order_type == OrderType::Limit && !order.crosses(other_order) {
                continue;
            }

            Matcher::execute_trade(&mut state, order, other_order);
            if order.is_filled() {
                return;
            }
        }

        if order.order_type == OrderType::Limit && !order.is_filled() {
            debug!("Placing order of {} at {}", order.unfilled(), order.price);
            state
                .order_book
                .place(order.side, order.price, order.unfilled());
            drop(state);

            let mut new_order = order.clone();
            new_order.quantity = order.unfilled();
            self.push_order(new_order);
        }
    }

    fn execute_trade(state: &mut RwLockWriteGuard<State>, order: &mut Order, other: &mut Order) {
        let (buy_order_id, sell_order_id) = match order.side {
            Side::Buy => (order.id, other.id),
            Side::Sell => (other.id, order.id),
        };

        let used_qty = other.fill(order.unfilled());
        order.fill(used_qty);
        debug!("Filled bid at {}", other.price);

        let trade = Trade::new(other.price, other.quantity, buy_order_id, sell_order_id);
        state.push_trade(trade);

        state.order_book.take(!order.side, other.price, used_qty);
        debug!("Taking ask of {} at {}", used_qty, other.price);
    }

    fn push_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => self.bids.push(order),
            Side::Sell => self.asks.push(order),
        }
    }

    pub fn remove_filled_orders(&mut self, side: Side) {
        let orders = match side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };
        orders.retain(|order| !order.is_filled());
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{PricePair, OrderStatus};

    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_create_new_bid() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, ob);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Sell);

        assert_eq!(matcher.asks.len(), 0);
        assert_eq!(matcher.bids.len(), 1);
        assert_eq!(matcher.bids[0].price, dec!(10));
        assert_eq!(matcher.bids[0].quantity, dec!(100));
    }

    #[test]
    fn should_create_new_ask() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, ob);

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Buy);

        assert_eq!(matcher.bids.len(), 0);
        assert_eq!(matcher.asks.len(), 1);
        assert_eq!(matcher.asks[0].price, dec!(10));
        assert_eq!(matcher.asks[0].quantity, dec!(100));
    }

    #[test]
    fn should_create_ask_and_fill_it_with_another_order() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, ob);

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Sell);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Buy);

        assert_eq!(matcher.bids.len(), 0);
        assert_eq!(matcher.asks.len(), 0);
    }

    #[test]
    fn should_handle_partial_fill() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Sell);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(145));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Buy);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![PricePair::new(dec!(10), dec!(45))]);

        assert_eq!(matcher.asks, vec![]);
        assert_eq!(matcher.bids, vec![
            Order{
                id: OrderId(1),
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::PartiallyFilled,
                price: dec!(10),
                quantity: dec!(45),
                filled: dec!(100)
            }
        ]);
    }

    #[test]
    fn should_handle_market_order() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Sell);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Market, dec!(10), dec!(145));
        matcher.process(&mut o);
        matcher.remove_filled_orders(Side::Buy);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![]);

        assert_eq!(matcher.asks, vec![]);
        assert_eq!(matcher.bids, vec![]);
    }

}
