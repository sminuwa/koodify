use serde::Serialize;
use serde_json::Value;
use tracing::{info, warn};
use client::{ApiResponse, CreateVirtualAccountRequest, VirtualAccountData};

use crate::db::{self, models::VirtualAccountRecord};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountDataWithMeta {
    pub account_name: String,
    pub account_number: String,
    pub account_type: String,
    pub bank_name: String,
    pub email: String,
    pub metadata: Value,
}

impl VirtualAccountDataWithMeta {
    pub fn from_parts(data: VirtualAccountData, metadata: Value) -> Self {
        Self {
            account_name: data.account_name,
            account_number: data.account_number,
            account_type: data.account_type,
            bank_name: data.bank_name,
            email: data.email,
            metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredVirtualAccount {
    pub id: uuid::Uuid,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: String,
    pub email: String,
    pub zainbox_code: String,
    pub zainpay_code: String,
    pub metadata: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<VirtualAccountRecord> for StoredVirtualAccount {
    fn from(row: VirtualAccountRecord) -> Self {
        Self {
            id: row.id,
            account_name: row.account_name,
            account_number: row.account_number,
            bank_name: row.bank_name,
            email: row.email,
            zainbox_code: row.zainbox_code,
            zainpay_code: row.zainpay_code,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

pub async fn create_with_metadata(
    state: &AppState,
    request: CreateVirtualAccountRequest,
    metadata: Value,
) -> Result<ApiResponse<VirtualAccountDataWithMeta>, CreateError> {
    let zainbox_code = request.zainbox_code.clone();

    // Only known Zainpay fields are serialized by the client — metadata never leaves here.
    let response = state
        .zainpay
        .virtual_accounts()
        .create(&request)
        .await
        .map_err(CreateError::Upstream)?;

    if !response.is_success() {
        warn!(
            code = %response.code,
            description = %response.description,
            "zainpay create virtual account failed"
        );
        return Ok(ApiResponse {
            code: response.code,
            status: response.status,
            description: response.description,
            data: None,
        });
    }

    let Some(data) = response.data.clone() else {
        return Ok(ApiResponse {
            code: response.code,
            status: response.status,
            description: response.description,
            data: None,
        });
    };

    match db::virtual_accounts::insert_success(
        &state.pool,
        &zainbox_code,
        &metadata,
        &response,
        &data,
    )
    .await
    {
        Ok(row) => {
            info!(
                account_number = %row.account_number,
                "stored virtual account with metadata"
            );
        }
        Err(err) => {
            // Zainpay already created the account; surface success with metadata
            // even if persistence fails, but log loudly.
            warn!(error = %err, "failed to persist virtual account metadata");
        }
    }

    Ok(ApiResponse {
        code: response.code,
        status: response.status,
        description: response.description,
        data: Some(VirtualAccountDataWithMeta::from_parts(data, metadata)),
    })
}

pub async fn get_by_account_number(
    state: &AppState,
    account_number: &str,
) -> Result<Option<StoredVirtualAccount>, sqlx::Error> {
    let row = db::virtual_accounts::find_by_account_number(&state.pool, account_number).await?;
    Ok(row.map(StoredVirtualAccount::from))
}

pub async fn balance_with_metadata(
    state: &AppState,
    account_number: &str,
) -> Result<ApiResponse<Value>, CreateError> {
    let response = state
        .zainpay
        .virtual_accounts()
        .balance(account_number)
        .await
        .map_err(CreateError::Upstream)?;

    let metadata = db::virtual_accounts::find_by_account_number(&state.pool, account_number)
        .await
        .ok()
        .flatten()
        .map(|row| row.metadata)
        .unwrap_or_else(|| Value::Object(Default::default()));

    Ok(response.map_data(|data| {
        data.map(|balance| {
            let mut value = serde_json::to_value(balance).unwrap_or(Value::Null);
            if let Value::Object(ref mut map) = value {
                map.insert("metadata".into(), metadata);
            }
            value
        })
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Upstream(#[from] client::ZainpayError),
}
