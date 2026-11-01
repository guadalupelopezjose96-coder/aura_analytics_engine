#![forbid(unsafe_code)]
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    dead_code,
    unused_imports,
    unused_variables
)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::unreadable_literal,
    clippy::enum_clike_unportable_variant
)]
#![allow(clippy::similar_names, clippy::doc_markdown)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;

// ============================================================================
// CLAUSULA 3: ATRIBUCION E INTEGRIDAD ESTATUTARIA INMUTABLE
// ============================================================================
pub const AAUFSA_STATUTE_TITLE: &str =
    "Automated Asset Utilization and Fiscal Solvency Act of 2026 (IRC Chapter 56)";
pub const AAUFSA_AUTHOR: &str = "Jose Emanuel Guadalupe Lopez";
pub const AAUFSA_ORG_ID: u64 = 320_717_657_398;

// ============================================================================
// AISLAMIENTO CELULAR: 6 DOMINIOS CORPORATIVOS (ORG ID: 320717657398)
// ============================================================================
pub const DOMAIN_AURATETRAX: u64 = 877_771_073_297;
pub const DOMAIN_FOURPHONIX: u64 = 964_521_086_569;
pub const DOMAIN_AURAINTELCORE: u64 = 188_680_247_367;
pub const DOMAIN_AURAINTEL_GLOBAL: u64 = 1_087_754_464_927;
pub const DOMAIN_STANDARD1979: u64 = 282_230_018_778;
pub const DOMAIN_PROPHONIX: u64 = 358_233_233;

pub const VALID_DOMAINS: [u64; 6] = [
    DOMAIN_AURATETRAX,
    DOMAIN_FOURPHONIX,
    DOMAIN_AURAINTELCORE,
    DOMAIN_AURAINTEL_GLOBAL,
    DOMAIN_STANDARD1979,
    DOMAIN_PROPHONIX,
];

// ============================================================================
// CONSTANTES FISCALES AAUFSA (PUNTO FIJO EN CENTAVOS Y BASIS POINTS)
// ============================================================================
pub const AAUFSA_RATE_BPS: u64 = 1_530; // 15.30% Sec. 5000F (Paridad FICA)
pub const CLAWBACK_SURCHARGE_BPS: u64 = 200; // 2.00% Recargo Sec. 5000G(c)
pub const BPS_DIVISOR: u64 = 10_000;
pub const SMALL_BUSINESS_EXEMPTION_CENTS: u64 = 500_000_000; // $5,000,000.00 USD Sec. 5000G(b)
pub const SAFE_HARBOR_MIN_MONTHS: u32 = 36; // 36 meses Sec. 5000G(a)
pub const DISPLACEMENT_MIN_DAYS: u32 = 90; // > 90 dias Sec. 5000H(c)

// ============================================================================
// ESTRUCTURAS DE DATOS Y TAXONOMIA DE 14 DIMENSIONES
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplacementType {
    DirectReplacement,
    AlgorithmicConsolidation,
    AutonomousOffshoring,
    SyntheticWorkforceExpansion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatutoryCategory {
    TaxableSec5000F,
    ExemptSafeHarborOrImmunity,
    ExemptSmallBusinessSec5000Gb,
    NonSubjectSec5000Hc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VppDispatchCommand {
    NominalGridOperation,
    FrequencyDroopCorrection,
    HighLoadCurtailmentWarning,
    EmergencyBreakerTrip,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplacementTaxonomy14D {
    pub d01_cognitive_automation: bool,
    pub d02_robotic_actuation: bool,
    pub d03_predictive_scheduling: bool,
    pub d04_autonomous_logistics: bool,
    pub d05_synthetic_customer_agent: bool,
    pub d06_algorithmic_underwriting: bool,
    pub d07_automated_code_synthesis: bool,
    pub d08_vpp_grid_autonomy: bool,
    pub d09_zero_touch_provisioning: bool,
    pub d10_continuous_compliance_audit: bool,
    pub d11_automated_treasury_routing: bool,
    pub d12_neural_quality_control: bool,
    pub d13_autonomous_cyber_defense: bool,
    pub d14_generative_design_execution: bool,
}

#[must_use]
pub const fn default_taxonomy() -> DisplacementTaxonomy14D {
    DisplacementTaxonomy14D {
        d01_cognitive_automation: true,
        d02_robotic_actuation: false,
        d03_predictive_scheduling: true,
        d04_autonomous_logistics: false,
        d05_synthetic_customer_agent: false,
        d06_algorithmic_underwriting: false,
        d07_automated_code_synthesis: true,
        d08_vpp_grid_autonomy: true,
        d09_zero_touch_provisioning: true,
        d10_continuous_compliance_audit: true,
        d11_automated_treasury_routing: true,
        d12_neural_quality_control: false,
        d13_autonomous_cyber_defense: true,
        d14_generative_design_execution: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainGrossReceipts {
    pub domain_id: u64,
    pub annual_gross_receipts_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationExpansionEvent {
    pub domain_id: u64,
    pub asset_id: [u8; 16],
    pub is_cloud_provider_infrastructure: bool,
    pub displacement_type: DisplacementType,
    pub taxonomy: DisplacementTaxonomy14D,
    pub displacement_duration_days: u32,
    pub baseline_payroll_cents: u64,
    pub safe_harbor_reassignment_months: u32,
    pub safe_harbor_breached_clawback: bool,
    pub consolidated_group_revenue_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxAssessmentResult {
    pub domain_id: u64,
    pub statutory_category: StatutoryCategory,
    pub displacement_type: DisplacementType,
    pub is_taxable: bool,
    pub safe_harbor_applied: bool,
    pub clawback_triggered: bool,
    pub displaced_payroll_cents: u64,
    pub base_tax_cents: u64,
    pub clawback_surcharge_cents: u64,
    pub total_tax_amount_cents: u64,
    pub social_security_allocation_cents: u64,
    pub workforce_reskilling_allocation_cents: u64,
    pub statutory_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VppBusState {
    pub bus_id: u32,
    pub current_mw: u64,
    pub max_mw: u64,
    pub current_ma: u64,
    pub max_ma: u64,
    pub grid_frequency_chz: u32, // 6000 = 60.00 Hz
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerNodeAddition {
    pub node_id: [u8; 16],
    pub domain_id: u64,
    pub injection_capacity_mw: u64,
    pub rated_current_ma: u64,
}

// ============================================================================
// MODULO 1, 2 Y 3: TELEMETRIA JSON, DESPACHADOR VPP Y PERSISTENCIA SQL/SHA256
// ============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveTelemetryPacket {
    pub cycle_id: u64,
    pub previous_ledger_hash: String,
    pub grid_frequency_chz: u32,
    pub der_node: DerNodeAddition,
    pub fiscal_event: AutomationExpansionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveOperationalReceipt {
    pub cycle_id: u64,
    pub organization_id: u64,
    pub domain_id: u64,
    pub dispatch_command: VppDispatchCommand,
    pub updated_bus_mw: u64,
    pub grid_frequency_chz: u32,
    pub assessment: TaxAssessmentResult,
    pub previous_ledger_hash: String,
    pub current_ledger_hash: String,
    pub sql_transaction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidDomain(u64),
    ArithmeticOverflow,
    VppOverCapacity { requested_mw: u64, max_mw: u64 },
    VppOverCurrent { requested_ma: u64, max_ma: u64 },
    GridFrequencyInstability(u32),
    SerializationError(String),
}

pub struct CapacityAndFiscalEngine;

impl CapacityAndFiscalEngine {
    /// Valida que el dominio pertenezca a los 6 dominios autorizados de la Org 320717657398.
    ///
    /// # Errors
    /// Retorna `EngineError::InvalidDomain` si el ID no pertenece a `VALID_DOMAINS`.
    pub fn validate_domain(domain_id: u64) -> Result<(), EngineError> {
        if VALID_DOMAINS.contains(&domain_id) {
            Ok(())
        } else {
            Err(EngineError::InvalidDomain(domain_id))
        }
    }

    /// Consolida los ingresos brutos del grupo controlado bajo IRC Sec. 414 / 1563.
    ///
    /// # Errors
    /// Retorna error si algún dominio es inválido o si ocurre desbordamiento aritmético.
    pub fn aggregate_controlled_group_receipts(
        receipts: &[DomainGrossReceipts],
    ) -> Result<u64, EngineError> {
        let mut total: u64 = 0;
        for item in receipts {
            Self::validate_domain(item.domain_id)?;
            total = total
                .checked_add(item.annual_gross_receipts_cents)
                .ok_or(EngineError::ArithmeticOverflow)?;
        }
        Ok(total)
    }

    /// Evalúa la integración de un nodo DER en el bus de `AuraIntel` VPP.
    ///
    /// # Errors
    /// Retorna error si el dominio es inválido, la frecuencia está fuera de rango o se excede capacidad.
    pub fn evaluate_der_expansion(
        bus: &mut VppBusState,
        der: &DerNodeAddition,
    ) -> Result<u64, EngineError> {
        Self::validate_domain(der.domain_id)?;

        if bus.grid_frequency_chz < 5_950 || bus.grid_frequency_chz > 6_050 {
            return Err(EngineError::GridFrequencyInstability(
                bus.grid_frequency_chz,
            ));
        }

        let new_mw = bus
            .current_mw
            .checked_add(der.injection_capacity_mw)
            .ok_or(EngineError::ArithmeticOverflow)?;
        if new_mw > bus.max_mw {
            return Err(EngineError::VppOverCapacity {
                requested_mw: new_mw,
                max_mw: bus.max_mw,
            });
        }

        let new_ma = bus
            .current_ma
            .checked_add(der.rated_current_ma)
            .ok_or(EngineError::ArithmeticOverflow)?;
        if new_ma > bus.max_ma {
            return Err(EngineError::VppOverCurrent {
                requested_ma: new_ma,
                max_ma: bus.max_ma,
            });
        }

        bus.current_mw = new_mw;
        bus.current_ma = new_ma;
        Ok(bus.current_mw)
    }

    /// Determina la orden de despacho activo de `AuraIntel` VPP según carga y frecuencia.
    #[must_use]
    pub const fn compute_dispatch_command(bus: &VppBusState) -> VppDispatchCommand {
        if bus.grid_frequency_chz < 5_950 || bus.grid_frequency_chz > 6_050 {
            VppDispatchCommand::EmergencyBreakerTrip
        } else if bus.grid_frequency_chz < 5_985 || bus.grid_frequency_chz > 6_015 {
            VppDispatchCommand::FrequencyDroopCorrection
        } else if bus.current_mw * 100 >= bus.max_mw * 85 {
            VppDispatchCommand::HighLoadCurtailmentWarning
        } else {
            VppDispatchCommand::NominalGridOperation
        }
    }

    /// Evalúa un evento de automatización bajo el estatuto AAUFSA (IRC Chapter 56).
    ///
    /// # Errors
    /// Retorna error si el dominio no está autorizado o si hay desbordamiento en punto fijo.
    pub fn evaluate_aaufsa_expansion(
        event: &AutomationExpansionEvent,
    ) -> Result<TaxAssessmentResult, EngineError> {
        Self::validate_domain(event.domain_id)?;

        // 1. Inmunidad del Desarrollador / Proveedor Cloud & SaaS (Sec. 5000H(d))
        if event.is_cloud_provider_infrastructure {
            return Ok(TaxAssessmentResult {
                domain_id: event.domain_id,
                statutory_category: StatutoryCategory::ExemptSafeHarborOrImmunity,
                displacement_type: event.displacement_type,
                is_taxable: false,
                safe_harbor_applied: true,
                clawback_triggered: false,
                displaced_payroll_cents: 0,
                base_tax_cents: 0,
                clawback_surcharge_cents: 0,
                total_tax_amount_cents: 0,
                social_security_allocation_cents: 0,
                workforce_reskilling_allocation_cents: 0,
                statutory_basis:
                    "Exento bajo Sec. 5000H(d): Inmunidad del Proveedor Cloud/SaaS ($0.00 USD)"
                        .to_string(),
            });
        }

        // 2. Exención de Pequeña Empresa <= $5,000,000.00 USD (Sec. 5000G(b))
        if event.consolidated_group_revenue_cents <= SMALL_BUSINESS_EXEMPTION_CENTS {
            return Ok(TaxAssessmentResult {
                domain_id: event.domain_id,
                statutory_category: StatutoryCategory::ExemptSmallBusinessSec5000Gb,
                displacement_type: event.displacement_type,
                is_taxable: false,
                safe_harbor_applied: true,
                clawback_triggered: false,
                displaced_payroll_cents: event.baseline_payroll_cents,
                base_tax_cents: 0,
                clawback_surcharge_cents: 0,
                total_tax_amount_cents: 0,
                social_security_allocation_cents: 0,
                workforce_reskilling_allocation_cents: 0,
                statutory_basis:
                    "Exento bajo Sec. 5000G(b): Ingresos consolidados <= $5,000,000.00 USD"
                        .to_string(),
            });
        }

        // 3. No sujeción por duración <= 90 días (Sec. 5000H(c))
        if event.displacement_duration_days <= DISPLACEMENT_MIN_DAYS {
            return Ok(TaxAssessmentResult {
                domain_id: event.domain_id,
                statutory_category: StatutoryCategory::NonSubjectSec5000Hc,
                displacement_type: event.displacement_type,
                is_taxable: false,
                safe_harbor_applied: false,
                clawback_triggered: false,
                displaced_payroll_cents: event.baseline_payroll_cents,
                base_tax_cents: 0,
                clawback_surcharge_cents: 0,
                total_tax_amount_cents: 0,
                social_security_allocation_cents: 0,
                workforce_reskilling_allocation_cents: 0,
                statutory_basis: "No sujeto bajo Sec. 5000H(c): Desplazamiento temporal <= 90 dias"
                    .to_string(),
            });
        }

        // 4. Safe Harbor por Reasignación Laboral >= 36 meses sin ruptura (Sec. 5000G(a))
        if event.safe_harbor_reassignment_months >= SAFE_HARBOR_MIN_MONTHS
            && !event.safe_harbor_breached_clawback
        {
            return Ok(TaxAssessmentResult {
                domain_id: event.domain_id,
                statutory_category: StatutoryCategory::ExemptSafeHarborOrImmunity,
                displacement_type: event.displacement_type,
                is_taxable: false,
                safe_harbor_applied: true,
                clawback_triggered: false,
                displaced_payroll_cents: event.baseline_payroll_cents,
                base_tax_cents: 0,
                clawback_surcharge_cents: 0,
                total_tax_amount_cents: 0,
                social_security_allocation_cents: 0,
                workforce_reskilling_allocation_cents: 0,
                statutory_basis: "Exento: Reasignacion efectiva >= 36 meses bajo Sec. 5000G(a)"
                    .to_string(),
            });
        }

        // 5. Cálculo del Impuesto Base 15.30% (Sec. 5000F) y Recaptura Clawback 2.00% (Sec. 5000G(c))
        let base_tax_cents = event
            .baseline_payroll_cents
            .checked_mul(AAUFSA_RATE_BPS)
            .ok_or(EngineError::ArithmeticOverflow)?
            / BPS_DIVISOR;

        let clawback_surcharge_cents = if event.safe_harbor_breached_clawback {
            base_tax_cents
                .checked_mul(CLAWBACK_SURCHARGE_BPS)
                .ok_or(EngineError::ArithmeticOverflow)?
                / BPS_DIVISOR
        } else {
            0
        };

        let total_tax_amount_cents = base_tax_cents
            .checked_add(clawback_surcharge_cents)
            .ok_or(EngineError::ArithmeticOverflow)?;

        // Bifurcación Fiduciaria Exacta: 75% Seguro Social / 25% Recalificación Laboral (Sec. 4)
        let social_security_allocation_cents = total_tax_amount_cents
            .checked_mul(75)
            .ok_or(EngineError::ArithmeticOverflow)?
            / 100;
        let workforce_reskilling_allocation_cents = total_tax_amount_cents
            .checked_sub(social_security_allocation_cents)
            .ok_or(EngineError::ArithmeticOverflow)?;

        let statutory_basis = if event.safe_harbor_breached_clawback {
            "Gravamen + Recaptura Clawback: 15.30% Sec. 5000F + 2.00% Recargo Sec. 5000G(c)"
                .to_string()
        } else {
            "Sujeto a gravamen: 15.30% AAUFSA Sec. 5000F liquidable".to_string()
        };

        Ok(TaxAssessmentResult {
            domain_id: event.domain_id,
            statutory_category: StatutoryCategory::TaxableSec5000F,
            displacement_type: event.displacement_type,
            is_taxable: true,
            safe_harbor_applied: false,
            clawback_triggered: event.safe_harbor_breached_clawback,
            displaced_payroll_cents: event.baseline_payroll_cents,
            base_tax_cents,
            clawback_surcharge_cents,
            total_tax_amount_cents,
            social_security_allocation_cents,
            workforce_reskilling_allocation_cents,
            statutory_basis,
        })
    }

    /// Procesa un paquete de telemetría en vivo, actualiza el bus, encadena SHA-256 y emite SQL ACID.
    ///
    /// # Errors
    /// Retorna error si falla la validación eléctrica, fiscal o la serialización JSON.
    pub fn process_live_telemetry_packet(
        bus: &mut VppBusState,
        packet: &LiveTelemetryPacket,
    ) -> Result<LiveOperationalReceipt, EngineError> {
        bus.grid_frequency_chz = packet.grid_frequency_chz;
        let updated_bus_mw = Self::evaluate_der_expansion(bus, &packet.der_node)?;
        let dispatch_command = Self::compute_dispatch_command(bus);
        let assessment = Self::evaluate_aaufsa_expansion(&packet.fiscal_event)?;

        let mut hasher = Sha256::new();
        hasher.update(packet.previous_ledger_hash.as_bytes());
        hasher.update(packet.cycle_id.to_le_bytes());
        hasher.update(assessment.domain_id.to_le_bytes());
        hasher.update(updated_bus_mw.to_le_bytes());
        hasher.update(assessment.total_tax_amount_cents.to_le_bytes());
        let current_ledger_hash = hex::encode(hasher.finalize());

        let sql_transaction = format!(
            "BEGIN ISOLATION LEVEL SERIALIZABLE; \
             SET LOCAL app.current_domain_id = '{}'; \
             INSERT INTO aaufsa_vpp_operational_ledger \
             (cycle_id, org_id, domain_id, bus_mw, grid_freq_chz, dispatch_cmd, \
              displaced_payroll_cents, total_tax_cents, oasi_trust_cents, reskilling_trust_cents, \
              prev_hash, record_hash) \
             VALUES ({}, {}, {}, {}, {}, '{:?}', {}, {}, {}, {}, '{}', '{}'); \
             COMMIT;",
            assessment.domain_id,
            packet.cycle_id,
            AAUFSA_ORG_ID,
            assessment.domain_id,
            updated_bus_mw,
            bus.grid_frequency_chz,
            dispatch_command,
            assessment.displaced_payroll_cents,
            assessment.total_tax_amount_cents,
            assessment.social_security_allocation_cents,
            assessment.workforce_reskilling_allocation_cents,
            packet.previous_ledger_hash,
            current_ledger_hash
        );

        Ok(LiveOperationalReceipt {
            cycle_id: packet.cycle_id,
            organization_id: AAUFSA_ORG_ID,
            domain_id: assessment.domain_id,
            dispatch_command,
            updated_bus_mw,
            grid_frequency_chz: bus.grid_frequency_chz,
            assessment,
            previous_ledger_hash: packet.previous_ledger_hash.clone(),
            current_ledger_hash,
            sql_transaction,
        })
    }
}

fn format_usd(cents: u64) -> String {
    let dollars = cents / 100;
    let rem_cents = cents % 100;
    format!("${dollars}.{rem_cents:02} USD")
}

fn print_case(title: &str, res: &TaxAssessmentResult) {
    println!("==================================================================");
    println!("{title}");
    println!("==================================================================");
    println!(" - Dominio ID: {}", res.domain_id);
    println!(" - Categoria Estatutaria: {:?}", res.statutory_category);
    println!(" - Especie de Desplazamiento: {:?}", res.displacement_type);
    println!(
        " - Nomina Efectiva Desplazada: {}",
        format_usd(res.displaced_payroll_cents)
    );
    println!(" - Sujeto a Gravamen: {}", res.is_taxable);
    println!(
        " - Monto Impuesto Total: {}",
        format_usd(res.total_tax_amount_cents)
    );
    println!(" - Base Estatutaria: {}", res.statutory_basis);
    if res.is_taxable {
        if res.clawback_triggered {
            println!(
                "   * Impuesto Base (15.30%): {} | Recargo Clawback (2.00%): {}",
                format_usd(res.base_tax_cents),
                format_usd(res.clawback_surcharge_cents)
            );
        }
        println!(
            "   * Fondo Seguro Social (75%): {}",
            format_usd(res.social_security_allocation_cents)
        );
        println!(
            "   * Fondo Recalificacion (25%): {}",
            format_usd(res.workforce_reskilling_allocation_cents)
        );
    }
    println!();
}

fn run_live_json_ingestion(json_payload: &str) -> Result<(), String> {
    let state_path = "audit_logs/vpp_bus_state.json";
    let sql_queue_path = "audit_logs/vpp_sql_ledger_queue.sql";
    let _ = fs::create_dir_all("audit_logs");

    let mut bus: VppBusState = if let Ok(data) = fs::read_to_string(state_path) {
        serde_json::from_str(&data).map_err(|e| format!("Error leyendo estado VPP: {e}"))?
    } else {
        VppBusState {
            bus_id: 1,
            current_mw: 45_000,
            max_mw: 250_000,
            current_ma: 120_000,
            max_ma: 900_000,
            grid_frequency_chz: 6_000,
        }
    };

    let packet: LiveTelemetryPacket = serde_json::from_str(json_payload)
        .map_err(|e| format!("Error parseando JSON de telemetria: {e}"))?;

    let receipt = CapacityAndFiscalEngine::process_live_telemetry_packet(&mut bus, &packet)
        .map_err(|e| format!("Fallo en motor VPP/AAUFSA: {e:?}"))?;

    let updated_state_json =
        serde_json::to_string_pretty(&bus).map_err(|e| format!("Error serializando bus: {e}"))?;
    fs::write(state_path, updated_state_json)
        .map_err(|e| format!("Error guardando estado VPP: {e}"))?;

    let mut sql_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(sql_queue_path)
        .map_err(|e| format!("Error abriendo cola SQL: {e}"))?;
    writeln!(sql_file, "{}", receipt.sql_transaction)
        .map_err(|e| format!("Error escribiendo sentencia SQL: {e}"))?;

    let receipt_json = serde_json::to_string_pretty(&receipt)
        .map_err(|e| format!("Error serializando recibo operativo: {e}"))?;
    println!("{receipt_json}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "--ingest-json" {
        if let Err(err) = run_live_json_ingestion(&args[2]) {
            eprintln!("[ERROR CRITICO VPP LIVE]: {err}");
            std::process::exit(1);
        }
        return;
    }

    println!("==================================================================");
    println!("ESTATUTO: {AAUFSA_STATUTE_TITLE}");
    println!("AUTOR EXCLUSIVO (Clausula 3): {AAUFSA_AUTHOR} | ORG: {AAUFSA_ORG_ID}");
    println!("==================================================================");

    let group_receipts = [
        DomainGrossReceipts {
            domain_id: DOMAIN_AURATETRAX,
            annual_gross_receipts_cents: 250_000_000,
        },
        DomainGrossReceipts {
            domain_id: DOMAIN_FOURPHONIX,
            annual_gross_receipts_cents: 180_000_000,
        },
        DomainGrossReceipts {
            domain_id: DOMAIN_AURAINTELCORE,
            annual_gross_receipts_cents: 150_000_000,
        },
        DomainGrossReceipts {
            domain_id: DOMAIN_AURAINTEL_GLOBAL,
            annual_gross_receipts_cents: 120_000_000,
        },
        DomainGrossReceipts {
            domain_id: DOMAIN_STANDARD1979,
            annual_gross_receipts_cents: 90_000_000,
        },
        DomainGrossReceipts {
            domain_id: DOMAIN_PROPHONIX,
            annual_gross_receipts_cents: 60_000_000,
        },
    ];

    let Ok(consolidated_cents) =
        CapacityAndFiscalEngine::aggregate_controlled_group_receipts(&group_receipts)
    else {
        return;
    };

    println!(
        "Ingresos Consolidados Grupo Controlado (6 Dominios): {}",
        format_usd(consolidated_cents)
    );

    let mut vpp_bus = VppBusState {
        bus_id: 1,
        current_mw: 45_000,
        max_mw: 100_000,
        current_ma: 120_000,
        max_ma: 500_000,
        grid_frequency_chz: 6_000,
    };

    let der_node = DerNodeAddition {
        node_id: [1u8; 16],
        domain_id: DOMAIN_AURAINTEL_GLOBAL,
        injection_capacity_mw: 15_000,
        rated_current_ma: 40_000,
    };

    if let Ok(new_mw) = CapacityAndFiscalEngine::evaluate_der_expansion(&mut vpp_bus, &der_node) {
        println!("DER Integrado Exitosamente en AuraIntel VPP. Capacidad Bus: {new_mw} mW\n");
    }

    let case1_saas = AutomationExpansionEvent {
        domain_id: DOMAIN_AURAINTELCORE,
        asset_id: [0xAA; 16],
        is_cloud_provider_infrastructure: true,
        displacement_type: DisplacementType::DirectReplacement,
        taxonomy: default_taxonomy(),
        displacement_duration_days: 365,
        baseline_payroll_cents: 12_000_000,
        safe_harbor_reassignment_months: 0,
        safe_harbor_breached_clawback: false,
        consolidated_group_revenue_cents: consolidated_cents,
    };
    if let Ok(res1) = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&case1_saas) {
        print_case(
            "CASO 1: Elaboracion y Expansion de SaaS (EXENTO Sec. 5000H(d) - $0.00 USD)",
            &res1,
        );
    }

    let case2_safe_harbor = AutomationExpansionEvent {
        domain_id: DOMAIN_FOURPHONIX,
        asset_id: [0xBB; 16],
        is_cloud_provider_infrastructure: false,
        displacement_type: DisplacementType::DirectReplacement,
        taxonomy: default_taxonomy(),
        displacement_duration_days: 180,
        baseline_payroll_cents: 12_000_000,
        safe_harbor_reassignment_months: 36,
        safe_harbor_breached_clawback: false,
        consolidated_group_revenue_cents: consolidated_cents,
    };
    if let Ok(res2) = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&case2_safe_harbor) {
        print_case(
            "CASO 2: Reasignacion Laboral >= 36 Meses (EXENTO Sec. 5000G(a) - $0.00 USD)",
            &res2,
        );
    }

    let case3_taxable = AutomationExpansionEvent {
        domain_id: DOMAIN_AURATETRAX,
        asset_id: [0xCC; 16],
        is_cloud_provider_infrastructure: false,
        displacement_type: DisplacementType::DirectReplacement,
        taxonomy: default_taxonomy(),
        displacement_duration_days: 180,
        baseline_payroll_cents: 12_000_000,
        safe_harbor_reassignment_months: 0,
        safe_harbor_breached_clawback: false,
        consolidated_group_revenue_cents: consolidated_cents,
    };
    if let Ok(res3) = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&case3_taxable) {
        print_case(
            "CASO 3: Sustitucion Directa sin Safe Harbor (GRAVABLE Sec. 5000F)",
            &res3,
        );
    }

    let case4_clawback = AutomationExpansionEvent {
        domain_id: DOMAIN_STANDARD1979,
        asset_id: [0xDD; 16],
        is_cloud_provider_infrastructure: false,
        displacement_type: DisplacementType::DirectReplacement,
        taxonomy: default_taxonomy(),
        displacement_duration_days: 180,
        baseline_payroll_cents: 12_000_000,
        safe_harbor_reassignment_months: 18,
        safe_harbor_breached_clawback: true,
        consolidated_group_revenue_cents: consolidated_cents,
    };
    if let Ok(res4) = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&case4_clawback) {
        print_case(
            "CASO 4: Recaptura Clawback por Ruptura de Safe Harbor (Sec. 5000G(c))",
            &res4,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> AutomationExpansionEvent {
        AutomationExpansionEvent {
            domain_id: DOMAIN_AURATETRAX,
            asset_id: [9u8; 16],
            is_cloud_provider_infrastructure: false,
            displacement_type: DisplacementType::DirectReplacement,
            taxonomy: default_taxonomy(),
            displacement_duration_days: 120,
            baseline_payroll_cents: 12_000_000,
            safe_harbor_reassignment_months: 0,
            safe_harbor_breached_clawback: false,
            consolidated_group_revenue_cents: 850_000_000,
        }
    }

    #[test]
    fn test_01_clause_3_author_and_org_integrity() {
        assert_eq!(AAUFSA_AUTHOR, "Jose Emanuel Guadalupe Lopez");
        assert_eq!(AAUFSA_ORG_ID, 320_717_657_398);
        assert_eq!(VALID_DOMAINS.len(), 6);
    }

    #[test]
    fn test_02_domain_isolation_validation() {
        for &domain in &VALID_DOMAINS {
            assert_eq!(CapacityAndFiscalEngine::validate_domain(domain), Ok(()));
        }
        assert_eq!(
            CapacityAndFiscalEngine::validate_domain(999_999_999),
            Err(EngineError::InvalidDomain(999_999_999))
        );
    }

    #[test]
    fn test_03_irc_414_1563_controlled_group_aggregation() {
        let receipts = [
            DomainGrossReceipts {
                domain_id: DOMAIN_AURATETRAX,
                annual_gross_receipts_cents: 300_000_000,
            },
            DomainGrossReceipts {
                domain_id: DOMAIN_PROPHONIX,
                annual_gross_receipts_cents: 250_000_000,
            },
        ];
        let res = CapacityAndFiscalEngine::aggregate_controlled_group_receipts(&receipts);
        assert_eq!(res, Ok(550_000_000));
    }

    #[test]
    fn test_04_cloud_saas_developer_immunity_sec_5000hd() {
        let mut ev = sample_event();
        ev.is_cloud_provider_infrastructure = true;
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(!a.is_taxable);
            assert_eq!(a.total_tax_amount_cents, 0);
            assert_eq!(
                a.statutory_category,
                StatutoryCategory::ExemptSafeHarborOrImmunity
            );
        }
    }

    #[test]
    fn test_05_small_business_5m_threshold_sec_5000gb() {
        let mut ev = sample_event();
        ev.consolidated_group_revenue_cents = 500_000_000;
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(!a.is_taxable);
            assert_eq!(a.total_tax_amount_cents, 0);
        }
    }

    #[test]
    fn test_06_duration_90_days_threshold_sec_5000hc() {
        let mut ev = sample_event();
        ev.displacement_duration_days = 90;
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(!a.is_taxable);
            assert_eq!(a.total_tax_amount_cents, 0);
            assert_eq!(a.statutory_category, StatutoryCategory::NonSubjectSec5000Hc);
        }
    }

    #[test]
    fn test_07_safe_harbor_36_months_sec_5000ga() {
        let mut ev = sample_event();
        ev.safe_harbor_reassignment_months = 36;
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(!a.is_taxable);
            assert!(a.safe_harbor_applied);
            assert_eq!(a.total_tax_amount_cents, 0);
        }
    }

    #[test]
    fn test_08_exact_15_3_pct_tax_and_fiduciary_split() {
        let ev = sample_event();
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(a.is_taxable);
            assert_eq!(a.total_tax_amount_cents, 1_836_000);
            assert_eq!(a.social_security_allocation_cents, 1_377_000);
            assert_eq!(a.workforce_reskilling_allocation_cents, 459_000);
        }
    }

    #[test]
    fn test_09_clawback_recapture_surcharge_sec_5000gc() {
        let mut ev = sample_event();
        ev.safe_harbor_reassignment_months = 18;
        ev.safe_harbor_breached_clawback = true;
        let res = CapacityAndFiscalEngine::evaluate_aaufsa_expansion(&ev);
        assert!(res.is_ok());
        if let Ok(a) = res {
            assert!(a.is_taxable);
            assert!(a.clawback_triggered);
            assert_eq!(a.base_tax_cents, 1_836_000);
            assert_eq!(a.clawback_surcharge_cents, 36_720);
            assert_eq!(a.total_tax_amount_cents, 1_872_720);
            assert_eq!(a.social_security_allocation_cents, 1_404_540);
            assert_eq!(a.workforce_reskilling_allocation_cents, 468_180);
        }
    }

    #[test]
    fn test_10_vpp_der_saturation_and_frequency_guards() {
        let mut bus = VppBusState {
            bus_id: 1,
            current_mw: 90_000,
            max_mw: 100_000,
            current_ma: 100_000,
            max_ma: 500_000,
            grid_frequency_chz: 6000,
        };
        let der_over = DerNodeAddition {
            node_id: [1u8; 16],
            domain_id: DOMAIN_AURAINTEL_GLOBAL,
            injection_capacity_mw: 15_000,
            rated_current_ma: 10_000,
        };
        assert_eq!(
            CapacityAndFiscalEngine::evaluate_der_expansion(&mut bus, &der_over),
            Err(EngineError::VppOverCapacity {
                requested_mw: 105_000,
                max_mw: 100_000
            })
        );
    }

    #[test]
    fn test_11_live_telemetry_dispatch_and_curtailment() {
        let mut bus = VppBusState {
            bus_id: 1,
            current_mw: 80_000,
            max_mw: 100_000,
            current_ma: 100_000,
            max_ma: 500_000,
            grid_frequency_chz: 6000,
        };
        let packet = LiveTelemetryPacket {
            cycle_id: 1,
            previous_ledger_hash: "0".repeat(64),
            grid_frequency_chz: 6000,
            der_node: DerNodeAddition {
                node_id: [2u8; 16],
                domain_id: DOMAIN_AURAINTEL_GLOBAL,
                injection_capacity_mw: 10_000, // Lleva el bus a 90,000 mW (>= 85% de 100,000 mW)
                rated_current_ma: 20_000,
            },
            fiscal_event: sample_event(),
        };
        let receipt = CapacityAndFiscalEngine::process_live_telemetry_packet(&mut bus, &packet);
        assert!(receipt.is_ok());
        if let Ok(r) = receipt {
            assert_eq!(r.updated_bus_mw, 90_000);
            assert_eq!(
                r.dispatch_command,
                VppDispatchCommand::HighLoadCurtailmentWarning
            );
            assert_eq!(r.current_ledger_hash.len(), 64);
        }
    }

    #[test]
    fn test_12_sql_ledger_generation_and_hash_chain() {
        let mut bus = VppBusState {
            bus_id: 1,
            current_mw: 40_000,
            max_mw: 100_000,
            current_ma: 100_000,
            max_ma: 500_000,
            grid_frequency_chz: 5980, // Activa FrequencyDroopCorrection
        };
        let packet = LiveTelemetryPacket {
            cycle_id: 42,
            previous_ledger_hash: "a".repeat(64),
            grid_frequency_chz: 5980,
            der_node: DerNodeAddition {
                node_id: [3u8; 16],
                domain_id: DOMAIN_AURATETRAX,
                injection_capacity_mw: 5_000,
                rated_current_ma: 10_000,
            },
            fiscal_event: sample_event(),
        };
        let receipt = CapacityAndFiscalEngine::process_live_telemetry_packet(&mut bus, &packet);
        assert!(receipt.is_ok());
        if let Ok(r) = receipt {
            assert_eq!(
                r.dispatch_command,
                VppDispatchCommand::FrequencyDroopCorrection
            );
            assert!(r
                .sql_transaction
                .contains("INSERT INTO aaufsa_vpp_operational_ledger"));
            assert!(r.sql_transaction.contains("1836000"));
        }
    }
}
