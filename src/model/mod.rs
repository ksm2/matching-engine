use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ops::Not;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{Receiver, Sender as OneShotSender};
use tokio::sync::{RwLock, RwLockReadGuard};

pub use order::{Order, OrderStatus};
pub use order_book::{OrderBook, PricePair};

mod order;
mod order_book;

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

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContext {
    tx: Sender<(OpenOrder, OneShotSender<Order>)>,
    order_book: Arc<RwLock<OrderBook>>,
}

impl AppContext {
    pub fn new(
        tx: Sender<(OpenOrder, OneShotSender<Order>)>,
        order_book: Arc<RwLock<OrderBook>>,
    ) -> Self {
        Self { tx, order_book }
    }

    pub async fn read_order_book(&self) -> RwLockReadGuard<OrderBook> {
        self.order_book.read().await
    }

    pub async fn open_order(
        &self,
        order: OpenOrder,
    ) -> Result<Receiver<Order>, Box<dyn Error + Send + Sync>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send((order, tx)).await?;
        Ok(rx)
    }
}
