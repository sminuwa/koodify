-- Keep both identifiers on customers:
--   id          UUID  (opaque / technical)
--   customer_id BIGSERIAL (1, 2, 3, … human-friendly)
-- Child FKs remain on customer_id.

TRUNCATE TABLE credentials, kyc_profiles, customers RESTART IDENTITY CASCADE;

ALTER TABLE customers
    ADD COLUMN IF NOT EXISTS id UUID;

-- Backfill any existing rows (none after truncate, but safe).
UPDATE customers
SET id = gen_random_uuid()
WHERE id IS NULL;

ALTER TABLE customers
    ALTER COLUMN id SET NOT NULL,
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- customer_id stays as BIGSERIAL PK from migration 009.
-- Add unique constraint on id for lookups / future FKs.
ALTER TABLE customers
    DROP CONSTRAINT IF EXISTS customers_id_key;

ALTER TABLE customers
    ADD CONSTRAINT customers_id_key UNIQUE (id);

CREATE INDEX IF NOT EXISTS idx_customers_id ON customers (id);
