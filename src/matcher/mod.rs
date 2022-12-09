use log::{debug, info};
use std::collections::BinaryHeap;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::model::{MessagePort, OpenOrder, Order, OrderId, Side, State, Trade, WriteAheadLog};

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

        let mut order = Order::open(OrderId(id), message.side, message.price, message.quantity);
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
