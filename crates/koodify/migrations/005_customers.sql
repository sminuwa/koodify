CREATE TABLE IF NOT EXISTS customers (
    id              UUID PRIMARY KEY,
    customer_id     TEXT UNIQUE NOT NULL,
    display_name    TEXT NOT NULL,
    email           TEXT,
    mobile_number   TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_customers_email
    ON customers (email);

CREATE INDEX IF NOT EXISTS idx_customers_status
    ON customers (status);

CREATE INDEX IF NOT EXISTS idx_customers_metadata
    ON customers USING GIN (metadata);

CREATE INDEX IF NOT EXISTS idx_customers_display_name
    ON customers (display_name);

-- Backfill customers from any existing KYC profiles.
INSERT INTO customers (
    id, customer_id, display_name, email, mobile_number, metadata, status, created_at, updated_at
)
SELECT
    gen_random_uuid(),
    k.customer_id,
    trim(concat_ws(' ', k.title, k.first_name, NULLIF(k.middle_name, ''), k.surname)),
    k.email,
    k.mobile_number,
    k.metadata,
    k.status,
    k.created_at,
    k.updated_at
FROM kyc_profiles k
ON CONFLICT (customer_id) DO NOTHING;

-- Link KYC → customers
ALTER TABLE kyc_profiles
    DROP CONSTRAINT IF EXISTS fk_kyc_profiles_customer;

ALTER TABLE kyc_profiles
    ADD CONSTRAINT fk_kyc_profiles_customer
    FOREIGN KEY (customer_id) REFERENCES customers (customer_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE;

-- Link API credentials → customers (nullable = system/provider-wide keys)
ALTER TABLE api_credentials
    ADD COLUMN IF NOT EXISTS customer_id TEXT;

CREATE INDEX IF NOT EXISTS idx_api_credentials_customer_id
    ON api_credentials (customer_id);

ALTER TABLE api_credentials
    DROP CONSTRAINT IF EXISTS fk_api_credentials_customer;

ALTER TABLE api_credentials
    ADD CONSTRAINT fk_api_credentials_customer
    FOREIGN KEY (customer_id) REFERENCES customers (customer_id)
    ON UPDATE CASCADE
    ON DELETE SET NULL;
