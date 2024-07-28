use super::ohlcv::OHLCVList;
use chrono::{offset::Utc, Datelike};

use std::env;

mod discord;

pub use self::discord::BufferedDiscordWebhook;

lazy_static::lazy_static! {
    static ref THREE_DAY: bool = env::var("DAY3").is_ok();
}

#[async_trait::async_trait]
pub trait Notifier {
    async fn notify(&self, msg: &str) -> shared::Result<()>;

    async fn post_analysis(&self, pair: &shared::Request, ohlcv: OHLCVList) -> shared::Result<()> {
        let mut msg = String::new();
        msg.push_str("### ");
        msg.push_str(
            &pair
                .token
                .as_ref()
                .map(|(base, quote)| format!(" {}/{}", base, quote))
                .unwrap_or_default(),
        );
        msg.push_str(&format!("\n`{}`", pair.pool_address));

        let mut yday_data = String::new();
        let is_first_three_day_open = Utc::now().ordinal() % 3 == 1 || *THREE_DAY;
        match ohlcv.clone().three_day().analyze() {
            Some(analysis) if is_first_three_day_open => {
                match analysis.bullish_engulfing.last() {
                    Some(b) if b.idx == analysis.ohlcv.len() - 1 => {
                        yday_data.push_str(&format!(
                            "\n`3D Bullish engulfing ({} candles) at {}`",
                            b.num_engulfing,
                            analysis.ohlcv.last().unwrap().close,
                        ));
                    }
                    _ => (),
                }
                match analysis.bearish_engulfing.last() {
                    Some(b) if b.idx == analysis.ohlcv.len() - 1 => {
                        yday_data.push_str(&format!(
                            "\n`3D Bearish engulfing ({} candles) at {}`",
                            b.num_engulfing,
                            analysis.ohlcv.last().unwrap().close,
                        ));
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        if let Some(analysis) = ohlcv.analyze() {
            if let Some(yday) = analysis.last_day_data() {
                if let Some(b) = yday.range_high_break {
                    yday_data.push_str(&format!("\n`Range high {} broken`", b.prev_bound));
                }
                if let Some(b) = yday.range_low_break {
                    yday_data.push_str(&format!("\n`Range low {} broken`", b.prev_bound));
                }
            }
        }

        if yday_data.is_empty() {
            log::warn!("analysis insufficient for pool {} {:?}", pair.pool_address, pair.token);
            return Ok(());
        }

        msg.push_str(&yday_data);
        msg.push('\n');

        self.notify(&msg).await
    }

    async fn flush(&self) -> shared::Result<()> {
        self.notify("").await
    }
}
