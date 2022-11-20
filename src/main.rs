use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::model::OrderBook;

mod api;
mod matcher;
mod model;

fn main() -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(10)
        .build()?;

    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let order_book = Arc::new(RwLock::new(OrderBook::new()));

    let context = model::AppContext::new(tx, order_book.clone());
    let handle = rt.spawn(api::api(context));

    matcher::matcher(&rt, rx, order_book);
    rt.block_on(handle)?;

    Ok(())
}
