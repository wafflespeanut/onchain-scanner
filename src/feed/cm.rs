use serde::Deserialize;

struct CoinMarketCap {
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct Paginated<T> {
    pub data: PaginatedData<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaginatedData<T> {
    pub has_next_page: bool,
    pub total: u32,
    pub page_list: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenInfo {
    pub base_token_symbol: String,
    pub pair_contract_address: String,
    pub quoto_token_symbol: String,
}

struct Network(u32);

impl From<super::Network> for Network {
    fn from(network: super::Network) -> Self {
        match network {
            super::Network::Solana => Network(16),
            super::Network::Ethereum => Network(1),
            super::Network::Base => Network(199),
            super::Network::Blast => Network(210),
            super::Network::TON => Network(173),
            super::Network::BSC => Network(14),
            super::Network::Arbitrum => Network(51),
            super::Network::Avalanche => Network(28),
            super::Network::Optimism => Network(42),
            super::Network::Fantom => Network(24),
            super::Network::Metis => Network(99),
            super::Network::Ronin => Network(66),
        }
    }
}

#[async_trait::async_trait]
impl super::Feed for CoinMarketCap {
    type Network = Network;

    async fn fetch_addresses(
        &self,
        network: Self::Network,
        page: u32,
    ) -> Result<Vec<super::Pair>, shared::Error> {
        match self.client
            .get(format!("https://api.coinmarketcap.com/dexer/v3/platformpage/pair-pages?platform-id={}&sort-field=txs24h&desc=true&page={}&pageSize=100", network.0, page))
            .send()
            .await {
            Ok(resp) => {
                let r = resp.json::<Paginated<TokenInfo>>().await?;
                Ok(r.data.page_list.into_iter().map(|x| super::Pair {
                    contract_address: x.pair_contract_address,
                    base_token: x.base_token_symbol,
                    quote_token: x.quoto_token_symbol,
                }).collect())
            },
            Err(e) => Err(shared::Error::Http(e.into())),
        }
    }
}
