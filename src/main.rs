#![feature(iter_intersperse)]

use log::{error, info};
use prometheus::{HistogramOpts, HistogramVec, Registry};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::model::{ApiContext, State};
use crate::utils::netflix_buckets;

mod api;
mod config;
mod matcher;
mod model;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    // Read environment variables from .env
    dotenv::dotenv().ok();

    // Initialize logger from environment
    env_logger::init();

    let req_duration_histogram = HistogramVec::new(
        HistogramOpts::new(
            "request_duration_seconds",
            "Duration of a request in seconds",
        )
        .buckets(netflix_buckets(1e3, 1e9)),
        &["method", "path"],
    )?;
    let registry = Registry::new();
    registry.register(Box::new(req_duration_histogram.clone()))?;

    // Parse config from environment
    let config = match envy::prefixed("APP_").from_env::<Config>() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return Ok(());
        }
    };

    // Create async runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.api_threads)
        .build()?;
    info!("Starting {} API threads", config.api_threads);

    // Initialize matching engine state:
    // - State: Our data structure which holds the order book and trades
    // - RwLock: A lock which allows many parallel reads or one write at a time
    // - Arc: Allows different scopes to hold a reference to the lock
    let state = Arc::new(RwLock::new(State::new()));

    // Initialize the order command message channel
    let (order_sender, order_receiver) = tokio::sync::mpsc::channel(32);

    // Spawn async API threads
    let context = ApiContext::new(
        registry,
        req_duration_histogram,
        order_sender,
        state.clone(),
    );
    let handle = rt.spawn(api::api(config, context));

    // Run the matcher
    matcher::matcher(&rt, order_receiver, state);
    rt.block_on(handle)?;

    Ok(())
}
