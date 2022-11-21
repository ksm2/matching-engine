use log::{error, info};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::model::{ApiContext, State};

mod api;
mod config;
mod matcher;
mod model;

fn main() -> Result<(), Box<dyn Error>> {
    // Read environment variables from .env
    dotenv::dotenv().ok();

    // Initialize logger from environment
    env_logger::init();

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

    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let state = Arc::new(RwLock::new(State::new()));

    // Spawn async API threads
    let context = ApiContext::new(tx, state.clone());
    let handle = rt.spawn(api::api(config, context));

    // Run the matcher
    matcher::matcher(&rt, rx, state);
    rt.block_on(handle)?;

    Ok(())
}
