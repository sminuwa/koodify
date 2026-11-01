-- Idempotent inbound webhook deliveries (replay protection).
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    delivery_hash TEXT PRIMARY KEY,
    resource_key  TEXT,
    event_name    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created
    ON webhook_deliveries (created_at);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_resource
    ON webhook_deliveries (resource_key);
