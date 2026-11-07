-- Wrap merchant sk_… for outbound webhook HMAC (merchants verify with the same sk_).
-- Auth still uses secret_key_hash; plaintext is never returned from list APIs.

ALTER TABLE credentials
    ADD COLUMN IF NOT EXISTS secret_key_wrapped TEXT;

COMMENT ON COLUMN credentials.secret_key_wrapped IS
    'AES-GCM sealed merchant secret used only to sign outbound webhooks; merchants verify with sk_…';
