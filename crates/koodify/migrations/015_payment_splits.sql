-- Saved reusable split plans + per-txn split runs / legs (Zainbox Payment Split).

CREATE TABLE IF NOT EXISTS split_plans (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    primary_zainbox TEXT NOT NULL,
    shares          JSONB NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_split_plans_primary
    ON split_plans (primary_zainbox);

-- One row per checkout txn that requested a split.
CREATE TABLE IF NOT EXISTS payment_splits (
    txn_ref              TEXT PRIMARY KEY,
    primary_zainbox      TEXT NOT NULL,
    checkout_amount      TEXT NOT NULL,
    total_kobo           BIGINT NOT NULL,
    plan_snapshot        JSONB NOT NULL,
    status               TEXT NOT NULL DEFAULT 'pending',
    source_account       TEXT,
    source_bank_code     TEXT,
    error                TEXT,
    claimed_at           TIMESTAMPTZ,
    completed_at         TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT payment_splits_status_check CHECK (
        status IN ('pending', 'running', 'success', 'partial', 'failed', 'skipped')
    )
);

CREATE INDEX IF NOT EXISTS idx_payment_splits_status
    ON payment_splits (status);

CREATE TABLE IF NOT EXISTS payment_split_legs (
    id                UUID PRIMARY KEY,
    txn_ref           TEXT NOT NULL REFERENCES payment_splits (txn_ref) ON DELETE CASCADE,
    leg_index         INT NOT NULL,
    zainbox_code      TEXT NOT NULL,
    role              TEXT NOT NULL,
    amount_kobo       BIGINT NOT NULL,
    share_percent     DOUBLE PRECISION,
    share_amount      TEXT,
    status            TEXT NOT NULL DEFAULT 'pending',
    dest_account      TEXT,
    dest_bank_code    TEXT,
    transfer_txn_ref  TEXT,
    payment_ref       TEXT,
    error             TEXT,
    attempts          INT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (txn_ref, leg_index),
    CONSTRAINT payment_split_legs_role_check CHECK (role IN ('keep', 'transfer')),
    CONSTRAINT payment_split_legs_status_check CHECK (
        status IN ('pending', 'success', 'failed', 'skipped')
    )
);

CREATE INDEX IF NOT EXISTS idx_payment_split_legs_txn
    ON payment_split_legs (txn_ref);
