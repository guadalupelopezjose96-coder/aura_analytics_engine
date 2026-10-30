-- =============================================================================
-- ESQUEMA TRANSACCIONAL DETERMINISTA DE DOBLE PARTIDA Y SEGURIDAD RLS
-- =============================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tabla de Dominios Corporativos Autorizados
CREATE TABLE IF NOT EXISTS corporate_domains (
    domain_id BIGINT PRIMARY KEY,
    domain_fqdn VARCHAR(255) NOT NULL UNIQUE,
    business_unit VARCHAR(100) NOT NULL,
    active_status BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO corporate_domains (domain_id, domain_fqdn, business_unit)
VALUES
    (877771073297, 'auratetrax.com', 'AuraTetrax Industrial Power'),
    (964521086569, 'fourphonix.com', 'Fourphonix Grid Intelligence'),
    (188680247367, 'auraintelcore.com', 'AuraIntelCore VPP & Fiscal Engine'),
    (1087754464927, 'auraintel-global.com', 'AuraIntel Global Services'),
    (282230018778, 'standard1979.com', 'Standard1979 Fiduciary Audit'),
    (358233233, 'prophonix.com', 'Prophonix Industrial Telemetry')
ON CONFLICT (domain_id) DO NOTHING;

-- Catálogo de Cuentas Contables (Chart of Accounts)
CREATE TABLE IF NOT EXISTS chart_of_accounts (
    account_code VARCHAR(64) PRIMARY KEY,
    account_name VARCHAR(255) NOT NULL,
    account_type VARCHAR(32) NOT NULL CHECK (account_type IN ('ASSET', 'LIABILITY', 'EQUITY', 'REVENUE', 'EXPENSE', 'FIDUCIARY_PAYABLE'))
);

INSERT INTO chart_of_accounts (account_code, account_name, account_type)
VALUES
    ('1010-CASH-STRIPE', 'Stripe Settlement Clearing', 'ASSET'),
    ('2010-TAX-INDIRECT-PAYABLE', 'Indirect Sales Tax / VAT Payable', 'LIABILITY'),
    ('2020-AAUFSA-OASI-PAYABLE', 'AAUFSA Sec. 4 OASI Social Security Trust Fund', 'FIDUCIARY_PAYABLE'),
    ('2030-AAUFSA-RESKILL-PAYABLE', 'AAUFSA Sec. 4 Workforce Reskilling Trust Fund', 'FIDUCIARY_PAYABLE'),
    ('4010-REVENUE-SAAS-CORE', 'Deterministic SaaS & Telemetry Revenue', 'REVENUE')
ON CONFLICT (account_code) DO NOTHING;

-- Registro Transaccional Inmutable
CREATE TABLE IF NOT EXISTS web_transactions (
    transaction_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    domain_id BIGINT NOT NULL REFERENCES corporate_domains(domain_id),
    delivery_model VARCHAR(32) NOT NULL CHECK (delivery_model IN ('ThirdPartyCloudSaas', 'InternalAutomation')),
    gross_amount_micro_usd BIGINT NOT NULL CHECK (gross_amount_micro_usd >= 0),
    indirect_tax_micro_usd BIGINT NOT NULL CHECK (indirect_tax_micro_usd >= 0),
    aaufsa_tax_micro_usd BIGINT NOT NULL CHECK (aaufsa_tax_micro_usd >= 0),
    clawback_micro_usd BIGINT NOT NULL CHECK (clawback_micro_usd >= 0),
    safe_harbor_applied BOOLEAN NOT NULL DEFAULT FALSE,
    cloud_exempt_applied BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Libro Mayor Contable de Doble Partida (General Ledger)
CREATE TABLE IF NOT EXISTS journal_entries (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES web_transactions(transaction_id),
    domain_id BIGINT NOT NULL REFERENCES corporate_domains(domain_id),
    account_code VARCHAR(64) NOT NULL REFERENCES chart_of_accounts(account_code),
    debit_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (debit_micro_usd >= 0),
    credit_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (credit_micro_usd >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_debit_or_credit CHECK (
        (debit_micro_usd > 0 AND credit_micro_usd = 0) OR
        (credit_micro_usd > 0 AND debit_micro_usd = 0)
    )
);

-- -----------------------------------------------------------------------------
-- PROTECCIÓN WORM: REGLAS INMUTABLES (DO INSTEAD NOTHING)
-- -----------------------------------------------------------------------------
CREATE OR REPLACE RULE rule_web_transactions_no_update AS
    ON UPDATE TO web_transactions DO INSTEAD NOTHING;

CREATE OR REPLACE RULE rule_web_transactions_no_delete AS
    ON DELETE TO web_transactions DO INSTEAD NOTHING;

CREATE OR REPLACE RULE rule_journal_entries_no_update AS
    ON UPDATE TO journal_entries DO INSTEAD NOTHING;

CREATE OR REPLACE RULE rule_journal_entries_no_delete AS
    ON DELETE TO journal_entries DO INSTEAD NOTHING;

-- -----------------------------------------------------------------------------
-- ROW-LEVEL SECURITY (RLS) MULTI-TENANT POR DOMINIO CORPORATIVO
-- -----------------------------------------------------------------------------
ALTER TABLE web_transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE journal_entries ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS rls_web_transactions_isolation ON web_transactions;
CREATE POLICY rls_web_transactions_isolation ON web_transactions
    AS RESTRICTIVE
    USING (domain_id = NULLIF(current_setting('app.current_domain_id', true), '')::BIGINT);

DROP POLICY IF EXISTS rls_journal_entries_isolation ON journal_entries;
CREATE POLICY rls_journal_entries_isolation ON journal_entries
    AS RESTRICTIVE
    USING (domain_id = NULLIF(current_setting('app.current_domain_id', true), '')::BIGINT);

-- -----------------------------------------------------------------------------
-- TRIGGER DE VERIFICACIÓN DE EQUILIBRIO DE DOBLE PARTIDA (SUM(D) - SUM(C) = 0)
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION verify_transaction_balance()
RETURNS TRIGGER AS $$
DECLARE
    v_total_debit BIGINT;
    v_total_credit BIGINT;
BEGIN
    SELECT COALESCE(SUM(debit_micro_usd), 0), COALESCE(SUM(credit_micro_usd), 0)
    INTO v_total_debit, v_total_credit
    FROM journal_entries
    WHERE transaction_id = NEW.transaction_id;

    IF v_total_debit <> v_total_credit THEN
        RAISE EXCEPTION 'Fallo de integridad de doble partida: Total Debitos (%) <> Total Creditos (%) para transaccion %',
            v_total_debit, v_total_credit, NEW.transaction_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger deferred al finalizar la transacción
CREATE OR REPLACE TRIGGER trg_verify_double_entry_balance
    AFTER INSERT ON journal_entries
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION verify_transaction_balance();
