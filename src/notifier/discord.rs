use async_std::sync::Mutex;
use serde_json::json;

use std::time::{Duration, Instant};

const MAX_CHARS: usize = 1990;

pub struct BufferedDiscordWebhook {
    client: reqwest::Client,
    url: String,
    buffer: Mutex<RateLimitedBuffer>,
}

struct RateLimitedBuffer {
    msg: String,
    reset: Duration,
    current: Instant,
}

impl BufferedDiscordWebhook {
    pub fn new(url: String) -> Self {
        BufferedDiscordWebhook {
            client: reqwest::Client::new(),
            url,
            buffer: Mutex::new(RateLimitedBuffer {
                msg: String::with_capacity(MAX_CHARS),
                reset: Duration::from_secs(0),
                current: Instant::now(),
            }),
        }
    }
}

#[async_trait::async_trait]
impl super::Notifier for BufferedDiscordWebhook {
    async fn notify(&self, msg: &str) -> shared::Result<()> {
        let msg = {
            let mut g = self.buffer.lock().await;
            g.msg.push_str(&msg);
            g.msg.push('\n');
            if g.msg.len() < MAX_CHARS || g.current.elapsed() < g.reset {
                log::debug!("buffering message until char limit");
                return Ok(());
            }

            let mut m = String::with_capacity(MAX_CHARS);
            let mut extra = String::new();
            let mut inside_code = false;
            for line in g.msg.split('\n') {
                let line = line.trim();
                if !extra.is_empty() || m.len() + line.len() > MAX_CHARS {
                    if inside_code && extra.is_empty() {
                        m.push_str("```");
                        extra.push_str("```");
                    }
                    extra.push_str(line);
                    extra.push('\n');
                    continue;
                }
                m.push_str(line);
                m.push('\n');
                if line.contains("```") {
                    inside_code = !inside_code;
                }
            }
            g.msg = extra;
            m
        };

        log::info!("POST {} (len: {})", self.url, msg.len());
        let res = self
            .client
            .post(&self.url)
            .json(&json!({ "content": msg.trim() }))
            .send()
            .await?;
        let reset = res
            .headers()
            .get("X-RateLimit-Reset-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok())
            .map(|f| Duration::from_secs(f.ceil() as u64));
        // Rate-limit headers don't seem to work accurately in discord's API, so we resort to 1 request per rate limit window
        // (as returned by X-RateLimit-Reset-After or Retry-After or 1 second if none helps)
        {
            let mut g = self.buffer.lock().await;
            g.reset = reset.unwrap_or(Duration::from_secs(1));
            g.current = Instant::now();
            if res.status().as_u16() == 429 {
                let reset: Option<Duration> = res
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<f64>().ok())
                    .map(|f| Duration::from_secs(f.ceil() as u64));
                log::warn!(
                    "rate limited, retrying after {:?}",
                    reset
                );
                g.reset = reset.unwrap_or(Duration::from_secs(1));
                g.msg = msg + "\n" + &g.msg;
            }
        }

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
