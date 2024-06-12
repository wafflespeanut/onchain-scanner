use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
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
    UnexpectedStatusCode(u16, Option<String>),
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Storage error: {0}")]
    Storage(#[from] redb::Error),
    #[error("Batch error: {0}")]
    Batch(#[from] std::sync::Arc<Error>),
}

macro_rules! impl_from(
    ($from:ty, $to:ident) => {
        impl From<$from> for Error {
            fn from(e: $from) -> Self {
                Error::$to(e.into())
            }
        }
    };
);

impl_from!(redb::DatabaseError, Storage);
impl_from!(redb::StorageError, Storage);
impl_from!(redb::CommitError, Storage);
impl_from!(redb::TransactionError, Storage);
impl_from!(redb::TableError, Storage);

pub type Result<T> = std::result::Result<T, Error>;
