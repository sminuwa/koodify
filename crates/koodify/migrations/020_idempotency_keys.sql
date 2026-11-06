-- Safe retries for money-moving POSTs (DX issue 07).
-- Scope + key are unique; request_hash detects same-key / different-body conflicts.
CREATE TABLE IF NOT EXISTS idempotency_keys (
    scope          TEXT NOT NULL,
    idem_key       TEXT NOT NULL,
    request_hash   TEXT NOT NULL,
    state          TEXT NOT NULL DEFAULT 'pending'
                   CHECK (state IN ('pending', 'completed')),
    response_body  JSONB,
    resource_key   TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (scope, idem_key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_created
    ON idempotency_keys (created_at);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_resource
    ON idempotency_keys (resource_key)
    WHERE resource_key IS NOT NULL;
