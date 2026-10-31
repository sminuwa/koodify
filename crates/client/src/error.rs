use thiserror::Error;

/// Errors returned by [`crate::ZainpayClient`].
#[derive(Debug, Error)]
pub enum ZainpayError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to deserialize Zainpay response: {0}")]
    Deserialize(#[source] serde_json::Error),

    #[error("unexpected HTTP status {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
}
