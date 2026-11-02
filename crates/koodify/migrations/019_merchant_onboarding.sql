-- Merchant onboarding readiness:
--   • email lookup index for future login (not UNIQUE — seed may share notification emails)
--   • optional password_hash for future auth (API may set later)
--   • index helpers for dashboard lookups

CREATE INDEX IF NOT EXISTS idx_customers_email_lower
    ON customers (lower(email))
    WHERE email IS NOT NULL AND btrim(email) <> '';

ALTER TABLE customers
    ADD COLUMN IF NOT EXISTS password_hash TEXT;

CREATE INDEX IF NOT EXISTS idx_kyc_profiles_status ON kyc_profiles (status);
