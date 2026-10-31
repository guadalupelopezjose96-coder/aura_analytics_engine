-- =============================================================================
-- AURAINTEL VPP & ENERGETICS TRANSACTIVE LEDGER (POSTGRESQL 16+ SERIALIZABLE)
-- Organización Holding: guadalupelopezjose96-org (320717657398)
-- Cumplimiento: FERC Order 2222, IEEE 1547-2018, NERC CIP, IRC Chapter 56 (AAUFSA)
-- =============================================================================

BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE;

ALTER DATABASE CURRENT SET default_transaction_isolation TO 'serializable';

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Tabla de Telemetría Sincrofasorial y Eventos de Maniobra de Interruptores (WORM)
CREATE TABLE IF NOT EXISTS vpp_synchrophasor_dispatch_events (
    event_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    holding_org_id VARCHAR(32) NOT NULL DEFAULT '320717657398' CHECK (holding_org_id = '320717657398'),
    domain_id VARCHAR(32) NOT NULL CHECK (domain_id IN (
        '877771073297', -- AuraTetrax-com
        '964521086569', -- Fourphonix-com
        '188680247367', -- AuraIntelCore-com
        '1087754464927', -- AuraIntel-Global-com
        '282230018778', -- Standard1979-com
        '358233233'     -- Prophonix-com
    )),
    lifecycle_folder_id VARCHAR(32) NOT NULL CHECK (lifecycle_folder_id IN (
        '938478392755', -- 01-Shared-Services
        '978426653396', -- 02-Production
        '467686749558', -- 03-Non-Production
        '10470744625'   -- 04-Sandbox
    )),
    vpp_resource_id VARCHAR(128) NOT NULL,
    grid_freq_uhz BIGINT NOT NULL CHECK (grid_freq_uhz BETWEEN 59950000 AND 60050000),
    grid_voltage_micro_pu BIGINT NOT NULL CHECK (grid_voltage_micro_pu BETWEEN 950000 AND 1050000),
    phase_angle_diff_udeg BIGINT NOT NULL CHECK (ABS(phase_angle_diff_udeg) <= 10000000),
    rocof_uhz_per_sec BIGINT NOT NULL CHECK (ABS(rocof_uhz_per_sec) <= 500000),
    ptp_time_skew_ns BIGINT NOT NULL CHECK (ptp_time_skew_ns <= 1000),
    dispatched_active_power_uw BIGINT NOT NULL,
    dispatched_reactive_power_uvar BIGINT NOT NULL,
    dispatched_energy_uwh BIGINT NOT NULL CHECK (dispatched_energy_uwh >= 0),
    primary_operator_id VARCHAR(128) NOT NULL,
    secondary_engineer_id VARCHAR(128) NOT NULL CHECK (primary_operator_id <> secondary_engineer_id),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    event_sha256 CHAR(64) NOT NULL
);

-- 2. Libro Mayor de Energía Transaccional y Liquidación LMP / AAUFSA (Cabecera)
CREATE TABLE IF NOT EXISTS energetics_settlement_entries (
    entry_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    idempotency_key VARCHAR(128) NOT NULL UNIQUE,
    domain_id VARCHAR(32) NOT NULL CHECK (domain_id IN (
        '877771073297', '964521086569', '188680247367',
        '1087754464927', '282230018778', '358233233'
    )),
    iso_rto_node_id VARCHAR(64) NOT NULL,
    dispatched_energy_uwh BIGINT NOT NULL CHECK (dispatched_energy_uwh >= 0),
    lmp_price_per_mwh_micro_usd BIGINT NOT NULL CHECK (lmp_price_per_mwh_micro_usd >= 0),
    irc_45y_ptc_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (irc_45y_ptc_micro_usd >= 0),
    aaufsa_sec_5000f_tax_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (aaufsa_sec_5000f_tax_micro_usd >= 0),
    aaufsa_oasdi_75_pct_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (aaufsa_oasdi_75_pct_micro_usd >= 0),
    aaufsa_reskilling_25_pct_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (aaufsa_reskilling_25_pct_micro_usd >= 0),
    aaufsa_statutory_basis VARCHAR(256) NOT NULL,
    posted_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT chk_aaufsa_trust_fund_split CHECK (
        aaufsa_sec_5000f_tax_micro_usd = (aaufsa_oasdi_75_pct_micro_usd + aaufsa_reskilling_25_pct_micro_usd)
    )
);

-- 3. Líneas de Partida Doble del Libro Mayor Energético
CREATE TABLE IF NOT EXISTS energetics_settlement_lines (
    line_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entry_id UUID NOT NULL REFERENCES energetics_settlement_entries(entry_id) ON DELETE RESTRICT,
    domain_id VARCHAR(32) NOT NULL CHECK (domain_id IN (
        '877771073297', '964521086569', '188680247367',
        '1087754464927', '282230018778', '358233233'
    )),
    account_code VARCHAR(64) NOT NULL,
    debit_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (debit_micro_usd >= 0),
    credit_micro_usd BIGINT NOT NULL DEFAULT 0 CHECK (credit_micro_usd >= 0),
    CONSTRAINT chk_non_zero_exclusive_leg CHECK (
        (debit_micro_usd > 0 AND credit_micro_usd = 0) OR
        (credit_micro_usd > 0 AND debit_micro_usd = 0)
    )
);

-- 4. Reglas WORM Inmutables (Write-Once-Read-Many) - DO INSTEAD NOTHING
CREATE OR REPLACE RULE worm_no_update_vpp_events AS
    ON UPDATE TO vpp_synchrophasor_dispatch_events DO INSTEAD NOTHING;
CREATE OR REPLACE RULE worm_no_delete_vpp_events AS
    ON DELETE TO vpp_synchrophasor_dispatch_events DO INSTEAD NOTHING;

CREATE OR REPLACE RULE worm_no_update_settlement_entries AS
    ON UPDATE TO energetics_settlement_entries DO INSTEAD NOTHING;
CREATE OR REPLACE RULE worm_no_delete_settlement_entries AS
    ON DELETE TO energetics_settlement_entries DO INSTEAD NOTHING;

CREATE OR REPLACE RULE worm_no_update_settlement_lines AS
    ON UPDATE TO energetics_settlement_lines DO INSTEAD NOTHING;
CREATE OR REPLACE RULE worm_no_delete_settlement_lines AS
    ON DELETE TO energetics_settlement_lines DO INSTEAD NOTHING;

-- 5. Trigger Diferido de Verificación Estricta de Partida Doble (Sum(Debits) - Sum(Credits) = 0)
CREATE OR REPLACE FUNCTION verify_energetics_double_entry_balance()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_total_debits BIGINT;
    v_total_credits BIGINT;
    v_line_count INT;
BEGIN
    SELECT
        COALESCE(SUM(debit_micro_usd), 0),
        COALESCE(SUM(credit_micro_usd), 0),
        COUNT(*)
    INTO v_total_debits, v_total_credits, v_line_count
    FROM energetics_settlement_lines
    WHERE entry_id = NEW.entry_id;

    IF v_line_count < 2 THEN
        RAISE EXCEPTION 'VIOLACION_PARTIDA_DOBLE: El asiento % requiere al menos 2 lineas contables.', NEW.entry_id;
    END IF;

    IF (v_total_debits - v_total_credits) <> 0 THEN
        RAISE EXCEPTION 'DESBALANCE_CONTABLE_ENERGETICO: Asiento % presenta Debitos (%) != Creditos (%). Delta debe ser 0.',
            NEW.entry_id, v_total_debits, v_total_credits;
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_enforce_zero_balance_energetics ON energetics_settlement_lines;
CREATE CONSTRAINT TRIGGER trg_enforce_zero_balance_energetics
AFTER INSERT ON energetics_settlement_lines
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
EXECUTE FUNCTION verify_energetics_double_entry_balance();

-- 6. Aislamiento Multi-Tenant Estricto mediante Row-Level Security (RLS)
ALTER TABLE vpp_synchrophasor_dispatch_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE vpp_synchrophasor_dispatch_events FORCE ROW LEVEL SECURITY;

ALTER TABLE energetics_settlement_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE energetics_settlement_entries FORCE ROW LEVEL SECURITY;

ALTER TABLE energetics_settlement_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE energetics_settlement_lines FORCE ROW LEVEL SECURITY;

CREATE POLICY rls_vpp_events_tenant_isolation ON vpp_synchrophasor_dispatch_events
    USING (domain_id = current_setting('app.current_domain_id', true))
    WITH CHECK (domain_id = current_setting('app.current_domain_id', true));

CREATE POLICY rls_settlement_entries_tenant_isolation ON energetics_settlement_entries
    USING (domain_id = current_setting('app.current_domain_id', true))
    WITH CHECK (domain_id = current_setting('app.current_domain_id', true));

CREATE POLICY rls_settlement_lines_tenant_isolation ON energetics_settlement_lines
    USING (domain_id = current_setting('app.current_domain_id', true))
    WITH CHECK (domain_id = current_setting('app.current_domain_id', true));

COMMIT;
