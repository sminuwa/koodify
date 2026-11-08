//! Merchant dashboard password hashing (bcrypt).

use axum::http::StatusCode;

const BCRYPT_COST: u32 = 12;

pub fn hash_password(password: &str) -> Result<String, (StatusCode, String)> {
    bcrypt::hash(password, BCRYPT_COST).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("password hash failed: {err}"),
        )
    })
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    bcrypt::verify(password, password_hash).unwrap_or(false)
}

pub fn validate_password_strength(password: &str) -> Result<(), (StatusCode, String)> {
    if password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}
