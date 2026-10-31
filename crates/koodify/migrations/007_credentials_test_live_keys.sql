-- Each customer has exactly two credential rows: test + live (public/private keys).
DROP TABLE IF EXISTS credentials;

CREATE TABLE credentials (
    id              UUID PRIMARY KEY,
    customer_id     TEXT NOT NULL REFERENCES customers (customer_id)
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

CREATE INDEX IF NOT EXISTS idx_credentials_customer_id
    ON credentials (customer_id);

CREATE INDEX IF NOT EXISTS idx_credentials_environment
    ON credentials (environment);

CREATE INDEX IF NOT EXISTS idx_credentials_status
    ON credentials (status);

CREATE INDEX IF NOT EXISTS idx_credentials_metadata
    ON credentials USING GIN (metadata);
