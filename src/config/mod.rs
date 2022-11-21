use serde::Deserialize;
use std::thread;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_api_threads")]
    pub api_threads: usize,
}

fn default_host() -> String {
    "[::]:3000".into()
}

fn default_api_threads() -> usize {
    let cores: usize = thread::available_parallelism().unwrap().into();
    usize::max(1, cores - 1)
}
