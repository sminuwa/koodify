-- Append-only activity / audit log for payment lifecycle events.
-- Use `source` to tell Path 1 (Zainpay webhook) vs Path 2 (reconciler) vs status poll.

CREATE TABLE IF NOT EXISTS activity_events (
    id           BIGSERIAL PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    event        TEXT NOT NULL,
    source       TEXT NOT NULL,
    txn_ref      TEXT,
    zainbox_code TEXT,
    summary      TEXT,
    payload      JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_activity_events_txn_ref
    ON activity_events (txn_ref, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_activity_events_created
    ON activity_events (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_activity_events_source
    ON activity_events (source, created_at DESC);
