use log::{debug, info};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::model::{MessagePort, OpenOrder, Order, OrderId, Side, State, Trade};

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

        let mut order = Order::open(OrderId(id), message.side, message.price, message.quantity);
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
            if !order.crosses(other_order) {
                continue;
            }

            Matcher::execute_trade(&mut state, order, other_order);
            if order.is_filled() {
                return;
            }
        }

        if !order.is_filled() {
            debug!("Placing order of {} at {}", order.unfilled(), order.price);
            state
                .order_book
                .place(order.side, order.price, order.unfilled());
            drop(state);
            self.push_order(order.clone());
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
