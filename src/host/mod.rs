use super::provider::Provider;
use std::sync::Arc;

mod aws;
mod azure;
mod gcloud;

pub use self::aws::AwsLambda;

#[async_trait::async_trait]
pub trait Host<P: Provider> {
    fn bulk_size(&self) -> usize;

    async fn __trigger(
        &self,
        request: Vec<Vec<shared::Request>>,
    ) -> Vec<shared::Result<Vec<shared::Response>>>;

    async fn trigger(&self, request: Vec<Vec<shared::Request>>) -> Vec<Vec<shared::Result<P>>> {
        // assert_eq!(request.len(), self.bulk_size()); // ensure that requests match max size for this host
        let resp_len = request.iter().map(|r| r.len()).collect::<Vec<_>>();
        self.__trigger(request)
            .await
            .into_iter()
            .zip(resp_len.into_iter())
            .map(|(res, l)| match res {
                Ok(resp) => resp
                    .into_iter()
                    .map(|r| {
                        if let Some(e) = r.err {
                            return Err(shared::Error::Runtime(e));
                        }
                        match (r.status, r.body) {
                            (Some(200), Some(b)) => {
                                serde_json::from_str(&b).map_err(shared::Error::Serde)
                            }
                            (Some(s), b) => Err(shared::Error::UnexpectedStatusCode(s, b)),
                            _ => unreachable!(),
                        }
                    })
                    .collect::<Vec<_>>(),
                Err(e) => {
                    let e = Arc::new(e);
                    (0..l).map(|_| Err(e.clone().into())).collect::<Vec<_>>()
                }
            })
            .collect()
    }
}
