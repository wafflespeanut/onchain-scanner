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
        // lock and block for the whole thing
        let mut g: async_std::sync::MutexGuard<RateLimitedBuffer> = self.buffer.lock().await;
        if !msg.trim().is_empty() {
            g.msg.push_str(&msg);
            g.msg.push('\n');
        }
        log::debug!("current buffer len: {}", g.msg.len());
        if (g.msg.len() < MAX_CHARS || g.current.elapsed() < g.reset) && g.current.elapsed() < Duration::from_secs(5) {
            log::debug!("buffering message until char limit");
            return Ok(());
        }

        let mut msg = String::with_capacity(MAX_CHARS);
        let mut extra = String::new();
        for line in g.msg.split('\n') {
            let line = line.trim();
            if !extra.is_empty() || msg.len() + line.len() > MAX_CHARS {
                extra.push_str(line);
                extra.push('\n');
                continue;
            }
            msg.push_str(line);
            msg.push('\n');
        }
        g.msg = extra;

        if msg.trim().is_empty() {
            log::debug!("empty message, skipping");
            return Ok(());
        }

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
        g.reset = reset.unwrap_or(Duration::from_secs(1));
        g.current = Instant::now();
        if res.status().as_u16() == 429 {
            let reset: Option<Duration> = res
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok())
                .map(|f| Duration::from_secs(f.ceil() as u64));
            log::warn!("rate limited, retrying after {:?}", reset);
            g.reset = reset.unwrap_or(Duration::from_secs(1));
            g.msg = msg + "\n" + &g.msg;
        }
        log::info!(
            "remaining buffer len: {} (reset in {:?})",
            g.msg.len(),
            g.reset
        );

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
