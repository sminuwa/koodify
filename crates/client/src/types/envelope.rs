use serde::{Deserialize, Serialize};

/// Standard Zainpay API response envelope.
///
/// Successful application results use `code == "00"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ApiResponse<T> {
    pub code: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Returns `true` when the application-level code indicates success.
    pub fn is_success(&self) -> bool {
        crate::types::codes::is_success_code(&self.code)
    }

    /// Maps the inner `data` value while preserving the envelope fields.
    pub fn map_data<U, F>(self, f: F) -> ApiResponse<U>
    where
        F: FnOnce(Option<T>) -> Option<U>,
    {
        ApiResponse {
            code: self.code,
            status: self.status,
            description: self.description,
            data: f(self.data),
        }
    }
}
