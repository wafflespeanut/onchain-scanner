use std::time::Duration;

use serde::Deserialize;

const DELAY: Duration = Duration::from_millis(4050);

pub struct GeckoTerminalTop;
pub struct GeckoTerminalTrending;

#[derive(Deserialize)]
pub struct PaginatedData {
    pub data: Vec<TokenInfo>,
}

impl Into<Vec<super::Pair>> for PaginatedData {
    fn into(self) -> Vec<super::Pair> {
        self.data
            .into_iter()
            .map(|x| {
                let name = x.attributes.name;
                let mut name = name.split("/");
                super::Pair {
                    contract_address: x.attributes.address,
                    base_token: name.next().map(|s| s.trim().into()).unwrap_or_default(),
                    quote_token: name.next().map(|s| s.trim().into()).unwrap_or_default(),
                    liquidity: x.attributes.fdv_usd.parse().ok(),
                }
            })
            .collect()
    }
}

#[derive(Deserialize)]
pub struct TokenInfo {
    pub attributes: Attributes,
}

#[derive(Deserialize)]
pub struct Attributes {
    name: String,
    address: String,
    #[serde(default)]
    fdv_usd: String,
}

impl super::Feed for GeckoTerminalTop {
    const MAX_PAGES: Option<u16> = Some(10);

    const DELAY: Duration = DELAY;

    type Response = PaginatedData;

    fn url(network: super::Network, page: u16) -> String {
        format!(
            "https://api.geckoterminal.com/api/v2/networks/{}/pools?page={}&sort=h24_tx_count_desc",
            network, page
        )
    }
}

impl super::Feed for GeckoTerminalTrending {
    const MAX_PAGES: Option<u16> = Some(10);

    const DELAY: Duration = DELAY;

    type Response = PaginatedData;

    fn url(network: super::Network, page: u16) -> String {
        format!(
            "https://api.geckoterminal.com/api/v2/networks/{}/trending_pools?page={}",
            network, page
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::feed::Pair;
    use serde_json::json;

    #[test]
    fn extract_pair() {
        let data: super::PaginatedData = serde_json::from_value(json!({"data":[{"id":"solana_7sXvhsvzmuxomqdFAU6fzxY3bukX9KkVcWWCLSw51osX","type":"pool","attributes":{"base_token_price_usd":"0.0000000171958095080056","base_token_price_native_currency":"0.000000000078358821118124686299470878035953686996828667070205991","quote_token_price_usd":"149.3","quote_token_price_native_currency":"1.0","base_token_price_quote_token":"0.000000000078358821","quote_token_price_base_token":"12761805062","address":"7sXvhsvzmuxomqdFAU6fzxY3bukX9KkVcWWCLSw51osX","name":"Trump / SOL","pool_created_at":"2024-06-11T11:34:31Z","fdv_usd":"1.71958095","market_cap_usd":null,"price_change_percentage":{"m5":"0","h1":"0","h6":"0","h24":"-99.98"},"transactions":{"m5":{"buys":0,"sells":0,"buyers":0,"sellers":0},"m15":{"buys":0,"sells":0,"buyers":0,"sellers":0},"m30":{"buys":0,"sells":0,"buyers":0,"sellers":0},"h1":{"buys":0,"sells":0,"buyers":0,"sellers":0},"h24":{"buys":27679,"sells":24411,"buyers":360,"sellers":304}},"volume_usd":{"m5":"0.0","h1":"0.0","h6":"0.0","h24":"2524456.90576966"},"reserve_in_usd":"220.6525"},"relationships":{"base_token":{"data":{"id":"solana_C1XRDFrD9EYNUvi69SD9tZs4nvq3oVWNACUfFHpwWu3y","type":"token"}},"quote_token":{"data":{"id":"solana_So11111111111111111111111111111111111111112","type":"token"}},"dex":{"data":{"id":"raydium","type":"dex"}}}},{"id":"solana_57Tu1cFCTwCQnYu4qeatEeLKkcHxqBXzgZ2dKaEyNS8F","type":"pool","attributes":{"base_token_price_usd":"0.00195141461717062","base_token_price_native_currency":"0.000012775371432701097358074","quote_token_price_usd":"151.45","quote_token_price_native_currency":"1.0","base_token_price_quote_token":"0.00001278","quote_token_price_base_token":"78275.61","address":"57Tu1cFCTwCQnYu4qeatEeLKkcHxqBXzgZ2dKaEyNS8F","name":"AIBang / SOL","pool_created_at":"2024-06-10T12:50:58Z","fdv_usd":"19514146","market_cap_usd":null,"price_change_percentage":{"m5":"0.92","h1":"1.85","h6":"4.03","h24":"4.87"},"transactions":{"m5":{"buys":19,"sells":14,"buyers":11,"sellers":9},"m15":{"buys":187,"sells":183,"buyers":20,"sellers":21},"m30":{"buys":444,"sells":443,"buyers":20,"sellers":22},"h1":{"buys":953,"sells":946,"buyers":23,"sellers":24},"h24":{"buys":24849,"sells":24211,"buyers":662,"sellers":84}},"volume_usd":{"m5":"4471.8294436042","h1":"253046.673274373","h6":"1596607.89203508","h24":"6375746.97775684"},"reserve_in_usd":"250904.4737"},"relationships":{"base_token":{"data":{"id":"solana_5qpPeXdMrK5JPWVE8eUxzW8xbFpYyZE8TEJ8fUw22hpE","type":"token"}},"quote_token":{"data":{"id":"solana_So11111111111111111111111111111111111111112","type":"token"}},"dex":{"data":{"id":"raydium","type":"dex"}}}},{"id":"solana_83v8iPyZihDEjDdY8RdZddyZNyUtXngz69Lgo9Kt5d6d","type":"pool","attributes":{"base_token_price_usd":"151.45","base_token_price_native_currency":"1.0","quote_token_price_usd":"1.00108266092773","quote_token_price_native_currency":"0.00660523137647648","base_token_price_quote_token":"151.40","quote_token_price_base_token":"0.00660523","address":"83v8iPyZihDEjDdY8RdZddyZNyUtXngz69Lgo9Kt5d6d","name":"SOL / USDC","pool_created_at":"2023-07-05T14:50:59Z","fdv_usd":"87516899676","market_cap_usd":null,"price_change_percentage":{"m5":"-0.01","h1":"0.16","h6":"3.81","h24":"-1.08"},"transactions":{"m5":{"buys":60,"sells":44,"buyers":41,"sellers":32},"m15":{"buys":179,"sells":166,"buyers":89,"sellers":70},"m30":{"buys":360,"sells":400,"buyers":149,"sellers":138},"h1":{"buys":816,"sells":777,"buyers":296,"sellers":228},"h24":{"buys":24114,"sells":23143,"buyers":4243,"sellers":3830}},"volume_usd":{"m5":"7221.137909989","h1":"114034.292492884","h6":"888248.832436134","h24":"4650013.62122384"},"reserve_in_usd":"316942.2403"},"relationships":{"base_token":{"data":{"id":"solana_So11111111111111111111111111111111111111112","type":"token"}},"quote_token":{"data":{"id":"solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","type":"token"}},"dex":{"data":{"id":"orca","type":"dex"}}}}]})).unwrap();
        let pairs: Vec<Pair> = data.into();
        assert_eq!(pairs[0].base_token, "Trump");
        assert_eq!(pairs[0].quote_token, "SOL");
        assert_eq!(
            pairs[0].contract_address,
            "7sXvhsvzmuxomqdFAU6fzxY3bukX9KkVcWWCLSw51osX"
        );
    }
}
