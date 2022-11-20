use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::model::{OpenOrder, OrderBook, PricePair};

pub fn matcher(rt: &Runtime, mut rx: Receiver<OpenOrder>, ob: Arc<RwLock<OrderBook>>) {
    while let Some(message) = rt.block_on(rx.recv()) {
        println!("Processing {:?}", message);
        let mut ob = rt.block_on(ob.write());
        let pair = PricePair::new(message.price, message.quantity);
        ob.bid_or_ask(message.side, pair);
    }
}
