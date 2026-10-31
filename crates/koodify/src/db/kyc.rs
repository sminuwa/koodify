use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::KycProfile;

const COLUMNS: &str = r#"
    id, customer_id, title, first_name, middle_name, surname, email, mobile_number,
    bvn, dob, gender, address, state, bank_type, zainbox_code, metadata, status,
    created_at, updated_at
"#;

pub struct UpsertKyc<'a> {
    pub customer_id: i64,
    pub title: &'a str,
    pub first_name: &'a str,
    pub middle_name: Option<&'a str>,
    pub surname: &'a str,
    pub email: &'a str,
    pub mobile_number: &'a str,
    pub bvn: &'a str,
    pub dob: &'a str,
    pub gender: &'a str,
    pub address: &'a str,
    pub state: &'a str,
    pub bank_type: Option<&'a str>,
    pub zainbox_code: Option<&'a str>,
    pub metadata: &'a Value,
    pub status: &'a str,
}

pub async fn upsert(pool: &PgPool, input: UpsertKyc<'_>) -> Result<KycProfile, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, KycProfile>(&format!(
        r#"
        INSERT INTO kyc_profiles (
            id, customer_id, title, first_name, middle_name, surname, email, mobile_number,
            bvn, dob, gender, address, state, bank_type, zainbox_code, metadata, status
        )
        VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,
            $9,$10,$11,$12,$13,$14,$15,$16,$17
        )
        ON CONFLICT (customer_id) DO UPDATE SET
            title = EXCLUDED.title,
            first_name = EXCLUDED.first_name,
            middle_name = EXCLUDED.middle_name,
            surname = EXCLUDED.surname,
            email = EXCLUDED.email,
            mobile_number = EXCLUDED.mobile_number,
            bvn = EXCLUDED.bvn,
            dob = EXCLUDED.dob,
            gender = EXCLUDED.gender,
            address = EXCLUDED.address,
            state = EXCLUDED.state,
            bank_type = EXCLUDED.bank_type,
            zainbox_code = EXCLUDED.zainbox_code,
            metadata = EXCLUDED.metadata,
            status = EXCLUDED.status,
            updated_at = now()
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(input.customer_id)
    .bind(input.title)
    .bind(input.first_name)
    .bind(input.middle_name)
    .bind(input.surname)
    .bind(input.email)
    .bind(input.mobile_number)
    .bind(input.bvn)
    .bind(input.dob)
    .bind(input.gender)
    .bind(input.address)
    .bind(input.state)
    .bind(input.bank_type)
    .bind(input.zainbox_code)
    .bind(input.metadata)
    .bind(input.status)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<KycProfile>, sqlx::Error> {
    sqlx::query_as::<_, KycProfile>(&format!(
        r#"SELECT {COLUMNS} FROM kyc_profiles WHERE id = $1"#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_customer_id(
    pool: &PgPool,
    customer_id: i64,
) -> Result<Option<KycProfile>, sqlx::Error> {
    sqlx::query_as::<_, KycProfile>(&format!(
        r#"SELECT {COLUMNS} FROM kyc_profiles WHERE customer_id = $1"#
    ))
    .bind(customer_id)
    .fetch_optional(pool)
    .await
}

pub async fn list(
    pool: &PgPool,
    status: Option<&str>,
) -> Result<Vec<KycProfile>, sqlx::Error> {
    if let Some(status) = status {
        sqlx::query_as::<_, KycProfile>(&format!(
            r#"
            SELECT {COLUMNS}
            FROM kyc_profiles
            WHERE status = $1
            ORDER BY customer_id ASC
            "#
        ))
        .bind(status)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, KycProfile>(&format!(
            r#"
            SELECT {COLUMNS}
            FROM kyc_profiles
            ORDER BY customer_id ASC
            "#
        ))
        .fetch_all(pool)
        .await
    }
}

pub async fn set_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Option<KycProfile>, sqlx::Error> {
    sqlx::query_as::<_, KycProfile>(&format!(
        r#"
        UPDATE kyc_profiles
        SET status = $2, updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await
}
