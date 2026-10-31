-- Rename provider API credential store to `credentials`.
ALTER TABLE IF EXISTS api_credentials RENAME TO credentials;

ALTER INDEX IF EXISTS api_credentials_pkey RENAME TO credentials_pkey;
ALTER INDEX IF EXISTS api_credentials_name_key RENAME TO credentials_name_key;
ALTER INDEX IF EXISTS idx_api_credentials_provider RENAME TO idx_credentials_provider;
ALTER INDEX IF EXISTS idx_api_credentials_environment RENAME TO idx_credentials_environment;
ALTER INDEX IF EXISTS idx_api_credentials_status RENAME TO idx_credentials_status;
ALTER INDEX IF EXISTS idx_api_credentials_metadata RENAME TO idx_credentials_metadata;
ALTER INDEX IF EXISTS idx_api_credentials_customer_id RENAME TO idx_credentials_customer_id;

ALTER TABLE credentials
    DROP CONSTRAINT IF EXISTS fk_api_credentials_customer;

ALTER TABLE credentials
    ADD CONSTRAINT fk_credentials_customer
    FOREIGN KEY (customer_id) REFERENCES customers (customer_id)
    ON UPDATE CASCADE
    ON DELETE SET NULL;
