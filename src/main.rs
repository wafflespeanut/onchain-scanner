#![recursion_limit = "512"]

pub mod feed;
pub mod host;
pub mod http;
pub mod notifier;
pub mod ohlcv;
pub mod provider;
pub mod runner;
pub mod storage;

use std::env;

use self::http::Handler;
use self::runner::{Config, Runner};

#[tokio::main]
async fn main() {
    let c: Config = env::var("CONFIG")
        .ok()
        .and_then(|f| {
            let f = std::fs::File::open(f).expect("opening config file");
            serde_json::from_reader(f).expect("parsing config file")
        })
        .expect("config env not set");

    fast_log::init(
        fast_log::Config::new()
            .level(if env::var("DEBUG").ok().is_some() {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .console()
            .chan_len(None),
    )
    .expect("initializing logger");

    let runner = Runner::new(c).expect("configuring runner");
    let storage = runner.storage.clone();
    tokio::task::spawn(runner.run());
    _ = &*self::http::AUTH_KEY;
    Handler::serve(&env::var("ADDR").expect("address unset"), storage).await;
}
