-- Local cache of Zainpay zainboxes; customers are keyed to a code_name.

CREATE TABLE IF NOT EXISTS zainboxes (
    code_name           TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    callback_url        TEXT,
    email_notification  TEXT,
    tags                TEXT,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    raw                 JSONB NOT NULL DEFAULT '{}'::jsonb,
    synced_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_zainboxes_name ON zainboxes (name);
CREATE INDEX IF NOT EXISTS idx_zainboxes_is_active ON zainboxes (is_active);

ALTER TABLE customers
    ADD COLUMN IF NOT EXISTS zainbox_code TEXT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'customers_zainbox_code_key'
    ) THEN
        ALTER TABLE customers
            ADD CONSTRAINT customers_zainbox_code_key UNIQUE (zainbox_code);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'customers_zainbox_code_fkey'
    ) THEN
        ALTER TABLE customers
            ADD CONSTRAINT customers_zainbox_code_fkey
            FOREIGN KEY (zainbox_code) REFERENCES zainboxes (code_name)
            ON UPDATE CASCADE ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_customers_zainbox_code ON customers (zainbox_code);
