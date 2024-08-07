use serde::{Deserialize, Serialize};

mod emitted {
    include!(concat!(env!("OUT_DIR"), "/build.rs"));
}
mod error;

pub use self::emitted::{AWS_REGIONS, IGNORED_POOLS};
pub use self::error::{Error, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct Request {
    pub network: String,
    pub pool_address: String,
    pub token: Option<(String, String)>,
    #[serde(skip)]
    pub mc_or_fdv: Option<f64>,
    #[serde(skip)]
    pub maybe_duplicate: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: Option<u16>,
    pub body: Option<String>,
    pub err: Option<String>,
}
