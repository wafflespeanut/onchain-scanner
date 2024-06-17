use super::ohlcv::Analysis;

mod discord;

pub use self::discord::BufferedDiscordWebhook;

#[async_trait::async_trait]
pub trait Notifier {
    async fn notify(&self, msg: &str) -> shared::Result<()>;

    async fn post_analysis(
        &self,
        pair: &shared::Request,
        analysis: Analysis,
    ) -> shared::Result<()> {
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
        if let Some(yday) = analysis.last_day_data() {
            if let Some(b) = yday.range_high_break {
                yday_data.push_str(&format!("\nRange high {} broken", b.prev_bound));
            }
            if let Some(b) = yday.range_low_break {
                yday_data.push_str(&format!("\nRange low {} broken", b.prev_bound));
            }
            if let Some(b) = yday.bullish_engulfing {
                yday_data.push_str(&format!(
                    "\nBullish engulfing ({} candles) at {}",
                    b.num_engulfing, yday.ohlcv.close
                ));
            }
            if let Some(b) = yday.bearish_engulfing {
                yday_data.push_str(&format!(
                    "\nBearish engulfing ({} candles) at {}",
                    b.num_engulfing, yday.ohlcv.close
                ));
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
