use super::Host;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_lambda::{config::Region, primitives::Blob, Client};
use futures::future;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct AwsLambda {
    clients: Vec<Client>,
}

#[async_trait::async_trait]
impl<P: super::Provider> Host<P> for AwsLambda
where
    P: Send + DeserializeOwned,
{
    async fn configure(&mut self) {
        if self.clients.len() > 0 {
            return;
        }
        for &r in shared::AWS_REGIONS.iter() {
            let region_provider = RegionProviderChain::first_try(Region::new(String::from(r)));
            let config = aws_config::from_env().region(region_provider).load().await;
            self.clients.push(Client::new(&config));
        }
    }

    async fn __trigger(
        &self,
        request: Vec<shared::Request>,
    ) -> Vec<shared::Result<shared::Response>> {
        future::join_all(self.clients.iter().zip(request.into_iter()).map(|(client, request)| {
            async move {
                let res = client
                    .invoke()
                    .function_name("FetchOnchainBars")
                    .payload(Blob::new(serde_json::to_vec(&request)?))
                    .send()
                    .await
                    .map_err(|e| shared::Error::AwsSdk(e.into()))?;
                if res.payload.is_none() {
                    return Err(shared::Error::NoPayload);
                }
                let payload = res.payload.unwrap().into_inner();
                if res.status_code != 200 {
                    return Err(shared::Error::UnexpectedStatusCode(
                        res.status_code,
                        String::from_utf8(payload).ok(),
                    ));
                }
                serde_json::from_slice(&payload).map_err(shared::Error::Serde)
            }
        }))
        .await
    }
}
