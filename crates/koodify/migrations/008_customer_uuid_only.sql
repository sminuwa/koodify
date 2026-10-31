-- Drop text business keys (`customer_id` / cus_… prefixes).
-- Customers are identified only by UUID `id`; child FKs use UUID too.

TRUNCATE TABLE credentials, kyc_profiles, customers CASCADE;

ALTER TABLE credentials DROP CONSTRAINT IF EXISTS credentials_customer_id_fkey;
ALTER TABLE credentials DROP CONSTRAINT IF EXISTS fk_credentials_customer;
ALTER TABLE credentials DROP CONSTRAINT IF EXISTS credentials_customer_id_environment_key;

ALTER TABLE kyc_profiles DROP CONSTRAINT IF EXISTS fk_kyc_profiles_customer;
ALTER TABLE kyc_profiles DROP CONSTRAINT IF EXISTS kyc_profiles_customer_id_key;

ALTER TABLE customers DROP COLUMN IF EXISTS customer_id;

-- KYC: customer_id becomes UUID → customers.id
ALTER TABLE kyc_profiles
    ALTER COLUMN customer_id TYPE UUID USING gen_random_uuid();

ALTER TABLE kyc_profiles
    ADD CONSTRAINT kyc_profiles_customer_id_key UNIQUE (customer_id);

ALTER TABLE kyc_profiles
    ADD CONSTRAINT fk_kyc_profiles_customer
    FOREIGN KEY (customer_id) REFERENCES customers (id)
    ON UPDATE CASCADE
    ON DELETE CASCADE;

-- Credentials: customer_id becomes UUID → customers.id
ALTER TABLE credentials
    ALTER COLUMN customer_id TYPE UUID USING gen_random_uuid();

ALTER TABLE credentials
    ADD CONSTRAINT credentials_customer_id_environment_key
    UNIQUE (customer_id, environment);

ALTER TABLE credentials
    ADD CONSTRAINT fk_credentials_customer
    FOREIGN KEY (customer_id) REFERENCES customers (id)
    ON UPDATE CASCADE
    ON DELETE CASCADE;
