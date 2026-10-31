-- customer_id is a plain auto-increment integer (1, 2, 3, …) — not a prefixed string, not UUID.
TRUNCATE TABLE credentials, kyc_profiles, customers CASCADE;

DROP TABLE IF EXISTS credentials;
DROP TABLE IF EXISTS kyc_profiles;
DROP TABLE IF EXISTS customers;

CREATE TABLE customers (
    customer_id     BIGSERIAL PRIMARY KEY,
    display_name    TEXT NOT NULL,
    email           TEXT,
    mobile_number   TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_customers_email ON customers (email);
CREATE INDEX IF NOT EXISTS idx_customers_status ON customers (status);
CREATE INDEX IF NOT EXISTS idx_customers_metadata ON customers USING GIN (metadata);
CREATE INDEX IF NOT EXISTS idx_customers_display_name ON customers (display_name);

CREATE TABLE kyc_profiles (
    id              UUID PRIMARY KEY,
    customer_id     BIGINT NOT NULL UNIQUE
                        REFERENCES customers (customer_id)
                        ON UPDATE CASCADE ON DELETE CASCADE,
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

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_email ON kyc_profiles (email);
CREATE INDEX IF NOT EXISTS idx_kyc_profiles_bvn ON kyc_profiles (bvn);
CREATE INDEX IF NOT EXISTS idx_kyc_profiles_status ON kyc_profiles (status);
CREATE INDEX IF NOT EXISTS idx_kyc_profiles_metadata ON kyc_profiles USING GIN (metadata);

CREATE TABLE credentials (
    id              UUID PRIMARY KEY,
    customer_id     BIGINT NOT NULL
                        REFERENCES customers (customer_id)
                        ON UPDATE CASCADE ON DELETE CASCADE,
    environment     TEXT NOT NULL CHECK (environment IN ('test', 'live')),
    public_key      TEXT NOT NULL,
    private_key     TEXT NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (customer_id, environment)
);

CREATE INDEX IF NOT EXISTS idx_credentials_customer_id ON credentials (customer_id);
CREATE INDEX IF NOT EXISTS idx_credentials_environment ON credentials (environment);
CREATE INDEX IF NOT EXISTS idx_credentials_status ON credentials (status);
CREATE INDEX IF NOT EXISTS idx_credentials_metadata ON credentials USING GIN (metadata);
