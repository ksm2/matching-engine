use crate::model::{OrderBook, Trade};

#[derive(Debug)]
pub struct State {
    pub order_book: OrderBook,
    pub trades: Vec<Trade>,
}

impl State {
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
            trades: Vec::new(),
        }
    }

    pub fn push_trade(&mut self, trade: Trade) {
        self.order_book.last(trade.price);
        self.trades.push(trade);
    }
}
