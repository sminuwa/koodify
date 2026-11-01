-- Naming consistency:
--   • webhook_deliveries.resource_key → txn_ref  (payment join key)
--   • zainboxes.code_name → zainbox_code         (canonical zainbox id)
--   • payment_splits / split_plans.primary_zainbox → primary_zainbox_code
--   • credentials.private_key → zainbox_code     (stores Zainpay codeName, not a crypto key)
--
-- transactions.resource_key stays polymorphic (txnRef OR accountNumber).

-- 1) Inbound webhook deliveries: payment correlation key
ALTER TABLE webhook_deliveries
    RENAME COLUMN resource_key TO txn_ref;

ALTER INDEX IF EXISTS idx_webhook_deliveries_resource
    RENAME TO idx_webhook_deliveries_txn_ref;

-- 2) Zainboxes cache: canonical business id column
ALTER TABLE zainboxes
    RENAME COLUMN code_name TO zainbox_code;

-- customers.zainbox_code FK continues to reference the renamed PK column.

-- 3) Split plan / run: role-qualified zainbox id
ALTER TABLE split_plans
    RENAME COLUMN primary_zainbox TO primary_zainbox_code;

ALTER INDEX IF EXISTS idx_split_plans_primary
    RENAME TO idx_split_plans_primary_zainbox_code;

ALTER TABLE payment_splits
    RENAME COLUMN primary_zainbox TO primary_zainbox_code;

-- 4) Credentials: stop calling the zainbox id "private_key"
ALTER TABLE credentials
    RENAME COLUMN private_key TO zainbox_code;
