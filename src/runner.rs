use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{offset::Utc, TimeDelta};
use futures::future;
use serde::Deserialize;
use strum::VariantArray;

use super::storage::Storage;

use super::{
    feed::{FeedClient, Network},
    host::Host,
    notifier::Notifier,
    provider::Provider,
};

const ONE_MIN_FIVE_SECS: Duration = Duration::from_secs(65);
const fn default_min_liquidity() -> u64 {
    1000
}

#[derive(Deserialize)]
pub struct Config {
    pub lambda_function: String,
    pub storage_path: String,
    #[serde(default)]
    pub max_pages: Option<u16>,
    #[serde(default)]
    pub max_attempts_per_pair: Option<u16>,
    pub host_requests_per_min: u8,
    #[serde(default)]
    pub discord_url_network: HashMap<String, String>,
    #[serde(default)]
    pub post_once: bool,
    #[serde(default)]
    pub post_now: bool,
    #[serde(default = "default_min_liquidity")]
    pub min_liquidity: u64,
}

pub struct Runner<P, N> {
    pub storage: Storage,
    feeds: Vec<Box<dyn FeedClient + Send + Sync + 'static>>,
    hosts: Vec<Box<dyn Host<P> + Send + Sync + 'static>>,
    config: Config,
    notifier: HashMap<String, Arc<N>>,
    buffer: Vec<shared::Request>,
    pools: HashSet<String>,
    ended_feeds: Vec<bool>,
    pairs: HashSet<String>,
    current: Instant,
}

impl Runner<super::provider::GeckoTerminal, super::notifier::BufferedDiscordWebhook> {
    pub fn new(c: Config) -> shared::Result<Self> {
        Ok(Runner {
            feeds: vec![
                Box::new(super::feed::CoinMarketCap::default()) as Box<_>,
                Box::new(super::feed::GeckoTerminalTop::default()) as Box<_>,
                Box::new(super::feed::GeckoTerminalTrending::default()) as Box<_>,
            ],
            hosts: vec![Box::new(super::host::AwsLambda::new(&c.lambda_function)?)],
            storage: super::storage::Storage::new(&c.storage_path).expect("init storage"),
            notifier: Network::VARIANTS
                .iter()
                .filter_map(|n| {
                    let url = c.discord_url_network.get(&n.to_string())?;
                    Some((
                        n.to_string(),
                        Arc::new(super::notifier::BufferedDiscordWebhook::new(url.clone())),
                    ))
                })
                .collect(),
            buffer: Vec::with_capacity(c.host_requests_per_min as usize),
            config: c,
            pools: HashSet::with_capacity(1000),
            pairs: HashSet::with_capacity(1000),
            ended_feeds: vec![],
            // add 1-min so that it doesn't block on first attempt
            current: Instant::now()
                .checked_sub(ONE_MIN_FIVE_SECS)
                .expect("simple time math fail"),
        })
    }
}

impl<P, N> Runner<P, N>
where
    P: Provider + Send + Sync + 'static,
    N: Notifier + Send + Sync + 'static,
{
    pub async fn run(mut self) {
        let mut current_page = 1;
        let mut current_network_idx = 0;
        let mut posted_once = !self.config.post_now;
        self.ended_feeds = vec![false; self.feeds.len()];

        let batch_len = self.config.host_requests_per_min as usize
            * self.hosts.iter().map(|h| h.bulk_size()).sum::<usize>();
        let networks = self
            .notifier
            .keys()
            .filter_map(|s| {
                Network::VARIANTS
                    .iter()
                    .find(|n| n.to_string() == s.as_str())
                    .cloned()
            })
            .collect::<Vec<_>>();
        if networks.is_empty() {
            log::error!("no networks enabled, exiting runner...");
            return;
        }
        log::info!("enabled networks: {:?}", networks);

        loop {
            if current_network_idx == networks.len() {
                posted_once = true;
                current_network_idx = 0;
            }

            if self.config.post_once && posted_once {
                log::info!("exiting after first run");
                return;
            }

            if posted_once {
                self.block_until_midnight().await;
                posted_once = false;
            }

            let network = networks[current_network_idx];
            match self.populate_pairs(network, current_page).await {
                Ok(true) => (),
                Ok(false) => {
                    current_page = 1;
                    current_network_idx += 1;
                    self.pools.clear();
                    self.ended_feeds = vec![false; self.feeds.len()];
                    continue;
                }
                Err(e) => {
                    log::error!("failed to populate pairs: {}", e);
                    continue;
                }
            }

            while self.buffer.len() >= batch_len {
                log::info!(
                    "reached batch size {} for {}, flushing",
                    self.buffer.len(),
                    network
                );
                self.flush().await;
            }

            current_page += 1;
            if let Some(max) = self.config.max_pages {
                if current_page > max {
                    log::info!("reached max pages for network: {}", network);
                    current_page = 1;
                    current_network_idx += 1;
                    self.pairs.clear();
                }
            }
        }
    }

    pub fn spawn_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let notifiers = self.notifier.clone();
        tokio::task::spawn(async move {
            loop {
                for (name, notifier) in &notifiers {
                    log::debug!("flushing notifier for {}", name);
                    if let Err(e) = notifier.flush().await {
                        log::error!("failed to flush notifier for {}: {}", name, e);
                    }
                }
                async_std::task::sleep(Duration::from_secs(5)).await;
            }
        })
    }

    #[allow(deprecated)]
    async fn block_until_midnight(&self) {
        let now = Utc::now();
        let tmrw_mid = (now + TimeDelta::days(1))
            .date()
            .and_hms_opt(0, 1, 0)
            .expect("invalid time?");
        let duration = tmrw_mid
            .signed_duration_since(now)
            .to_std()
            .expect("invalid duration?");
        log::info!(
            "blocking until UTC midnight for {} seconds",
            duration.as_secs()
        );
        async_std::task::sleep(duration).await;
    }

    async fn populate_pairs(&mut self, network: Network, page: u16) -> shared::Result<bool> {
        log::info!(
            "fetching addresses for network: {}, page: {}",
            network,
            page
        );
        let res = future::join_all(
            self.feeds
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let ended = self.ended_feeds[i];
                    async move {
                        if ended {
                            return Ok(vec![]);
                        }
                        f.fetch_addresses(network, page).await
                    }
                })
                .collect::<Vec<_>>(),
        )
        .await;

        for (i, pairs) in res.into_iter().enumerate() {
            let pairs = pairs?;
            log::debug!("received {} pairs", pairs.len());
            if pairs.is_empty() {
                log::info!(
                    "feed {} has ended for page {} (network: {})",
                    i + 1,
                    page,
                    network
                );
                self.ended_feeds[i] = true;
            }
            for pair in pairs {
                match self.storage.is_blocked(&pair.contract_address) {
                    Err(e) => log::error!("failed to check blacklist for address {}", e),
                    Ok(true) => {
                        log::info!("skipping blocked address: {}", pair.contract_address);
                        continue;
                    }
                    Ok(false) => (),
                }

                if let Some(min) = pair.liquidity {
                    if min < self.config.min_liquidity as f64 {
                        log::info!(
                            "skipping low liquidity pool: {} (USD: {})",
                            pair.contract_address,
                            min,
                        );
                        continue;
                    }
                }

                if !self.pools.insert(pair.contract_address.clone()) {
                    log::info!("skipping duplicate address: {}", pair.contract_address);
                    continue;
                }

                self.buffer.push(shared::Request {
                    network: network.to_string(),
                    pool_address: pair.contract_address,
                    mc_or_fdv: pair.mc_or_fdv,
                    maybe_duplicate: pair.base_token != ""
                        && !self.pairs.insert(pair.base_token.clone()),
                    token: if pair.base_token == "" && pair.quote_token == "" {
                        None
                    } else {
                        Some((pair.base_token, pair.quote_token))
                    },
                });
            }
        }

        if self.ended_feeds.iter().all(|&e| e) {
            log::debug!("all feeds ended for network: {}", network);
            return Ok(false);
        }
        Ok(true)
    }

    async fn flush(&mut self) {
        self.block_until_about_next_minute().await;
        let requests = self
            .hosts
            .iter()
            .map(|h| {
                (0..h.bulk_size())
                    .map(|_| {
                        self.buffer
                            .drain(..self.config.host_requests_per_min as usize)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // publish to all hosts at once
        let resp = future::join_all(
            self.hosts
                .iter()
                .zip(requests.clone().into_iter())
                .map(|(host, requests)| host.trigger(requests))
                .collect::<Vec<_>>(),
        )
        .await;

        for (host_batch, orig_batch) in resp.into_iter().zip(requests.into_iter()) {
            for (batch, orig) in host_batch.into_iter().zip(orig_batch) {
                for (resp, pair) in batch.into_iter().zip(orig.into_iter()) {
                    match resp.and_then(|r| r.ohlcv_data()) {
                        Err(e) => {
                            // TODO: ignore pair after max attempts
                            log::error!(
                                "failed to get data for pair ({}) {} (network: {}): {}",
                                pair.token
                                    .as_ref()
                                    .map(|(b, q)| format!("{}/{}", b, q))
                                    .unwrap_or_default(),
                                pair.pool_address,
                                pair.network,
                                e
                            );
                            match e {
                                // pool doesn't exist (better ignore it for the day)
                                shared::Error::UnexpectedStatusCode(404, _) => (),
                                _ => self.buffer.push(pair),
                            }
                        }
                        Ok(resp) => {
                            if let Err(e) = self
                                .notifier
                                .get(&pair.network)
                                .expect("missing notifier")
                                .post_analysis(&pair, resp)
                                .await
                            {
                                log::error!("failed to post analysis: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn block_until_about_next_minute(&mut self) {
        let elapsed = self.current.elapsed();
        if elapsed < ONE_MIN_FIVE_SECS {
            log::info!("blocking until next minute");
            async_std::task::sleep(ONE_MIN_FIVE_SECS - elapsed).await;
        }
        self.current = Instant::now();
    }
}
