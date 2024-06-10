use super::ohlcv::Analysis;

mod discord;

#[async_trait::async_trait]
pub trait Notifier {
    async fn notify(&self, msg: &str) -> shared::Result<()>;

    async fn post_analysis(&self, analysis: Analysis) -> shared::Result<()> {
        let mut msg = String::new();

        self.notify(&msg).await
    }
}
