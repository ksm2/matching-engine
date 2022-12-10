use log::{debug, info};
use std::collections::BinaryHeap;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::model::{MessagePort, OpenOrder, Order, OrderId, OrderType, Side, State, Trade, WriteAheadLog};

pub fn matcher(
    rt: &Runtime,
    mut rx: Receiver<MessagePort<OpenOrder, Order>>,
    ob: Arc<RwLock<State>>,
) {
    let mut id = 0_u64;
    let mut matcher = Matcher::new(rt, ob);

    matcher.restore_state().expect("Failed to restore state");

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

        matcher.save_command(&order);

        message.reply(order).unwrap();
    }

    info!("Matcher stopped listening for commands");
}

#[derive(Debug)]
struct Matcher<'a> {
    rt: &'a Runtime,
    wal: WriteAheadLog,
    state: Arc<RwLock<State>>,
    bids: BinaryHeap<Order>,
    asks: BinaryHeap<Order>,
}

impl<'a> Matcher<'a> {
    pub fn new(rt: &'a Runtime, state: Arc<RwLock<State>>) -> Self {
        let wal = WriteAheadLog::new(&String::from("./log")).expect("Expect wal to be initialized");

        Self {
            rt,
            wal,
            state,
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
        }
    }

    pub fn restore_state(&mut self) -> anyhow::Result<()> {
        let orders = self.wal.read_file()?;
        for order in orders {
            self.process(&mut order.clone());
        }

        Ok(())
    }

    pub fn save_command(&mut self, order: &Order) {
        self.wal.append_order(order).expect("Order not stored");
    }

    pub fn process(&mut self, order: &mut Order) {
        match order.order_type {
            OrderType::Limit => self.process_limit_order(order),
            OrderType::Market => self.process_market_order(order),
        }
    }

    fn process_limit_order(&mut self, order: &mut Order) {
        let mut state = self.rt.block_on(self.state.write());

        let opposite_orders = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        while !order.is_filled() {
            let filled = {
                let mut peek_other = match opposite_orders.peek_mut() {
                    None => break,
                    Some(o) => o,
                };
                let other = peek_other.deref_mut();

                if !other.crosses(order) {
                    break;
                }

                Matcher::execute_trade(&mut state, order, other);
                other.is_filled()
            };

            if filled {
                opposite_orders.pop();
            }
        }

        if !order.is_filled() {
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

    fn process_market_order(&mut self, order: &mut Order) {
        let mut state = self.rt.block_on(self.state.write());

        let opposite_orders = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        while !order.is_filled() {
            let filled = {
                let mut peek_other = match opposite_orders.peek_mut() {
                    None => break,
                    Some(o) => o,
                };
                let other = peek_other.deref_mut();

                if !other.crosses(order) {
                    break;
                }

                Matcher::execute_trade(&mut state, order, other);
                other.is_filled()
            };

            if filled {
                opposite_orders.pop();
            }
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

        let trade = Trade::new(other.price, used_qty, buy_order_id, sell_order_id);
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
}

#[cfg(test)]
mod tests {
    use crate::model::{PricePair, OrderStatus};

    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_create_new_bid_on_limit_order() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![PricePair::new(dec!(10), dec!(100))]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![
            Order{
                id: OrderId(1),
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::Open,
                price: dec!(10),
                quantity: dec!(100),
                filled: dec!(0),
                created_at: o.created_at
            }
        ]);
    }

    #[test]
    fn should_create_new_ask_on_limit_order() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.bids, vec![]);
        assert_eq!(new_state.order_book.asks, vec![PricePair::new(dec!(10), dec!(100))]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![
            Order{
                id: OrderId(1),
                side: Side::Sell,
                order_type: OrderType::Limit,
                status: OrderStatus::Open,
                price: dec!(10),
                quantity: dec!(100),
                filled: dec!(0),
                created_at: o.created_at
            }
        ]);
    }

    #[test]
    fn should_create_new_order_if_no_cross() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let mut o = Order::open(OrderId(2), Side::Sell, OrderType::Limit, dec!(11), dec!(100));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![PricePair::new(dec!(11), dec!(100))]);
        assert_eq!(new_state.order_book.bids, vec![PricePair::new(dec!(10), dec!(100))]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![
            Order{
                id: OrderId(2),
                side: Side::Sell,
                order_type: OrderType::Limit,
                status: OrderStatus::Open,
                price: dec!(11),
                quantity: dec!(100),
                filled: dec!(0),
                created_at: o.created_at
            }
        ]);
        let bids_vec = matcher.bids.into_sorted_vec();
        assert_eq!(bids_vec, vec![
            Order{
                id: OrderId(1),
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::Open,
                price: dec!(10),
                quantity: dec!(100),
                filled: dec!(0),
                created_at: bids_vec[0].created_at
            }
        ]);
    }

    #[test]
    fn should_create_ask_and_fill_it_with_another_order() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![]);
    }

    #[test]
    fn should_handle_limit_order_partial_fill() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Limit, dec!(10), dec!(145));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![PricePair::new(dec!(10), dec!(45))]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![
            Order{
                id: OrderId(1),
                side: Side::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::PartiallyFilled,
                price: dec!(10),
                quantity: dec!(45),
                filled: dec!(100),
                created_at: o.created_at
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

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Market, dec!(10), dec!(100));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![]);
    }

    #[test]
    fn should_handle_partial_market_order_higher_available_liquidity() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Market, dec!(10), dec!(145));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![]);
        assert_eq!(new_state.order_book.bids, vec![]);
        assert_eq!(matcher.asks.into_sorted_vec(), vec![]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![]);
    }

    #[test]
    fn should_handle_market_order_when_enough_liquidity() {
        let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        let ob = Arc::new(RwLock::new(State::new()));
        let mut matcher = Matcher::new(&rt, Arc::clone(&ob));

        let mut o = Order::open(OrderId(1), Side::Sell, OrderType::Limit, dec!(10), dec!(100));
        matcher.process(&mut o);

        let mut o = Order::open(OrderId(1), Side::Buy, OrderType::Market, dec!(10), dec!(45));
        matcher.process(&mut o);

        let new_state = rt.block_on(ob.read());
        assert_eq!(new_state.order_book.asks, vec![PricePair::new(dec!(10), dec!(55))]);
        assert_eq!(new_state.order_book.bids, vec![]);
        let bids_vec = matcher.asks.into_sorted_vec();
        assert_eq!(bids_vec, vec![Order{
            id: OrderId(1),
            side: Side::Sell,
            order_type: OrderType::Limit,
            status: OrderStatus::PartiallyFilled,
            price: dec!(10),
            quantity: dec!(100),
            filled: dec!(45),
            created_at: bids_vec[0].created_at
        }]);
        assert_eq!(matcher.bids.into_sorted_vec(), vec![]);
    }

}
