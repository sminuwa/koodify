-- Column order on customers: uuid first, then integer id (1, 2, 3, …).
-- Child FKs keep column name customer_id → customers(id).

ALTER TABLE credentials DROP CONSTRAINT IF EXISTS credentials_customer_id_fkey;
ALTER TABLE kyc_profiles DROP CONSTRAINT IF EXISTS kyc_profiles_customer_id_fkey;

CREATE TABLE customers_new (
    uuid            UUID NOT NULL DEFAULT gen_random_uuid(),
    id              BIGSERIAL PRIMARY KEY,
    display_name    TEXT NOT NULL,
    email           TEXT,
    mobile_number   TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT customers_uuid_key UNIQUE (uuid)
);

INSERT INTO customers_new (
    uuid, id, display_name, email, mobile_number, metadata, status, created_at, updated_at
)
SELECT
    id,
    customer_id,
    display_name,
    email,
    mobile_number,
    metadata,
    status,
    created_at,
    updated_at
FROM customers
ORDER BY customer_id;

SELECT setval(
    pg_get_serial_sequence('customers_new', 'id'),
    COALESCE((SELECT MAX(id) FROM customers_new), 1),
    true
);

DROP TABLE customers;

ALTER TABLE customers_new RENAME TO customers;

ALTER SEQUENCE customers_new_id_seq RENAME TO customers_id_seq;

CREATE INDEX IF NOT EXISTS idx_customers_uuid ON customers (uuid);
CREATE INDEX IF NOT EXISTS idx_customers_display_name ON customers (display_name);
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers (email);
CREATE INDEX IF NOT EXISTS idx_customers_status ON customers (status);
CREATE INDEX IF NOT EXISTS idx_customers_metadata ON customers USING gin (metadata);

ALTER TABLE kyc_profiles
    ADD CONSTRAINT kyc_profiles_customer_id_fkey
    FOREIGN KEY (customer_id) REFERENCES customers (id)
    ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE credentials
    ADD CONSTRAINT credentials_customer_id_fkey
    FOREIGN KEY (customer_id) REFERENCES customers (id)
    ON UPDATE CASCADE ON DELETE CASCADE;
