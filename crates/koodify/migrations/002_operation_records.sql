CREATE TABLE IF NOT EXISTS operation_records (
    id              UUID PRIMARY KEY,
    operation       TEXT NOT NULL,
    resource_key    TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_body    JSONB,
    response_body   JSONB NOT NULL,
    zainpay_code    TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_operation_records_operation
    ON operation_records (operation);

CREATE INDEX IF NOT EXISTS idx_operation_records_resource_key
    ON operation_records (resource_key);

CREATE INDEX IF NOT EXISTS idx_operation_records_metadata
    ON operation_records USING GIN (metadata);
