use crate::model::messages::{MessageChannel, MessagePort};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::{OpenOrder, Order, OrderBook, State, Trade};

#[derive(Debug, Clone)]
pub struct ApiContext {
    matcher: Sender<MessagePort<OpenOrder, Order>>,
    state: Arc<RwLock<State>>,
}

impl ApiContext {
    pub fn new(matcher: Sender<MessagePort<OpenOrder, Order>>, state: Arc<RwLock<State>>) -> Self {
        Self { matcher, state }
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
        command: OpenOrder,
    ) -> Result<Order, Box<dyn Error + Send + Sync>> {
        let msg = MessageChannel::new(command);
        let order = msg.send_to(&self.matcher).await?;
        Ok(order)
    }
}
