use std::time::Duration;

use serde::Deserialize;

pub struct CoinMarketCap;

#[derive(Deserialize)]
pub struct Paginated<T> {
    pub data: PaginatedData<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedData<T> {
    pub page_list: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub base_token_symbol: String,
    pub pair_contract_address: String,
    pub quoto_token_symbol: String,
}

impl Into<Vec<super::Pair>> for Paginated<TokenInfo> {
    fn into(self) -> Vec<super::Pair> {
        self.data
            .page_list
            .into_iter()
            .map(|x| super::Pair {
                contract_address: x.pair_contract_address,
                base_token: x.base_token_symbol,
                quote_token: x.quoto_token_symbol,
            })
            .collect()
    }
}

pub struct Network(u32);

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

impl super::Feed for CoinMarketCap {
    const DELAY: Duration = Duration::from_millis(1100);

    type Response = Paginated<TokenInfo>;

    fn modify(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("Host", "api.coinmarketcap.com")
           .header("Origin", "https://coinmarketcap.com")
           .header("Referer", "https://coinmarketcap.com/")
    }

    fn url(network: super::Network, page: u16) -> String {
        format!(
            "https://api.coinmarketcap.com/dexer/v3/platformpage/pair-pages?platform-id={}&sort-field=txs24h&desc=true&page={}&pageSize=100",
            Network::from(network).0,
            page
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::feed::Pair;
    use serde_json::json;

    #[test]
    fn extract_pair() {
        let data: super::Paginated<super::TokenInfo> = serde_json::from_value(json!({"data":{"hasNextPage":true,"total":"15687","count":100,"pageList":[{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640","poolId":"1364035","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"Wrapped Ether","baseTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","baseTokenSymbol":"WETH","quotoTokenName":"USD Coin","quotoTokenAddress":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","quotoTokenSymbol":"USDC","priceUsd":"3519.1883038030896","priceQuote":"3520.589226876345000461","volumeUsd24h":"244462190.73680976","basePrice1h":"-0.0016834275","quotePrice1h":"-0.0000768347","quoteChange24h":"0.0000766286","baseChange24h":"-0.0038838356","fdv":"10163333804.9034027293630591663069236720176","liquidity":"167028812.69972625","liquidityScore":"755.5182951395498","txns24h":"6940","baseCurrencyId":2396,"baseCurrencyName":"WETH","baseCurrencySlug":"weth","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","reverseOrder":true,"rank":2},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x3416cf6c708da44db2624d63ea0aaef7113527c6","poolId":"1407410","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"USD Coin","baseTokenAddress":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","baseTokenSymbol":"USDC","quotoTokenName":"Tether USD","quotoTokenAddress":"0xdac17f958d2ee523a2206206994597c13d831ec7","quotoTokenSymbol":"USDT","priceUsd":"1.0001458599268565","priceQuote":"1.0001280050941994","volumeUsd24h":"77860060.95629515","basePrice1h":"0.0001365601","quotePrice1h":"0.0002340769","quoteChange24h":"0.0004788517","baseChange24h":"0.00015714","fdv":"24008607069.7415815637793662687625","liquidity":"23643634.430906378","liquidityScore":"625.1801346583517","txns24h":"1033","baseCurrencyId":3408,"baseCurrencyName":"USDC","baseCurrencySlug":"usd-coin","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","reverseOrder":false,"rank":51},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x93d199263632a4ef4bb438f1feb99e57b4b5f0bd0000000000000000000005c2-0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0-0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","poolId":"7608966","dexerId":1404,"dexerName":"Balancer v2 (Ethereum)","baseTokenName":"Wrapped liquid staked Ether 2.0","baseTokenAddress":"0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0","baseTokenSymbol":"wstETH","quotoTokenName":"Wrapped Ether","quotoTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","quotoTokenSymbol":"WETH","priceUsd":"4119.084113830372","priceQuote":"1.1691577003872922","volumeUsd24h":"72692935.81625526","basePrice1h":"0.0004355395","quotePrice1h":"0.000640623","quoteChange24h":"0.002587646","baseChange24h":"0.0030658645","fdv":"12830682174.187663628049358760400669340656","liquidity":"16247830.306704227","liquidityScore":"600.1711652463981","txns24h":"147","baseCurrencyId":12409,"baseCurrencyName":"Lido wstETH","baseCurrencySlug":"lido-finance-wsteth","marketUrl":"https://app.balancer.fi/#/trade/ether/","reverseOrder":false,"rank":322},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x11b815efb8f581194ae79006d24e0d814b7697f6","poolId":"1393323","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"Wrapped Ether","baseTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","baseTokenSymbol":"WETH","quotoTokenName":"Tether USD","quotoTokenAddress":"0xdac17f958d2ee523a2206206994597c13d831ec7","quotoTokenSymbol":"USDT","priceUsd":"3518.2445036823588","priceQuote":"3517.317503896104","volumeUsd24h":"64207446.58822579","basePrice1h":"-0.0021612428","quotePrice1h":"0.0001875894","quoteChange24h":"0.0004075028","baseChange24h":"-0.0054392099","fdv":"10160608132.1504757902932330993291899403428","liquidity":"34749792.79169358","liquidityScore":"650.8521138390837","txns24h":"4193","baseCurrencyId":2396,"baseCurrencyName":"WETH","baseCurrencySlug":"weth","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","reverseOrder":false,"rank":5},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x4585fe77225b41b697c938b018e2ac67ac5a20c0","poolId":"1393296","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"Wrapped BTC","baseTokenAddress":"0x2260fac5e5542a773aa44fbcfedf7c193bc2c599","baseTokenSymbol":"WBTC","quotoTokenName":"Wrapped Ether","quotoTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","quotoTokenSymbol":"WETH","priceUsd":"67427.584828578","priceQuote":"19.166728935591866","volumeUsd24h":"57522504.58251003","basePrice1h":"-0.0004096146","quotePrice1h":"-0.0010765904","quoteChange24h":"-0.0033368591","baseChange24h":"-0.0002485756","fdv":"10318046101.72604354199595950","liquidity":"78405310.52591315","liquidityScore":"705.0999839856312","txns24h":"841","baseCurrencyId":3717,"baseCurrencyName":"Wrapped Bitcoin","baseCurrencySlug":"wrapped-bitcoin","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0x2260fac5e5542a773aa44fbcfedf7c193bc2c599","reverseOrder":false,"rank":63},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x109830a1aaad605bbf02a9dfa7b0b92ec2fb7daa","poolId":"1772230","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"Wrapped liquid staked Ether 2.0","baseTokenAddress":"0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0","baseTokenSymbol":"wstETH","quotoTokenName":"Wrapped Ether","quotoTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","quotoTokenSymbol":"WETH","priceUsd":"4115.212616479708","priceQuote":"1.169390455406572","volumeUsd24h":"53819105.967456296","basePrice1h":"-0.0011701053","quotePrice1h":"-0.0011737262","quoteChange24h":"-0.003245241","baseChange24h":"-0.0025951504","fdv":"12818622708.861915008465179719379527335184","liquidity":"22071487.063788038","liquidityScore":"620.5929797226954","txns24h":"633","baseCurrencyId":12409,"baseCurrencyName":"Lido wstETH","baseCurrencySlug":"lido-finance-wsteth","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0","reverseOrder":false,"rank":86},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0xc39e83fe4e412a885c0577c08eb53bdb6548004a","poolId":"10922374","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"RCH Token","baseTokenAddress":"0x57b96d4af698605563a4653d882635da59bf11af","baseTokenSymbol":"RCH","quotoTokenName":"Wrapped Ether","quotoTokenAddress":"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2","quotoTokenSymbol":"WETH","priceUsd":"2.070059982807265","priceQuote":"0.000587873985792966","volumeUsd24h":"51750687.06180759","basePrice1h":"-0.0628072069","quotePrice1h":"-0.0012026214","quoteChange24h":"-0.0049358757","baseChange24h":"-0.2469945685","fdv":"51806218.375532813218258768903096505931195","liquidity":"24034847.942552313","liquidityScore":"626.2741915498176","txns24h":"3175","baseCurrencyName":"RCH Token","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0x57b96d4af698605563a4653d882635da59bf11af","reverseOrder":false,"rank":11},{"platformId":1,"platformName":"Ethereum","dexerPlatformName":"Ethereum","platformCryptoId":1027,"pairContractAddress":"0x435664008f38b0650fbc1c9fc971d0a3bc2f1e47","poolId":"8375223","dexerId":1348,"dexerName":"Uniswap v3 (Ethereum)","baseTokenName":"USDe","baseTokenAddress":"0x4c9edd5852cd905f086c759e8383e09bff1e68b3","baseTokenSymbol":"USDe","quotoTokenName":"Tether USD","quotoTokenAddress":"0xdac17f958d2ee523a2206206994597c13d831ec7","quotoTokenSymbol":"USDT","priceUsd":"1.0011453562724977","priceQuote":"1.0011219931920832","volumeUsd24h":"49890233.79230558","basePrice1h":"0.0002845585","quotePrice1h":"0.0001391362","quoteChange24h":"0.0004307439","baseChange24h":"0.0008194654","fdv":"3440541274.1378902816662599694852523","liquidity":"16150206.262201134","liquidityScore":"599.7693946441749","txns24h":"338","baseCurrencyId":29470,"baseCurrencyName":"Ethena USDe","baseCurrencySlug":"ethena-usde","marketUrl":"https://app.uniswap.org/#/swap?outputCurrency=0x4c9edd5852cd905f086c759e8383e09bff1e68b3","reverseOrder":false,"rank":157}]},"status":{"timestamp":"2024-06-12T09:10:47.434Z","error_code":"0","error_message":"SUCCESS","elapsed":"0","credit_count":0}})).unwrap();
        let pairs: Vec<Pair> = data.into();
        assert_eq!(pairs[0].base_token, "WETH");
        assert_eq!(pairs[0].quote_token, "USDC");
        assert_eq!(
            pairs[0].contract_address,
            "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"
        );
    }
}
