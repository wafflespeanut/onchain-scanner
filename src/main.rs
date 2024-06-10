#![recursion_limit = "512"]

pub mod feed;
pub mod host;
pub mod notifier;
pub mod ohlcv;
pub mod provider;
pub mod runner;
pub mod storage;

fn main() {
    fast_log::init(fast_log::Config::new().console().chan_len(None)).expect("initializing logger");
}
