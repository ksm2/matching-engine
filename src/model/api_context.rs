use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{Receiver, Sender as OneShotSender};
use tokio::sync::{RwLock, RwLockReadGuard};

use super::{OpenOrder, Order, OrderBook};

#[derive(Debug, Clone)]
pub struct ApiContext {
    tx: Sender<(OpenOrder, OneShotSender<Order>)>,
    order_book: Arc<RwLock<OrderBook>>,
}

impl ApiContext {
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
