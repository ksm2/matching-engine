use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use crate::model::{OpenOrder, Order, OrderBook, OrderStatus, Side};

pub fn matcher(rt: &Runtime, mut rx: Receiver<OpenOrder>, ob: Arc<RwLock<OrderBook>>) {
    let mut bids = Vec::<Order>::new();
    let mut asks = Vec::<Order>::new();
    while let Some(message) = rt.block_on(rx.recv()) {
        println!("Processing {:?}", message);

        let mut ob = rt.block_on(ob.write());
        let order = Order::open(message.side, message.price, message.quantity);
        match message.side {
            Side::Buy => {
                let mut qty = order.quantity;
                'find_order: for ask in asks.iter_mut() {
                    if ask.status != OrderStatus::Filled && order.crosses(ask) {
                        let used_qty = ask.fill(qty);
                        println!("Filled bid at {}", ask.price);
                        println!("Taking ask of {} at {}", used_qty, ask.price);
                        ob.take_ask(ask.price, used_qty);

                        qty -= used_qty;
                        if qty.is_zero() {
                            break 'find_order;
                        }
                    }
                }
                if !qty.is_zero() {
                    println!("Placing bid of {} at {}", qty, order.price);
                    ob.place_bid(order.price, qty);
                    bids.push(order);
                }
            }
            Side::Sell => {
                let mut qty = order.quantity;
                'find_order: for bid in bids.iter_mut() {
                    if bid.status != OrderStatus::Filled && order.crosses(bid) {
                        let used_qty = bid.fill(qty);
                        println!("Filled ask at {}", bid.price);
                        println!("Taking bid of {} at {}", used_qty, bid.price);
                        ob.take_bid(bid.price, used_qty);

                        qty -= used_qty;
                        if qty.is_zero() {
                            break 'find_order;
                        }
                    }
                }
                if !qty.is_zero() {
                    println!("Placing ask of {} at {}", qty, order.price);
                    ob.place_ask(order.price, qty);
                    asks.push(order);
                }
            }
        }
    }
}
