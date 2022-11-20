use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::model::{OpenOrder, Order, OrderBook, OrderId, Side};

pub fn matcher(
    rt: &Runtime,
    mut rx: Receiver<(OpenOrder, Sender<Order>)>,
    ob: Arc<RwLock<OrderBook>>,
) {
    let mut id = 0_u64;
    let mut matcher = Matcher::new(rt, ob);
    while let Some((message, sender)) = rt.block_on(rx.recv()) {
        id += 1;
        println!("Processing {:?}", message);

        let mut order = Order::open(OrderId(id), message.side, message.price, message.quantity);
        matcher.process(&mut order);
        matcher.remove_filled_orders(order.side);
        sender.send(order).unwrap();
    }
}

#[derive(Debug)]
struct Matcher<'a> {
    rt: &'a Runtime,
    ob: Arc<RwLock<OrderBook>>,
    bids: Vec<Order>,
    asks: Vec<Order>,
}

impl<'a> Matcher<'a> {
    pub fn new(rt: &'a Runtime, ob: Arc<RwLock<OrderBook>>) -> Self {
        Self {
            rt,
            ob,
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn process(&mut self, order: &mut Order) {
        let mut ob = self.rt.block_on(self.ob.write());

        let opposite_orders = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        for other_order in opposite_orders.iter_mut() {
            if !order.crosses(other_order) {
                continue;
            }

            Matcher::execute_trade(&mut ob, order, other_order);
            if order.is_filled() {
                return;
            }
        }

        if !order.is_filled() {
            println!("Placing order of {} at {}", order.unfilled(), order.price);
            ob.place(order.side, order.price, order.unfilled());
            drop(ob);
            self.push_order(order.clone());
        }
    }

    fn execute_trade(ob: &mut RwLockWriteGuard<OrderBook>, order: &mut Order, other: &mut Order) {
        let (buy_order_id, sell_order_id) = match order.side {
            Side::Buy => (order.id, other.id),
            Side::Sell => (other.id, order.id),
        };

        let used_qty = other.fill(order.unfilled());
        order.fill(used_qty);
        println!("Filled bid at {}", other.price);

        ob.trade(other.price, used_qty, buy_order_id, sell_order_id);

        ob.take(!order.side, other.price, used_qty);
        println!("Taking ask of {} at {}", used_qty, other.price);
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
