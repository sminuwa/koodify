-- Per-merchant API secrets (sk_test_… / sk_live_…).
-- Merchants send these as X-Api-Key for zainbox-scoped reads/ops.
-- GATEWAY_API_KEY remains the platform admin key (not shared with merchants).
-- Plaintext is never stored — only SHA-256 hash + display prefix.

ALTER TABLE credentials
    ADD COLUMN IF NOT EXISTS secret_key_hash TEXT,
    ADD COLUMN IF NOT EXISTS secret_key_prefix TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_credentials_secret_key_hash
    ON credentials (secret_key_hash)
    WHERE secret_key_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_credentials_secret_key_prefix
    ON credentials (secret_key_prefix)
    WHERE secret_key_prefix IS NOT NULL;
