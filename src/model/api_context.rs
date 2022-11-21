use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{Receiver, Sender as OneShotSender};
use tokio::sync::{RwLock, RwLockReadGuard};

use super::{OpenOrder, Order, OrderBook, State, Trade};

#[derive(Debug, Clone)]
pub struct ApiContext {
    tx: Sender<(OpenOrder, OneShotSender<Order>)>,
    state: Arc<RwLock<State>>,
}

impl ApiContext {
    pub fn new(tx: Sender<(OpenOrder, OneShotSender<Order>)>, state: Arc<RwLock<State>>) -> Self {
        Self { tx, state }
    }

    pub async fn read_order_book(&self) -> RwLockReadGuard<OrderBook> {
        let state = self.state.read().await;
        RwLockReadGuard::map(state, |s| &s.order_book)
    }

    pub async fn read_trades(&self) -> RwLockReadGuard<Vec<Trade>> {
        let state = self.state.read().await;
        RwLockReadGuard::map(state, |s| &s.trades)
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
