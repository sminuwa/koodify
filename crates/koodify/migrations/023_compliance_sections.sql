-- Merchant compliance portal sections (business / representative / documents / declaration)
-- plus uploaded document metadata. File bytes live on disk under COMPLIANCE_STORAGE_DIR.

CREATE TABLE IF NOT EXISTS compliance_sections (
    id UUID PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES customers (id) ON DELETE CASCADE,
    section TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT compliance_sections_section_chk
        CHECK (section IN ('business', 'representative', 'documents', 'declaration')),
    CONSTRAINT compliance_sections_status_chk
        CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT compliance_sections_customer_section_key UNIQUE (customer_id, section)
);

CREATE INDEX IF NOT EXISTS idx_compliance_sections_customer
    ON compliance_sections (customer_id);

CREATE INDEX IF NOT EXISTS idx_compliance_sections_status
    ON compliance_sections (status);

CREATE TABLE IF NOT EXISTS compliance_documents (
    id UUID PRIMARY KEY,
    customer_id BIGINT NOT NULL REFERENCES customers (id) ON DELETE CASCADE,
    doc_type TEXT NOT NULL,
    original_name TEXT NOT NULL,
    content_type TEXT,
    byte_size BIGINT NOT NULL DEFAULT 0,
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT compliance_documents_type_chk
        CHECK (doc_type IN ('cacCertificate', 'utilityBill')),
    CONSTRAINT compliance_documents_customer_type_key UNIQUE (customer_id, doc_type)
);

CREATE INDEX IF NOT EXISTS idx_compliance_documents_customer
    ON compliance_documents (customer_id);
