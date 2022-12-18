use log::{debug, info};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use tokio::sync::watch::Sender;

use crate::config::Config;
use crate::model::{Market, MessagePort, OpenOrder, Order, OrderBook, OrderId, OrderType, State, Trade, WriteAheadLog};

#[derive(Debug)]
pub struct Matcher {
    rt: Arc<Runtime>,
    rx: Receiver<MessagePort<OpenOrder, Order>>,
    obx: Sender<OrderBook>,
    state: Arc<RwLock<State>>,
    wal: WriteAheadLog,
    market: Market,
}

impl Matcher {
    pub fn new(
        config: Config,
        rt: Arc<Runtime>,
        rx: Receiver<MessagePort<OpenOrder, Order>>,
        obx: Sender<OrderBook>,
        state: Arc<RwLock<State>>,
    ) -> Self {
        let wal = WriteAheadLog::new(&config.wal_location).expect("Expect wal to be initialized");
        let market = Market::new();

        Self {
            rt,
            rx,
            obx,
            state,
            wal,
            market,
        }
    }

    pub fn run(mut self) {
        let mut id = 0_u64;

        self.restore_state();

        info!("Matcher is listening for commands");
        while let Some(message) = self.rt.block_on(self.rx.recv()) {
            id += 1;

            debug!("Processing {:?}", message.req);

            let mut order = Order::open(
                OrderId(id),
                message.side,
                message.order_type,
                message.price,
                message.quantity,
            );
            self.save_command(&order);
            let ob = self.process(&mut order);
            self.obx.send(ob).unwrap();

            message.reply(order).unwrap();
        }

        info!("Matcher stopped listening for commands");
    }

    fn restore_state(&mut self) {
        let orders = self.wal.read_orders();
        for mut order in orders {
            self.process(&mut order);
        }
    }

    fn save_command(&mut self, order: &Order) {
        self.wal.append_order(order).expect("Order not stored");
    }

    fn process(&mut self, order: &mut Order) -> OrderBook {
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

        state.order_book.clone()
    }
}
