use log::{debug, info};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::model::{
    Market, MessagePort, OpenOrder, Order, OrderId, OrderType, State, Trade, WriteAheadLog,
};

pub fn matcher(
    config: Config,
    rt: Arc<Runtime>,
    mut rx: Receiver<MessagePort<OpenOrder, Order>>,
    ob: Arc<RwLock<State>>,
) {
    let mut id = 0_u64;
    let mut matcher = Matcher::new(config, rt.clone(), ob);

    matcher.restore_state();

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
        matcher.save_command(&order);
        matcher.process(&mut order);

        message.reply(order).unwrap();
    }

    info!("Matcher stopped listening for commands");
}

#[derive(Debug)]
struct Matcher {
    rt: Arc<Runtime>,
    wal: WriteAheadLog,
    state: Arc<RwLock<State>>,
    market: Market,
}

impl Matcher {
    pub fn new(config: Config, rt: Arc<Runtime>, state: Arc<RwLock<State>>) -> Self {
        let wal = WriteAheadLog::new(&config.wal_location).expect("Expect wal to be initialized");

        Self {
            rt,
            wal,
            state,
            market: Market::new(),
        }
    }

    pub fn restore_state(&mut self) {
        let orders = self.wal.read_orders();
        for mut order in orders {
            self.process(&mut order);
        }
    }

    pub fn save_command(&mut self, order: &Order) {
        self.wal.append_order(order).expect("Order not stored");
    }

    pub fn process(&mut self, order: &mut Order) {
        let mut state = self.rt.block_on(self.state.write());

        let trades = self.market.push(order);

        for trade in trades {
            let Trade {
                price, quantity, ..
            } = trade;
            state.order_book.take(!order.side, price, quantity);
            state.push_trade(trade);
            debug!("Taking liquidity of {} at {}", quantity, price);
        }

        if order.order_type == OrderType::Limit && !order.is_filled() {
            debug!("Placing order of {} at {}", order.unfilled(), order.price);
            state
                .order_book
                .place(order.side, order.price, order.unfilled());
        }
    }
}
