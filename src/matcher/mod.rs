use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::model::{OpenOrder, OrderBook, PricePair, Side};

pub fn matcher(rt: &Runtime, mut rx: Receiver<OpenOrder>, ob: Arc<RwLock<OrderBook>>) {
    while let Some(message) = rt.block_on(rx.recv()) {
        println!("Processing {:?}", message);
        let mut ob = rt.block_on(ob.write());
        let pair = PricePair {
            price: message.price,
            quantity: message.quantity,
        };
        match message.side {
            Side::Buy => add_price_pair(&mut ob.bids, pair),
            Side::Sell => add_price_pair(&mut ob.asks, pair),
        }
    }
}

fn add_price_pair(prices: &mut Vec<PricePair>, price: PricePair) {
    for pair in prices.iter_mut() {
        if pair.price == price.price {
            pair.quantity += price.quantity;
            return;
        }
    }
    prices.push(price);
}
