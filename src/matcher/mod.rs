use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::RwLock;

use crate::model::{OpenOrder, Order, OrderBook, OrderStatus, Side};

pub fn matcher(
    rt: &Runtime,
    mut rx: Receiver<(OpenOrder, Sender<Order>)>,
    ob: Arc<RwLock<OrderBook>>,
) {
    let mut matcher = Matcher::new(rt, ob);
    while let Some((message, sender)) = rt.block_on(rx.recv()) {
        println!("Processing {:?}", message);

        let mut order = Order::open(message.side, message.price, message.quantity);
        matcher.process(&mut order);
        sender.send(order).unwrap();
    }
}

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
        let mut qty = order.quantity;
        let mut ob = self.rt.block_on(self.ob.write());

        let opposite_orders = if order.side == Side::Buy {
            &mut self.asks
        } else {
            &mut self.bids
        };
        for other_order in opposite_orders.iter_mut() {
            if other_order.status != OrderStatus::Filled && order.crosses(other_order) {
                let used_qty = other_order.fill(qty);
                println!("Filled bid at {}", other_order.price);
                order.fill(used_qty);
                ob.trade(other_order.price, used_qty);

                println!("Taking ask of {} at {}", used_qty, other_order.price);
                ob.take(!order.side, other_order.price, used_qty);

                qty -= used_qty;
                if qty.is_zero() {
                    return;
                }
            }
        }

        if !qty.is_zero() {
            println!("Placing bid of {} at {}", qty, order.price);
            ob.place(order.side, order.price, qty);
            drop(ob);
            self.push_order(order.clone());
        }
    }

    fn push_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => self.bids.push(order),
            Side::Sell => self.asks.push(order),
        }
    }
}
