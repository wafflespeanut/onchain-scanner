use serde_json::json;

const MAX_CHARS: usize = 2000;

pub struct DiscordWebhook {
    client: reqwest::Client,
    url: String,
}

impl DiscordWebhook {
    pub fn new(url: String) -> Self {
        DiscordWebhook {
            client: reqwest::Client::new(),
            url,
        }
    }
}

#[async_trait::async_trait]
impl super::Notifier for DiscordWebhook {
    async fn notify(&self, msg: &str) -> shared::Result<()> {
        let res = self
            .client
            .post(&self.url)
            .json(&json!({ "content": msg.chars().take(MAX_CHARS).collect::<String>() }))
            .send()
            .await?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(shared::Error::UnexpectedStatusCode(
                res.status().as_u16().into(),
                Some(res.text().await?),
            ))
        }
    }
}
