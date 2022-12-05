use anyhow::{bail, Result};
use clap::{crate_version, Parser};
use log::info;
use prometheus::{HistogramOpts, HistogramVec, IntGauge, Registry};
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {}

fn main() -> Result<()> {
    // Parse CLI arguments
    Cli::parse();

    // Read environment variables from .env
    dotenv::dotenv().ok();

    // Initialize logger from environment
    env_logger::init();

    info!("Matching engine started");
    info!("Version: {}", crate_version!());

    let registry = Registry::new();

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

    // Parse config from environment
    let config = match envy::prefixed("APP_").from_env::<Config>() {
        Ok(config) => config,
        Err(e) => bail!("Failed to parse config: {}", e),
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
        connection_gauge,
        order_sender,
        state.clone(),
    );
    let handle = rt.spawn(api::api(config, context));

    // Run the matcher
    matcher::matcher(&rt, order_receiver, state);
    rt.block_on(handle)?;

    info!("Matching engine stopped");
    Ok(())
}
