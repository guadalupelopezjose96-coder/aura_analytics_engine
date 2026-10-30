#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const FIXED_SCALE: i128 = 1_000_000;
pub const AAUFSA_TAX_RATE_PPM: i128 = 153_000;
pub const CONTROLLED_GROUP_THRESHOLD_MICRO: i128 = 5_000_000 * FIXED_SCALE;
pub const SOCIAL_SECURITY_SPLIT_PPM: i128 = 750_000;
pub const WORKFORCE_RESKILLING_SPLIT_PPM: i128 = 250_000;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum AnalyticsDomain {
    AuraTetrax = 877771073297,
    Fourphonix = 964521086569,
    AuraIntelCore = 188680247367,
    AuraIntelGlobal = 1087754464927,
    Standard1979 = 282230018778,
    Prophonix = 358233233,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VppTelemetrySample {
    pub timestamp_utc_ms: i64,
    pub frequency_micro_hz: i128,
    pub active_power_micro_kw: i128,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VppStatisticalSummary {
    pub mean_power_micro_kw: i128,
    pub variance_power_micro: i128,
    pub p99_power_micro_kw: i128,
    pub samples_analyzed: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LaborDisplacementRecord {
    pub employee_id_hash: [u8; 32],
    pub displaced_days: u32,
    pub bls_median_annual_salary_micro: i128,
    pub reassignment_months_active: u32,
    pub reassignment_breached: bool,
    pub is_cloud_saas_exempt: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AaufsaFiscalAssessment {
    pub domain_id: u64,
    pub gross_revenue_controlled_group_micro: i128,
    pub threshold_met: bool,
    pub net_liability_micro: i128,
    pub clawback_triggered: bool,
    pub allocation_social_security_micro: i128,
    pub allocation_reskilling_micro: i128,
    pub audit_hash_sha256: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    EmptyDataset,
    ArithmeticOverflow,
    InvalidScale,
}

pub fn analyze_vpp_telemetry(
    samples: &mut [VppTelemetrySample],
) -> Result<VppStatisticalSummary, EngineError> {
    let n = samples.len();
    if n == 0 {
        return Err(EngineError::EmptyDataset);
    }
    let n_i128 = i128::try_from(n).map_err(|_| EngineError::ArithmeticOverflow)?;

    let mut sum_power: i128 = 0;
    for s in samples.iter() {
        sum_power = sum_power
            .checked_add(s.active_power_micro_kw)
            .ok_or(EngineError::ArithmeticOverflow)?;
    }
    let mean_power = sum_power
        .checked_div(n_i128)
        .ok_or(EngineError::ArithmeticOverflow)?;

    let mut sum_sq_diff: i128 = 0;
    for s in samples.iter() {
        let diff = s
            .active_power_micro_kw
            .checked_sub(mean_power)
            .ok_or(EngineError::ArithmeticOverflow)?;
        let scaled_diff = diff
            .checked_mul(diff)
            .ok_or(EngineError::ArithmeticOverflow)?
            .checked_div(FIXED_SCALE)
            .ok_or(EngineError::ArithmeticOverflow)?;
        sum_sq_diff = sum_sq_diff
            .checked_add(scaled_diff)
            .ok_or(EngineError::ArithmeticOverflow)?;
    }
    let variance_power = sum_sq_diff
        .checked_div(n_i128)
        .ok_or(EngineError::ArithmeticOverflow)?;

    samples.sort_by_key(|s| s.active_power_micro_kw);
    let rank_index = (n * 99) / 100;
    let p99_power = samples[if rank_index >= n { n - 1 } else { rank_index }].active_power_micro_kw;

    Ok(VppStatisticalSummary {
        mean_power_micro_kw: mean_power,
        variance_power_micro: variance_power,
        p99_power_micro_kw: p99_power,
        samples_analyzed: n,
    })
}

pub fn assess_aaufsa_compliance(
    domain: AnalyticsDomain,
    controlled_group_revenue_micro: i128,
    records: &[LaborDisplacementRecord],
) -> Result<AaufsaFiscalAssessment, EngineError> {
    let domain_id = domain as u64;
    let threshold_met = controlled_group_revenue_micro >= CONTROLLED_GROUP_THRESHOLD_MICRO;

    let mut total_liability_micro: i128 = 0;
    let mut clawback_active = false;

    if threshold_met {
        for record in records {
            if record.is_cloud_saas_exempt {
                continue;
            }

            if record.displaced_days > 90 {
                let safe_harbor_valid =
                    record.reassignment_months_active >= 36 && !record.reassignment_breached;

                if !safe_harbor_valid {
                    let tax_due = record
                        .bls_median_annual_salary_micro
                        .checked_mul(AAUFSA_TAX_RATE_PPM)
                        .ok_or(EngineError::ArithmeticOverflow)?
                        .checked_div(FIXED_SCALE)
                        .ok_or(EngineError::ArithmeticOverflow)?;

                    total_liability_micro = total_liability_micro
                        .checked_add(tax_due)
                        .ok_or(EngineError::ArithmeticOverflow)?;

                    if record.reassignment_breached && record.reassignment_months_active < 36 {
                        clawback_active = true;
                    }
                }
            }
        }
    }

    let allocation_ss = total_liability_micro
        .checked_mul(SOCIAL_SECURITY_SPLIT_PPM)
        .ok_or(EngineError::ArithmeticOverflow)?
        .checked_div(FIXED_SCALE)
        .ok_or(EngineError::ArithmeticOverflow)?;

    let allocation_reskilling = total_liability_micro
        .checked_sub(allocation_ss)
        .ok_or(EngineError::ArithmeticOverflow)?;

    let mut hasher = Sha256::new();
    hasher.update(domain_id.to_be_bytes());
    hasher.update(controlled_group_revenue_micro.to_be_bytes());
    hasher.update(total_liability_micro.to_be_bytes());
    hasher.update(if clawback_active { [1u8] } else { [0u8] });
    let audit_hash_sha256 = hex::encode(hasher.finalize());

    Ok(AaufsaFiscalAssessment {
        domain_id,
        gross_revenue_controlled_group_micro: controlled_group_revenue_micro,
        threshold_met,
        net_liability_micro: total_liability_micro,
        clawback_triggered: clawback_active,
        allocation_social_security_micro: allocation_ss,
        allocation_reskilling_micro: allocation_reskilling,
        audit_hash_sha256,
    })
}
