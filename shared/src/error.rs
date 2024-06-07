use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("AWS SDK error: {0}")]
    AwsSdk(#[from] aws_sdk_lambda::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Payload missing")]
    NoPayload,
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Unexpected status code: {0} (payload: {1:?}")]
    UnexpectedStatusCode(i32, Option<String>),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
}

pub type Result<T> = std::result::Result<T, Error>;
