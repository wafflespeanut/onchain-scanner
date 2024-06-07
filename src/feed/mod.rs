use std::time::Duration;

mod cm;

#[derive(strum_macros::Display, strum_macros::VariantArray)]
enum Network {
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

#[async_trait::async_trait]
trait Feed {
    const DELAY: Duration = Duration::from_secs(1);

    type Network: From<Network>;

    async fn fetch_addresses(
        &self,
        network: Self::Network,
        page: u32,
    ) -> Result<Vec<Pair>, shared::Error>;
}
