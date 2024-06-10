use futures::future;
use strum::VariantArray;

use super::storage::Storage;

use super::{
    feed::{Feed, Network},
    host::Host,
    notifier::Notifier,
    provider::Provider,
};

pub struct Config {
    pub max_pages: Option<u32>,
    pub max_attempts_per_pair: Option<u16>,
    pub num_host_requests: u8,
}

pub struct Runner<F, H, P, N> {
    feed: F,
    hosts: Vec<H>,
    provider: P,
    config: Config,
    storage: Storage,
    notifier: N,
    buffer: Vec<shared::Request>,
}

impl<F, H, P, N> Runner<F, H, P, N>
where
    F: Feed,
    H: Host<P> + Sync,
    P: Provider,
    N: Notifier + Sync,
{
    async fn run(&mut self) {
        let mut current_page = 1;
        let mut current_network_idx = 0;

        let batch_len = self.config.num_host_requests as usize
            * self.hosts.iter().map(|h| h.bulk_size()).sum::<usize>();
        loop {
            if current_network_idx == Network::VARIANTS.len() {
                current_network_idx = 0;
            }

            let network = Network::VARIANTS[current_network_idx];
            match self.populate_pairs(network, current_page).await {
                Ok(true) => (),
                Ok(false) => {
                    current_network_idx += 1;
                    continue;
                }
                Err(e) => {
                    log::error!("failed to populate pairs: {}", e);
                    continue;
                }
            }

            while self.buffer.len() >= batch_len {
                log::info!("Reached batch size {}, flushing", self.buffer.len());
                self.flush().await;
            }

            current_page += 1;
        }
    }

    async fn populate_pairs(&mut self, network: Network, page: u32) -> shared::Result<bool> {
        log::info!("fetching addresses for network: {}", network);
        let pairs = self.feed.fetch_addresses(network.into(), page).await?;
        if pairs.is_empty() {
            return Ok(false);
        }
        for pair in pairs {
            match self.storage.is_blocked(&pair.contract_address).await {
                Err(e) => log::error!("failed to check blacklist for address {}", e),
                Ok(true) => {
                    log::info!("skipping blocked address: {}", pair.contract_address);
                    continue;
                }
                Ok(false) => (),
            }

            self.buffer.push(shared::Request {
                network: network.to_string(),
                pool_address: pair.contract_address,
            });
        }
        Ok(true)
    }

    async fn flush(&mut self) {
        let requests = self
            .hosts
            .iter()
            .map(|h| {
                (0..h.bulk_size())
                    .map(|_| {
                        self.buffer
                            .drain(..self.config.num_host_requests as usize)
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
                                "failed to get data for pair {} (network: {}): {}",
                                pair.pool_address,
                                pair.network,
                                e
                            );
                            self.buffer.push(pair);
                        }
                        Ok(resp) => {
                            if let Some(analysis) = resp.analyze() {
                                if let Err(e) = self.notifier.post_analysis(analysis).await {
                                    log::error!("failed to post analysis: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
