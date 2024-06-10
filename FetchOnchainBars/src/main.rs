use shared::{Request, Response};

use futures::future;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

async fn handler(event: LambdaEvent<Vec<Request>>) -> Result<Vec<Response>, Error> {
    let urls = event.payload.into_iter().map(|r| {
        format!(
            "https://api.geckoterminal.com/api/v2/networks/{network}/pools/{pool}/ohlcv/day?limit=1000",
            network = r.network,
            pool = r.pool_address
        )
    });

    let client = reqwest::Client::new();
    let bodies = future::join_all(urls.into_iter().map(|url| {
        let client = &client;
        async move {
            let resp = match client.get(url).send().await {
                Ok(r) => r,
                Err(e) => {
                    return Response {
                        status: None,
                        body: None,
                        err: Some(e.to_string()),
                    }
                }
            };
            let code = resp.status().as_u16();
            match resp.text().await {
                Ok(body) => Response {
                    status: Some(code),
                    body: Some(body),
                    err: None,
                },
                Err(e) => Response {
                    status: Some(code),
                    body: None,
                    err: Some(e.to_string()),
                },
            }
        }
    }))
    .await;

    Ok(bodies)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(handler)).await
}
