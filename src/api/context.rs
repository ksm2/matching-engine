use anyhow::Result;
use futures::{stream, Stream};
use hyper::Method;
use prometheus::proto::MetricFamily;
use prometheus::{HistogramOpts, HistogramVec, IntGauge, Registry};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::buckets::netflix_buckets;
use crate::model::{MessageChannel, MessagePort, OpenOrder, Order, OrderBook, State, Trade};

#[derive(Debug, Clone)]
pub struct Context {
    registry: Registry,
    req_duration_histogram: HistogramVec,
    connection_gauge: IntGauge,
    order_book_receiver: Receiver<OrderBook>,
    matcher: Sender<MessagePort<OpenOrder, Order>>,
    state: Arc<RwLock<State>>,
}

impl Context {
    pub fn new(
        registry: Registry,
        order_book_receiver: Receiver<OrderBook>,
        matcher: Sender<MessagePort<OpenOrder, Order>>,
        state: Arc<RwLock<State>>,
    ) -> Result<Self> {
        let req_duration_histogram = HistogramVec::new(
            HistogramOpts::new(
                "request_duration_seconds",
                "Duration of a request in seconds",
            )
            .buckets(netflix_buckets(1e3, 1e8)),
            &["method", "path"],
        )?;
        registry.register(Box::new(req_duration_histogram.clone()))?;

        let connection_gauge = IntGauge::new("connected_clients", "Number of connected clients")?;
        registry.register(Box::new(connection_gauge.clone()))?;

        Ok(Self {
            registry,
            req_duration_histogram,
            connection_gauge,
            order_book_receiver,
            matcher,
            state,
        })
    }

    pub async fn read_order_book(&self) -> RwLockReadGuard<OrderBook> {
        let state = self.state.read().await;
        RwLockReadGuard::map(state, |s| &s.order_book)
    }

    pub fn subscribe_order_book(&self) -> impl Stream<Item = OrderBook> + Send + 'static {
        stream::unfold(
            (true, self.order_book_receiver.clone()),
            |(first, mut receiver)| async move {
                if first {
                    let book = receiver.borrow().clone();
                    return Some((book, (false, receiver)));
                }

                if receiver.changed().await.is_ok() {
                    let book = receiver.borrow().clone();
                    Some((book, (false, receiver)))
                } else {
                    None
                }
            },
        )
    }

    pub async fn read_trades(&self) -> RwLockReadGuard<Vec<Trade>> {
        let state = self.state.read().await;
        RwLockReadGuard::map(state, |s| &s.trades)
    }

    pub async fn open_order(&self, command: OpenOrder) -> Result<Order, anyhow::Error> {
        let msg = MessageChannel::new(command);
        let order = msg.send_to(&self.matcher).await?;
        Ok(order)
    }

    pub fn observe_req_duration(&self, method: &Method, path: &str, duration: Duration) {
        self.req_duration_histogram
            .with_label_values(&[method.as_str(), path])
            .observe(duration.as_secs_f64());
    }

    pub fn inc_connections(&self) {
        self.connection_gauge.inc();
    }

    pub fn dec_connections(&self) {
        self.connection_gauge.dec();
    }

    pub fn gather_metrics(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
