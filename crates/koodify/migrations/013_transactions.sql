-- Rename operation_records → transactions and add zainbox business key.

ALTER TABLE IF EXISTS operation_records RENAME TO transactions;

ALTER TABLE transactions
    ADD COLUMN IF NOT EXISTS zainbox_code TEXT;

-- Keep historical index names remapped where Postgres renamed them with the table.
ALTER INDEX IF EXISTS idx_operation_records_operation RENAME TO idx_transactions_operation;
ALTER INDEX IF EXISTS idx_operation_records_resource_key RENAME TO idx_transactions_resource_key;
ALTER INDEX IF EXISTS idx_operation_records_metadata RENAME TO idx_transactions_metadata;

CREATE INDEX IF NOT EXISTS idx_transactions_zainbox_code
    ON transactions (zainbox_code);

-- Business lookup: payments / ops scoped to a zainbox + resource (txnRef, accountNumber, …)
CREATE INDEX IF NOT EXISTS idx_transactions_zainbox_resource
    ON transactions (zainbox_code, resource_key);

-- Note: zainbox_code is a soft business key (no FK) so payment logging is not
-- blocked when the local zainboxes cache has not been seeded yet.
