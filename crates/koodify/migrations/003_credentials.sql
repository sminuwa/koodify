-- Generic credential / KYC store.
-- kind examples: kyc | api_key | webhook_secret | card_token | other
-- subject_type examples: customer | merchant | zainbox | system
CREATE TABLE IF NOT EXISTS credentials (
    id              UUID PRIMARY KEY,
    kind            TEXT NOT NULL,
    subject_type    TEXT NOT NULL,
    subject_key     TEXT NOT NULL,
    name            TEXT NOT NULL DEFAULT 'default',
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (kind, subject_type, subject_key, name)
);

CREATE INDEX IF NOT EXISTS idx_credentials_subject
    ON credentials (subject_type, subject_key);

CREATE INDEX IF NOT EXISTS idx_credentials_kind
    ON credentials (kind);

CREATE INDEX IF NOT EXISTS idx_credentials_status
    ON credentials (status);

CREATE INDEX IF NOT EXISTS idx_credentials_payload
    ON credentials USING GIN (payload);

CREATE INDEX IF NOT EXISTS idx_credentials_metadata
    ON credentials USING GIN (metadata);
