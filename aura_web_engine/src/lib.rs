#![forbid(unsafe_code)]
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::panic,
    clippy::float_arithmetic
)]

pub mod static_handler;
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub const FIXED_SCALE: i128 = 1_000_000;
pub const STATUTORY_RATE_NUMERATOR: i128 = 153_000;
pub const RATE_DENOMINATOR: i128 = 1_000_000;
pub const SMALL_BIZ_THRESHOLD: i128 = 5_000_000 * FIXED_SCALE;
pub const MIN_SAFE_HARBOR_MONTHS: u32 = 36;
pub const FIDUCIARY_SPLIT_OASI_NUMERATOR: i128 = 75;
pub const FIDUCIARY_SPLIT_RESKILL_NUMERATOR: i128 = 25;
pub const FIDUCIARY_SPLIT_DENOMINATOR: i128 = 100;

pub const CORPORATE_DOMAINS: [u64; 6] = [
    877_771_073_297,   // AuraTetrax-com
    964_521_086_569,   // Fourphonix-com
    188_680_247_367,   // AuraIntelCore-com
    1_087_754_464_927, // AuraIntel-Global-com
    282_230_018_778,   // Standard1979-com
    358_233_233,       // Prophonix-com
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceDeliveryModel {
    ThirdPartyCloudSaas,
    InternalAutomation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub transaction_id: String,
    pub domain_id: u64,
    pub delivery_model: ServiceDeliveryModel,
    pub base_amount_micro_usd: i128,
    pub indirect_tax_micro_usd: i128,
    pub consolidated_gross_revenue_micro_usd: i128,
    pub human_fte_displaced: bool,
    pub days_displaced: u32,
    pub safe_harbor_months_honored: u32,
    pub unexcused_dismissal_occurred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiduciaryBifurcation {
    pub oasi_trust_fund_micro_usd: i128,
    pub reskilling_trust_fund_micro_usd: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatutorySettlementResult {
    pub gross_total_micro_usd: i128,
    pub indirect_tax_micro_usd: i128,
    pub taxable_automation_base_micro_usd: i128,
    pub aaufsa_tax_micro_usd: i128,
    pub clawback_surcharge_micro_usd: i128,
    pub is_cloud_exempt_sec_5000h_d: bool,
    pub is_small_business_exempt_sec_5000g_b: bool,
    pub is_safe_harbor_active_sec_5000g_a: bool,
    pub fiduciary_split: FiduciaryBifurcation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEdgeError {
    pub code: u16,
    pub message: String,
}

pub fn is_valid_domain(domain_id: u64) -> bool {
    CORPORATE_DOMAINS.contains(&domain_id)
}

pub fn calculate_statutory_settlement(
    req: &TransactionRequest,
) -> Result<StatutorySettlementResult, WebEdgeError> {
    if !is_valid_domain(req.domain_id) {
        return Err(WebEdgeError {
            code: 400,
            message: "Dominio corporativo no autorizado o invalido".to_string(),
        });
    }

    if req.base_amount_micro_usd < 0 || req.indirect_tax_micro_usd < 0 {
        return Err(WebEdgeError {
            code: 400,
            message: "Montos transaccionales no pueden ser negativos".to_string(),
        });
    }

    let gross_total_micro_usd = req
        .base_amount_micro_usd
        .checked_add(req.indirect_tax_micro_usd)
        .ok_or_else(|| WebEdgeError {
            code: 500,
            message: "Desbordamiento aritmético en calculo total".to_string(),
        })?;

    if req.delivery_model == ServiceDeliveryModel::ThirdPartyCloudSaas {
        return Ok(StatutorySettlementResult {
            gross_total_micro_usd,
            indirect_tax_micro_usd: req.indirect_tax_micro_usd,
            taxable_automation_base_micro_usd: 0,
            aaufsa_tax_micro_usd: 0,
            clawback_surcharge_micro_usd: 0,
            is_cloud_exempt_sec_5000h_d: true,
            is_small_business_exempt_sec_5000g_b: false,
            is_safe_harbor_active_sec_5000g_a: false,
            fiduciary_split: FiduciaryBifurcation {
                oasi_trust_fund_micro_usd: 0,
                reskilling_trust_fund_micro_usd: 0,
            },
        });
    }

    let is_small_business = req.consolidated_gross_revenue_micro_usd <= SMALL_BIZ_THRESHOLD;
    if is_small_business {
        return Ok(StatutorySettlementResult {
            gross_total_micro_usd,
            indirect_tax_micro_usd: req.indirect_tax_micro_usd,
            taxable_automation_base_micro_usd: 0,
            aaufsa_tax_micro_usd: 0,
            clawback_surcharge_micro_usd: 0,
            is_cloud_exempt_sec_5000h_d: false,
            is_small_business_exempt_sec_5000g_b: true,
            is_safe_harbor_active_sec_5000g_a: false,
            fiduciary_split: FiduciaryBifurcation {
                oasi_trust_fund_micro_usd: 0,
                reskilling_trust_fund_micro_usd: 0,
            },
        });
    }

    if !req.human_fte_displaced || req.days_displaced <= 90 {
        return Ok(StatutorySettlementResult {
            gross_total_micro_usd,
            indirect_tax_micro_usd: req.indirect_tax_micro_usd,
            taxable_automation_base_micro_usd: 0,
            aaufsa_tax_micro_usd: 0,
            clawback_surcharge_micro_usd: 0,
            is_cloud_exempt_sec_5000h_d: false,
            is_small_business_exempt_sec_5000g_b: false,
            is_safe_harbor_active_sec_5000g_a: false,
            fiduciary_split: FiduciaryBifurcation {
                oasi_trust_fund_micro_usd: 0,
                reskilling_trust_fund_micro_usd: 0,
            },
        });
    }

    let safe_harbor_active = req.safe_harbor_months_honored >= MIN_SAFE_HARBOR_MONTHS
        && !req.unexcused_dismissal_occurred;

    if safe_harbor_active {
        return Ok(StatutorySettlementResult {
            gross_total_micro_usd,
            indirect_tax_micro_usd: req.indirect_tax_micro_usd,
            taxable_automation_base_micro_usd: 0,
            aaufsa_tax_micro_usd: 0,
            clawback_surcharge_micro_usd: 0,
            is_cloud_exempt_sec_5000h_d: false,
            is_small_business_exempt_sec_5000g_b: false,
            is_safe_harbor_active_sec_5000g_a: true,
            fiduciary_split: FiduciaryBifurcation {
                oasi_trust_fund_micro_usd: 0,
                reskilling_trust_fund_micro_usd: 0,
            },
        });
    }

    let taxable_base = req.base_amount_micro_usd;
    let raw_tax = taxable_base
        .checked_mul(STATUTORY_RATE_NUMERATOR)
        .ok_or_else(|| WebEdgeError {
            code: 500,
            message: "Desbordamiento en calculo de tasa fiscal".to_string(),
        })?;
    let statutory_tax = raw_tax / RATE_DENOMINATOR;

    let clawback_surcharge = if req.unexcused_dismissal_occurred {
        let raw_clawback = taxable_base
            .checked_mul(20_000)
            .ok_or_else(|| WebEdgeError {
                code: 500,
                message: "Desbordamiento en calculo de clawback".to_string(),
            })?;
        raw_clawback / RATE_DENOMINATOR
    } else {
        0
    };

    let total_tax_assessed = statutory_tax
        .checked_add(clawback_surcharge)
        .ok_or_else(|| WebEdgeError {
            code: 500,
            message: "Desbordamiento en sumatoria fiscal".to_string(),
        })?;

    let oasi_amount = (total_tax_assessed
        .checked_mul(FIDUCIARY_SPLIT_OASI_NUMERATOR)
        .ok_or_else(|| WebEdgeError {
            code: 500,
            message: "Desbordamiento en split OASI".to_string(),
        })?)
        / FIDUCIARY_SPLIT_DENOMINATOR;

    let reskilling_amount =
        total_tax_assessed
            .checked_sub(oasi_amount)
            .ok_or_else(|| WebEdgeError {
                code: 500,
                message: "Error en balance de bifurcación fiduciaria".to_string(),
            })?;

    Ok(StatutorySettlementResult {
        gross_total_micro_usd,
        indirect_tax_micro_usd: req.indirect_tax_micro_usd,
        taxable_automation_base_micro_usd: taxable_base,
        aaufsa_tax_micro_usd: statutory_tax,
        clawback_surcharge_micro_usd: clawback_surcharge,
        is_cloud_exempt_sec_5000h_d: false,
        is_small_business_exempt_sec_5000g_b: false,
        is_safe_harbor_active_sec_5000g_a: false,
        fiduciary_split: FiduciaryBifurcation {
            oasi_trust_fund_micro_usd: oasi_amount,
            reskilling_trust_fund_micro_usd: reskilling_amount,
        },
    })
}

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    let mut nonce_bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let csp_nonce = BASE64.encode(nonce_bytes);

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    let csp_policy = format!(
        "default-src 'self'; script-src 'self' 'nonce-{csp_nonce}'; style-src 'self' 'nonce-{csp_nonce}'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; connect-src 'self' https://api.stripe.com; require-trusted-types-for 'script';",
    );

    if let Ok(val) = HeaderValue::from_str(&csp_policy) {
        headers.insert(header::CONTENT_SECURITY_POLICY, val);
    }
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63_072_000; includeSubDomains; preload"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(self), usb=()"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-embedder-policy"),
        HeaderValue::from_static("require-corp"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );

    response
}

async fn settle_transaction_handler(
    headers: HeaderMap,
    Json(payload): Json<TransactionRequest>,
) -> impl IntoResponse {
    let idempotency_key = headers.get("Idempotency-Key").and_then(|v| v.to_str().ok());

    if idempotency_key.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Falta cabecera Idempotency-Key obligatoria"
            })),
        )
            .into_response();
    }

    match calculate_statutory_settlement(&payload) {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::to_value(result).unwrap_or_default()),
        )
            .into_response(),
        Err(err) => (
            StatusCode::from_u16(err.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(serde_json::json!({ "error": err.message })),
        )
            .into_response(),
    }
}

pub fn create_secure_app() -> Router {
    Router::new()
        .route(
            "/api/v1/transaction/settle",
            post(settle_transaction_handler),
        )
        .layer(middleware::from_fn(security_headers_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_saas_immunity_sec_5000h_d() {
        let req = TransactionRequest {
            transaction_id: "TX-TEST-001".to_string(),
            domain_id: 188_680_247_367,
            delivery_model: ServiceDeliveryModel::ThirdPartyCloudSaas,
            base_amount_micro_usd: 100_000_000,
            indirect_tax_micro_usd: 16_000_000,
            consolidated_gross_revenue_micro_usd: 10_000_000 * FIXED_SCALE,
            human_fte_displaced: true,
            days_displaced: 120,
            safe_harbor_months_honored: 0,
            unexcused_dismissal_occurred: false,
        };

        let res = calculate_statutory_settlement(&req).expect("Calculo exitoso");
        assert!(res.is_cloud_exempt_sec_5000h_d);
        assert_eq!(res.aaufsa_tax_micro_usd, 0);
        assert_eq!(res.fiduciary_split.oasi_trust_fund_micro_usd, 0);
        assert_eq!(res.fiduciary_split.reskilling_trust_fund_micro_usd, 0);
    }

    #[test]
    fn test_internal_displacement_tax_and_split() {
        let req = TransactionRequest {
            transaction_id: "TX-TEST-002".to_string(),
            domain_id: 877_771_073_297,
            delivery_model: ServiceDeliveryModel::InternalAutomation,
            base_amount_micro_usd: 1_000_000_000,
            indirect_tax_micro_usd: 0,
            consolidated_gross_revenue_micro_usd: 10_000_000 * FIXED_SCALE,
            human_fte_displaced: true,
            days_displaced: 91,
            safe_harbor_months_honored: 12,
            unexcused_dismissal_occurred: false,
        };

        let res = calculate_statutory_settlement(&req).expect("Calculo exitoso");
        assert!(!res.is_cloud_exempt_sec_5000h_d);
        assert_eq!(res.aaufsa_tax_micro_usd, 153_000_000);
        assert_eq!(res.fiduciary_split.oasi_trust_fund_micro_usd, 114_750_000);
        assert_eq!(
            res.fiduciary_split.reskilling_trust_fund_micro_usd,
            38_250_000
        );
        assert_eq!(
            res.fiduciary_split.oasi_trust_fund_micro_usd
                + res.fiduciary_split.reskilling_trust_fund_micro_usd,
            res.aaufsa_tax_micro_usd
        );
    }
}
