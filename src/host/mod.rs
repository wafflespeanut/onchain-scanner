use super::provider::Provider;

mod aws;
mod azure;
mod gcloud;

#[async_trait::async_trait]
pub trait Host<P: Provider> {
    async fn configure(&mut self);
    async fn __trigger(
        &self,
        request: Vec<shared::Request>,
    ) -> Vec<shared::Result<shared::Response>>;

    async fn trigger(&self, request: Vec<shared::Request>) -> Vec<shared::Result<P>> {
        self.__trigger(request).await.into_iter().map(|r| r.and_then(|resp| {
            if let Some(e) = resp.err {
                return Err(shared::Error::Runtime(e))
            }
            match (resp.status, resp.body) {
                (Some(200), Some(b)) => serde_json::from_str(&b).map_err(shared::Error::Serde),
                (Some(s), b) => Err(shared::Error::UnexpectedStatusCode(s as i32, b)),
                _ => unreachable!(),
            }
        })).collect()
    }
}
