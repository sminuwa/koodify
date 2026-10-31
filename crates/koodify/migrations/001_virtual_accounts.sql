CREATE TABLE IF NOT EXISTS virtual_accounts (
    id              UUID PRIMARY KEY,
    account_number  TEXT UNIQUE NOT NULL,
    account_name    TEXT NOT NULL,
    bank_name       TEXT NOT NULL,
    email           TEXT NOT NULL,
    zainbox_code    TEXT NOT NULL,
    zainpay_code    TEXT NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw_response    JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_virtual_accounts_zainbox_code
    ON virtual_accounts (zainbox_code);

CREATE INDEX IF NOT EXISTS idx_virtual_accounts_metadata
    ON virtual_accounts USING GIN (metadata);
