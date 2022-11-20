use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ops::Neg;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};

pub use order::{Order, OrderStatus};
pub use order_book::OrderBook;

mod order;
mod order_book;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricePair {
    pub price: Decimal,
    pub quantity: Decimal,
}

impl PricePair {
    pub fn new(price: Decimal, quantity: Decimal) -> Self {
        Self { price, quantity }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenOrder {
    pub quantity: Decimal,
    pub price: Decimal,
    pub side: Side,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Neg for Side {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContext {
    tx: Sender<OpenOrder>,
    order_book: Arc<RwLock<OrderBook>>,
}

impl AppContext {
    pub fn new(tx: Sender<OpenOrder>, order_book: Arc<RwLock<OrderBook>>) -> Self {
        Self { tx, order_book }
    }

    pub async fn read_order_book(&self) -> RwLockReadGuard<OrderBook> {
        self.order_book.read().await
    }

    pub async fn open_order(&self, order: OpenOrder) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.tx.send(order).await?;
        Ok(())
    }
}
