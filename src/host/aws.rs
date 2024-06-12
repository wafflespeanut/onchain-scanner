use super::Host;
use aws_credential_types::Credentials;
use aws_sdk_lambda::{config::Region, primitives::Blob, Client, Config};
use futures::future;
use serde::de::DeserializeOwned;

use std::env;

#[derive(Default)]
pub struct AwsLambda {
    name: String,
    clients: Vec<Client>,
}

impl AwsLambda {
    pub fn new(name: &str) -> shared::Result<Self> {
        let mut clients = vec![];
        let creds = if let (Some(a), Some(s)) = (
            env::var("AWS_ACCESS_KEY").ok(),
            env::var("AWS_SECRET_ACCESS_KEY").ok(),
        ) {
            Credentials::from_keys(a, s, None)
        } else {
            return Err(shared::Error::Config(
                "Missing AWS access and secret keys".into(),
            ));
        };
        for &r in shared::AWS_REGIONS.iter() {
            let config = Config::builder()
                .credentials_provider(creds.clone())
                .region(Region::new(r))
                .build();
            clients.push(Client::from_conf(config));
        }
        Ok(AwsLambda {
            name: name.into(),
            clients,
        })
    }
}

#[async_trait::async_trait]
impl<P: super::Provider> Host<P> for AwsLambda
where
    P: Send + DeserializeOwned,
{
    fn bulk_size(&self) -> usize {
        self.clients.len()
    }

    async fn __trigger(
        &self,
        request: Vec<Vec<shared::Request>>,
    ) -> Vec<shared::Result<Vec<shared::Response>>> {
        future::join_all(self.clients.iter().zip(request.into_iter()).map(
            |(client, req)| async move {
                let res = client
                    .invoke()
                    .function_name(&self.name)
                    .payload(Blob::new(serde_json::to_vec(&req)?))
                    .send()
                    .await
                    .map_err(|e| shared::Error::AwsSdk(e.into()))?;
                if res.payload.is_none() {
                    return Err(shared::Error::NoPayload);
                }
                let payload = res.payload.unwrap().into_inner();
                if res.status_code != 200 {
                    return Err(shared::Error::UnexpectedStatusCode(
                        res.status_code as u16,
                        String::from_utf8(payload).ok(),
                    ));
                }
                serde_json::from_slice(&payload).map_err(shared::Error::Serde)
            },
        ))
        .await
    }
}
