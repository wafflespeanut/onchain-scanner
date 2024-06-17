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

        let mut today_data = String::new();
        if let Some(today) = analysis.today_data() {
            if let Some(b) = today.range_high_break {
                today_data.push_str(&format!("\nRange high {} broken", b.prev_bound));
            }
            if let Some(b) = today.range_low_break {
                today_data.push_str(&format!("\nRange low {} broken", b.prev_bound));
            }
            if let Some(b) = today.bullish_engulfing {
                today_data.push_str(&format!(
                    "\nBullish engulfing ({} candles) at {}",
                    b.num_engulfing, today.ohlcv.close
                ));
            }
            if let Some(b) = today.bearish_engulfing {
                today_data.push_str(&format!(
                    "\nBearish engulfing ({} candles) at {}",
                    b.num_engulfing, today.ohlcv.close
                ));
            }
        }

        if today_data.is_empty() {
            return Ok(());
        }

        msg.push_str(&today_data);
        msg.push_str("\n```");

        self.notify(&msg).await
    }
}
