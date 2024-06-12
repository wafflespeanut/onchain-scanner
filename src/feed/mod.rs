use async_std::sync::Mutex;
use serde::de::DeserializeOwned;

use std::marker::PhantomData;
use std::time::{Duration, Instant};

mod cm;
mod gt;

#[async_trait::async_trait]
pub trait FeedClient {
    async fn fetch_addresses(
        &self,
        network: Network,
        page: u16,
    ) -> Result<Vec<Pair>, shared::Error>;
}

pub type CoinMarketCap = DefaultClient<self::cm::CoinMarketCap>;
pub type GeckoTerminal = DefaultClient<self::gt::GeckoTerminal>;

#[derive(Clone, Copy, strum_macros::Display, strum_macros::VariantArray)]
pub enum Network {
    // serialized as geckoterminal's network names
    #[strum(serialize = "solana")]
    Solana,
    #[strum(serialize = "eth")]
    Ethereum,
    #[strum(serialize = "base")]
    Base,
    #[strum(serialize = "blast")]
    Blast,
    #[strum(serialize = "ton")]
    TON,
    #[strum(serialize = "bsc")]
    BSC,
    #[strum(serialize = "arbitrum")]
    Arbitrum,
    #[strum(serialize = "avax")]
    Avalanche,
    #[strum(serialize = "optimism")]
    Optimism,
    #[strum(serialize = "ftm")]
    Fantom,
    #[strum(serialize = "metis")]
    Metis,
    #[strum(serialize = "ronin")]
    Ronin,
}

pub struct Pair {
    pub base_token: String,
    pub quote_token: String,
    pub contract_address: String,
}

pub trait Feed {
    const DELAY: Duration;

    const MAX_PAGES: Option<u16> = None;

    type Response: DeserializeOwned + Into<Vec<Pair>>;

    fn modify(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req
    }

    fn url(network: Network, page: u16) -> String;
}

pub struct DefaultClient<F> {
    client: reqwest::Client,
    last_request_time: Mutex<Instant>,
    _mark: PhantomData<F>,
}

impl<F> Default for DefaultClient<F> {
    fn default() -> Self {
        DefaultClient {
            client: reqwest::Client::new(),
            last_request_time: Mutex::new(Instant::now()),
            _mark: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<F> FeedClient for DefaultClient<F>
where
    F: Feed + Send + Sync + 'static,
{
    async fn fetch_addresses(
        &self,
        network: Network,
        page: u16,
    ) -> Result<Vec<Pair>, shared::Error> {
        {
            // Avoid DoS'ing API and getting banned
            let mut time = self.last_request_time.lock().await;
            let elapsed = time.elapsed();
            if elapsed < F::DELAY {
                async_std::task::sleep(F::DELAY - elapsed).await;
            }
            *time = Instant::now();
        }

        if let Some(max_pages) = F::MAX_PAGES {
            if page > max_pages {
                log::debug!("Max pages {} reached for feed", max_pages);
                return Ok(vec![]);
            }
        }

        let url = F::url(network, page);
        log::info!("GET {}", url);
        match F::modify(self.client.get(&url))
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15")
            .header("Cache-Control", "no-cache")
            .send().await {
            Ok(resp) => {
                let code = resp.status();
                let bytes = resp.bytes().await?;
                if !code.is_success() {
                    log::warn!("reveived non-200 status code: {}", code);
                    return Err(shared::Error::UnexpectedStatusCode(code.as_u16(), Some(
                        String::from_utf8_lossy(&*bytes).to_string())));
                }
                let r: F::Response = serde_json::from_slice(&*bytes).map_err(|_| {
                    shared::Error::UnexpectedResponse(String::from_utf8_lossy(&*bytes).to_string())
                })?;
                Ok(r.into())
            }
            Err(e) => Err(shared::Error::Http(e.into())),
        }
    }
}
