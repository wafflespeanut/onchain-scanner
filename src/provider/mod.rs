use chrono::{offset::Utc, DateTime};
use serde::de::DeserializeOwned;

mod gt;

pub struct OHLC {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub trait Provider: DeserializeOwned {
    fn ohlc_data(&self) -> shared::Result<Vec<OHLC>>;
}

pub use self::gt::GeckoTerminal;
