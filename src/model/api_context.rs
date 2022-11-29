use crate::model::messages::{MessageChannel, MessagePort};
use hyper::Method;
use prometheus::proto::MetricFamily;
use prometheus::{HistogramVec, Registry};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::{OpenOrder, Order, OrderBook, State, Trade};

#[derive(Debug, Clone)]
pub struct ApiContext {
    registry: Registry,
    req_duration_histogram: HistogramVec,
    matcher: Sender<MessagePort<OpenOrder, Order>>,
    state: Arc<RwLock<State>>,
}

impl ApiContext {
    pub fn new(
        registry: Registry,
        req_duration_histogram: HistogramVec,
        matcher: Sender<MessagePort<OpenOrder, Order>>,
        state: Arc<RwLock<State>>,
    ) -> Self {
        Self {
            registry,
            req_duration_histogram,
            matcher,
            state,
        }
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

    pub fn observe_req_duration(&self, method: &Method, path: &str, duration: Duration) {
        self.req_duration_histogram
            .with_label_values(&[method.as_str(), path])
            .observe(duration.as_secs_f64());
    }

    pub fn gather_metrics(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
