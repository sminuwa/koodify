-- Split the generic credentials store into dedicated KYC and API tables.
DROP TABLE IF EXISTS credentials;

CREATE TABLE IF NOT EXISTS kyc_profiles (
    id              UUID PRIMARY KEY,
    customer_id     TEXT UNIQUE NOT NULL,
    title           TEXT NOT NULL,
    first_name      TEXT NOT NULL,
    middle_name     TEXT,
    surname         TEXT NOT NULL,
    email           TEXT NOT NULL,
    mobile_number   TEXT NOT NULL,
    bvn             TEXT NOT NULL,
    dob             TEXT NOT NULL,
    gender          TEXT NOT NULL,
    address         TEXT NOT NULL,
    state           TEXT NOT NULL,
    bank_type       TEXT,
    zainbox_code    TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_email
    ON kyc_profiles (email);

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_bvn
    ON kyc_profiles (bvn);

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_status
    ON kyc_profiles (status);

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_metadata
    ON kyc_profiles USING GIN (metadata);

CREATE TABLE IF NOT EXISTS api_credentials (
    id              UUID PRIMARY KEY,
    name            TEXT UNIQUE NOT NULL,
    provider        TEXT NOT NULL DEFAULT 'zainpay',
    environment     TEXT NOT NULL,
    base_url        TEXT,
    api_key         TEXT NOT NULL,
    webhook_secret  TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_api_credentials_provider
    ON api_credentials (provider);

CREATE INDEX IF NOT EXISTS idx_api_credentials_environment
    ON api_credentials (environment);

CREATE INDEX IF NOT EXISTS idx_api_credentials_status
    ON api_credentials (status);

CREATE INDEX IF NOT EXISTS idx_api_credentials_metadata
    ON api_credentials USING GIN (metadata);
