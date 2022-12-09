use serde::Deserialize;
use std::path::PathBuf;
use std::thread;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_api_threads")]
    pub api_threads: usize,
    #[serde(default = "default_wal_location")]
    pub wal_location: PathBuf,
}

fn default_host() -> String {
    "[::]:3000".into()
}

fn default_api_threads() -> usize {
    let cores: usize = thread::available_parallelism().unwrap().into();
    usize::max(1, cores - 1)
}

fn default_wal_location() -> PathBuf {
    "./log".into()
}
