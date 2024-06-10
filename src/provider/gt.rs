use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct OHLCVList {
    ohlcv_list: Vec<(i64, f64, f64, f64, f64, f64)>,
}

#[derive(Deserialize, Debug)]
struct RespData {
    attributes: OHLCVList,
}

#[derive(Deserialize, Debug)]
struct RespError {
    error_code: u16,
    error_message: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GeckoTerminal {
    Success { data: RespData },
    Failure { status: RespError },
}

impl super::Provider for GeckoTerminal {
    fn ohlcv_data(&self) -> shared::Result<crate::ohlcv::OHLCVList> {
        match self {
            GeckoTerminal::Success { data } => data
                .attributes
                .ohlcv_list
                .iter()
                .map(|&(t, o, h, l, c, v)| {
                    Ok(crate::ohlcv::OHLCV {
                        timestamp: DateTime::from_timestamp(t, 0)
                            .ok_or(shared::Error::InvalidTimestamp(t))?,
                        open: o,
                        high: h,
                        low: l,
                        close: c,
                        volume: v,
                    })
                })
                .collect::<shared::Result<Vec<_>>>()
                .map(Into::into),
            GeckoTerminal::Failure { status } => Err(shared::Error::UnexpectedStatusCode(
                status.error_code as i32,
                Some(status.error_message.clone()),
            )),
        }
    }
}
