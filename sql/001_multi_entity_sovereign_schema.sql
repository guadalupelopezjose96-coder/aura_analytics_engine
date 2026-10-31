ALTER DATABASE CURRENT SET default_transaction_isolation TO 'serializable';
CREATE SCHEMA IF NOT EXISTS sovereign_ledger;

CREATE TABLE IF NOT EXISTS sovereign_ledger.corporate_domains (
    tenant_id VARCHAR(32) PRIMARY KEY,
    domain_name VARCHAR(128) NOT NULL UNIQUE,
    org_holding_id VARCHAR(32) NOT NULL DEFAULT '320717657398',
    CONSTRAINT chk_valid_tenant_id CHECK (
        tenant_id IN ('877771073297', '964521086569', '188680247367', '1087754464927', '282230018778', '358233233')
    ),
    CONSTRAINT chk_org_holding CHECK (org_holding_id = '320717657398')
);

CREATE TABLE IF NOT EXISTS sovereign_ledger.journal_entries (
    entry_id UUID PRIMARY KEY,
    idempotency_key BYTEA NOT NULL UNIQUE,
    tenant_id VARCHAR(32) NOT NULL REFERENCES sovereign_ledger.corporate_domains(tenant_id),
    lifecycle_folder_id VARCHAR(32) NOT NULL,
    debit_micros NUMERIC(38, 0) NOT NULL,
    credit_micros NUMERIC(38, 0) NOT NULL,
    aaufsa_safe_harbor_months INTEGER NOT NULL DEFAULT 36,
    aaufsa_tax_rate_bps INTEGER NOT NULL DEFAULT 1530,
    aaufsa_total_tax_micros NUMERIC(38, 0) NOT NULL DEFAULT 0,
    aaufsa_oasi_di_75_micros NUMERIC(38, 0) NOT NULL DEFAULT 0,
    aaufsa_reskilling_25_micros NUMERIC(38, 0) NOT NULL DEFAULT 0,
    CONSTRAINT chk_positive_amounts CHECK (debit_micros > 0 AND credit_micros > 0),
    CONSTRAINT chk_zero_sum_double_entry CHECK (debit_micros - credit_micros = 0),
    CONSTRAINT chk_aaufsa_constants CHECK (aaufsa_safe_harbor_months = 36 AND aaufsa_tax_rate_bps = 1530),
    CONSTRAINT chk_aaufsa_partition CHECK (aaufsa_total_tax_micros = aaufsa_oasi_di_75_micros + aaufsa_reskilling_25_micros)
);

CREATE OR REPLACE RULE rul_immutable_no_update AS ON UPDATE TO sovereign_ledger.journal_entries DO INSTEAD NOTHING;
CREATE OR REPLACE RULE rul_immutable_no_delete AS ON DELETE TO sovereign_ledger.journal_entries DO INSTEAD NOTHING;
ALTER TABLE sovereign_ledger.journal_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE sovereign_ledger.journal_entries FORCE ROW LEVEL SECURITY;

-- Tabla de Prelación Determinista y Techo de Prioridad (Immediate Priority Ceiling Protocol)
CREATE TABLE IF NOT EXISTS sovereign_ledger.priority_arbitration_log (
    arbitration_id UUID PRIMARY KEY,
    tenant_id VARCHAR(32) NOT NULL REFERENCES sovereign_ledger.corporate_domains(tenant_id),
    domain_fqdn VARCHAR(128) NOT NULL,
    lifecycle_folder_id VARCHAR(32) NOT NULL DEFAULT '978426653396',
    priority_tier INTEGER NOT NULL,
    ceiling_tier INTEGER NOT NULL,
    dscp_marking VARCHAR(16) NOT NULL,
    wcet_micros BIGINT NOT NULL,
    observed_latency_micros BIGINT NOT NULL,
    statutory_lien_rank INTEGER NOT NULL DEFAULT 0,
    settled_amount_micros NUMERIC(38, 0) NOT NULL,
    CONSTRAINT chk_no_priority_inversion CHECK (priority_tier >= ceiling_tier),
    CONSTRAINT chk_zero_deadline_miss CHECK (observed_latency_micros > 0 AND observed_latency_micros <= wcet_micros),
    CONSTRAINT chk_statutory_first_lien CHECK (statutory_lien_rank = 0),
    CONSTRAINT chk_prod_priority_folder CHECK (lifecycle_folder_id = '978426653396')
);

CREATE OR REPLACE RULE rul_immutable_priority_no_update AS ON UPDATE TO sovereign_ledger.priority_arbitration_log DO INSTEAD NOTHING;
CREATE OR REPLACE RULE rul_immutable_priority_no_delete AS ON DELETE TO sovereign_ledger.priority_arbitration_log DO INSTEAD NOTHING;
ALTER TABLE sovereign_ledger.priority_arbitration_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE sovereign_ledger.priority_arbitration_log FORCE ROW LEVEL SECURITY;

-- Registro Forense Inmutable de Actos Ilegalizados y Proscritos (NIST CM-7 / EU AI Act Art. 5 / OFAC / AAUFSA)
CREATE TABLE IF NOT EXISTS sovereign_ledger.proscribed_violation_registry (
    proscription_id UUID PRIMARY KEY,
    tenant_id VARCHAR(32) NOT NULL REFERENCES sovereign_ledger.corporate_domains(tenant_id),
    domain_fqdn VARCHAR(128) NOT NULL,
    proscribed_vector VARCHAR(128) NOT NULL,
    statutory_citation VARCHAR(128) NOT NULL,
    enforcement_action VARCHAR(32) NOT NULL DEFAULT 'BLOCKED_FAIL_CLOSED',
    CONSTRAINT chk_enforcement_fail_closed CHECK (
        enforcement_action IN ('PROSCRIPTION_CLEARED_COMMITTED', 'BLOCKED_FAIL_CLOSED')
    )
);

CREATE OR REPLACE RULE rul_immutable_proscription_no_update AS ON UPDATE TO sovereign_ledger.proscribed_violation_registry DO INSTEAD NOTHING;
CREATE OR REPLACE RULE rul_immutable_proscription_no_delete AS ON DELETE TO sovereign_ledger.proscribed_violation_registry DO INSTEAD NOTHING;
ALTER TABLE sovereign_ledger.proscribed_violation_registry ENABLE ROW LEVEL SECURITY;
ALTER TABLE sovereign_ledger.proscribed_violation_registry FORCE ROW LEVEL SECURITY;

CREATE TABLE IF NOT EXISTS sovereign_ledger.gemini_enterprise_gateway_ledger (
    invocation_id UUID PRIMARY KEY,
    tenant_id VARCHAR(32) NOT NULL REFERENCES sovereign_ledger.corporate_domains(tenant_id),
    domain_fqdn VARCHAR(128) NOT NULL,
    github_repo_binding VARCHAR(128) NOT NULL,
    store_false_enforced BOOLEAN NOT NULL DEFAULT TRUE,
    prompt_shield_verified BOOLEAN NOT NULL DEFAULT TRUE,
    iso_42001_ai_bom_sealed BOOLEAN NOT NULL DEFAULT TRUE,
    tokens_consumed BIGINT NOT NULL,
    settled_micros NUMERIC(38, 0) NOT NULL,
    CONSTRAINT chk_gemini_zero_retention CHECK (store_false_enforced = TRUE AND prompt_shield_verified = TRUE),
    CONSTRAINT chk_gemini_positive_tokens CHECK (tokens_consumed > 0 AND settled_micros > 0)
);

CREATE OR REPLACE RULE rul_immutable_gemini_no_update AS ON UPDATE TO sovereign_ledger.gemini_enterprise_gateway_ledger DO INSTEAD NOTHING;
CREATE OR REPLACE RULE rul_immutable_gemini_no_delete AS ON DELETE TO sovereign_ledger.gemini_enterprise_gateway_ledger DO INSTEAD NOTHING;
ALTER TABLE sovereign_ledger.gemini_enterprise_gateway_ledger ENABLE ROW LEVEL SECURITY;
ALTER TABLE sovereign_ledger.gemini_enterprise_gateway_ledger FORCE ROW LEVEL SECURITY;
