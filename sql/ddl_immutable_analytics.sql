CREATE SCHEMA IF NOT EXISTS core_analytics_ledger;
SET search_path TO core_analytics_ledger;

CREATE TABLE IF NOT EXISTS corporate_domains (
    domain_id BIGINT PRIMARY KEY,
    domain_name VARCHAR(64) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO corporate_domains (domain_id, domain_name) VALUES
    (877771073297, 'AuraTetrax-com'),
    (964521086569, 'Fourphonix-com'),
    (188680247367, 'AuraIntelCore-com'),
    (1087754464927, 'AuraIntel-Global-com'),
    (282230018778, 'Standard1979-com'),
    (358233233, 'Prophonix-com')
ON CONFLICT (domain_id) DO NOTHING;

CREATE TABLE IF NOT EXISTS certified_analysis_journal (
    analysis_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id BIGINT NOT NULL REFERENCES corporate_domains(domain_id),
    lifecycle_folder_id BIGINT NOT NULL,
    analysis_type VARCHAR(64) NOT NULL,
    input_payload_hash CHAR(64) NOT NULL,
    calculated_liability_micro BIGINT NOT NULL,
    social_security_split_micro BIGINT NOT NULL,
    reskilling_split_micro BIGINT NOT NULL,
    proof_of_analysis_sha256 CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE OR REPLACE RULE no_update_analysis AS ON UPDATE TO certified_analysis_journal DO INSTEAD NOTHING;
CREATE OR REPLACE RULE no_delete_analysis AS ON DELETE TO certified_analysis_journal DO INSTEAD NOTHING;

CREATE TABLE IF NOT EXISTS fiscal_general_ledger (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id UUID NOT NULL REFERENCES certified_analysis_journal(analysis_id),
    domain_id BIGINT NOT NULL REFERENCES corporate_domains(domain_id),
    account_code VARCHAR(32) NOT NULL,
    debit_micro BIGINT NOT NULL CHECK (debit_micro >= 0),
    credit_micro BIGINT NOT NULL CHECK (credit_micro >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT chk_balanced_entry CHECK ((debit_micro > 0 AND credit_micro = 0) OR (credit_micro > 0 AND debit_micro = 0))
);

CREATE OR REPLACE RULE no_update_ledger AS ON UPDATE TO fiscal_general_ledger DO INSTEAD NOTHING;
CREATE OR REPLACE RULE no_delete_ledger AS ON DELETE TO fiscal_general_ledger DO INSTEAD NOTHING;

ALTER TABLE certified_analysis_journal ENABLE ROW LEVEL SECURITY;
ALTER TABLE fiscal_general_ledger ENABLE ROW LEVEL SECURITY;

CREATE POLICY domain_isolation_analysis_policy ON certified_analysis_journal
    AS RESTRICTIVE
    FOR ALL
    TO PUBLIC
    USING (domain_id = NULLIF(current_setting('app.current_domain_id', true), '')::BIGINT);

CREATE POLICY domain_isolation_ledger_policy ON fiscal_general_ledger
    AS RESTRICTIVE
    FOR ALL
    TO PUBLIC
    USING (domain_id = NULLIF(current_setting('app.current_domain_id', true), '')::BIGINT);
