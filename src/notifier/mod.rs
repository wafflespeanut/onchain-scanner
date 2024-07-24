use super::ohlcv::OHLCVList;
use chrono::{offset::Utc, Datelike};

mod discord;

pub use self::discord::BufferedDiscordWebhook;

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
        msg.push_str("\n```\n");
        msg.push_str(&pair.pool_address);
        msg.push_str("\n");

        let mut yday_data = String::new();
        let is_first_three_day_open = Utc::now().ordinal() % 3 == 1;
        match ohlcv.clone().three_day().analyze() {
            Some(analysis) if is_first_three_day_open => {
                match analysis.bullish_engulfing.last() {
                    Some(b) if b.idx == analysis.ohlcv.len() - 1 => {
                        yday_data.push_str(&format!(
                            "\n3D Bullish engulfing ({} candles) at {}",
                            b.num_engulfing,
                            analysis.ohlcv.last().unwrap().close,
                        ));
                    }
                    _ => (),
                }
                match analysis.bearish_engulfing.last() {
                    Some(b) if b.idx == analysis.ohlcv.len() - 1 => {
                        yday_data.push_str(&format!(
                            "\n3D Bearish engulfing ({} candles) at {}",
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
                    yday_data.push_str(&format!("\nRange high {} broken", b.prev_bound));
                }
                if let Some(b) = yday.range_low_break {
                    yday_data.push_str(&format!("\nRange low {} broken", b.prev_bound));
                }
            }
        }

        if yday_data.is_empty() {
            log::warn!("empty data for pool {} {:?}", pair.pool_address, pair.token);
            return Ok(());
        }

        msg.push_str(&yday_data);
        msg.push_str("\n```");

        self.notify(&msg).await
    }
}
