use serde::de::DeserializeOwned;

mod gt;

pub trait Provider: DeserializeOwned {
    fn ohlcv_data(&self) -> shared::Result<crate::ohlcv::OHLCVList>;
}

pub use self::gt::GeckoTerminal;
