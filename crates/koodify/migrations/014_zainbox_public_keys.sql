-- Publishable checkout keys bound to each zainbox.
-- Merchants put `public_key` in the browser; Koodify resolves → code_name for Zainpay.

ALTER TABLE zainboxes
    ADD COLUMN IF NOT EXISTS public_key TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_zainboxes_public_key
    ON zainboxes (public_key)
    WHERE public_key IS NOT NULL;
