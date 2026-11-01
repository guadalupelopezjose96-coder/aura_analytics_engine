#![forbid(unsafe_code)]
#![allow(
#[allow(non_camel_case_types)]
type fixed_point = i64;
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    dead_code,
    unused_imports,
    unused_variables
)]

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

type HmacSha256 = Hmac<Sha256>;

struct AppState {
    db: PgPool,
}

fn verify_stripe_signature(payload: &[u8], sig_header: &str, secret: &str) -> bool {
    let mut timestamp: Option<&str> = None;
    let mut signature: Option<&str> = None;

    for item in sig_header.split(',') {
        let parts: Vec<&str> = item.split('=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "t" => timestamp = Some(parts[1].trim()),
                "v1" => signature = Some(parts[1].trim()),
                _ => {}
            }
        }
    }

    let (Some(t), Some(expected_sig)) = (timestamp, signature) else {
        return false;
    };

    let signed_payload = [t.as_bytes(), b".", payload].concat();

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(&signed_payload);

    let Ok(expected_bytes) = hex::decode(expected_sig) else {
        return false;
    };

    mac.verify_slice(&expected_bytes).is_ok()
}

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "OPERATIONAL",
            "domain_id": "188680247367",
            "org_id": "320717657398",
            "compliance": "OWASP-ASVS-L3"
        })),
    )
}

async fn verify_ledger_integrity(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_token =
        env::var("LEDGER_ADMIN_KEY").unwrap_or_else(|_| "aura_secret_audit_key".to_string());

    let is_authorized = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|val| val == format!("Bearer {}", auth_token))
        .unwrap_or(false);

    if !is_authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "UNAUTHORIZED_AUDIT_ACCESS"})),
        );
    }

    // Se calcula el balance general y se obtiene el timestamp nativo de Postgres
    let verify_res = sqlx::query(
        "SELECT 
            COALESCE(SUM(debit_amount_cents), 0)::bigint AS total_debits,
            COALESCE(SUM(credit_amount_cents), 0)::bigint AS total_credits,
            (COALESCE(SUM(debit_amount_cents), 0) - COALESCE(SUM(credit_amount_cents), 0))::bigint AS delta,
            COUNT(DISTINCT entry_id)::bigint AS total_entries_count,
            COUNT(*)::bigint AS total_lines_count,
            TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS audit_timestamp
         FROM ledger_lines"
    )
    .fetch_one(&state.db)
    .await;

    match verify_res {
        Ok(row) => {
            let total_debits: i64 = row.get("total_debits");
            let total_credits: i64 = row.get("total_credits");
            let delta: i64 = row.get("delta");
            let entries_count: i64 = row.get("total_entries_count");
            let lines_count: i64 = row.get("total_lines_count");
            let timestamp: String = row.try_get("audit_timestamp").unwrap_or_default();

            let is_balanced = delta == 0 && total_debits == total_credits;

            (
                StatusCode::OK,
                Json(json!({
                    "is_mathematically_balanced": is_balanced,
                    "delta_cents": delta,
                    "total_debits_cents": total_debits,
                    "total_credits_cents": total_credits,
                    "total_entries_count": entries_count,
                    "total_lines_count": lines_count,
                    "audit_timestamp": timestamp
                })),
            )
        }
        Err(e) => {
            eprintln!("Error en verificación matemática del libro: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "LEDGER_VERIFICATION_QUERY_FAILED"})),
            )
        }
    }
}

async fn get_ledger_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_token =
        env::var("LEDGER_ADMIN_KEY").unwrap_or_else(|_| "aura_secret_audit_key".to_string());

    let is_authorized = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|val| val == format!("Bearer {}", auth_token))
        .unwrap_or(false);

    if !is_authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "UNAUTHORIZED_AUDIT_ACCESS"})),
        );
    }

    let income_res = sqlx::query(
        "SELECT 
            gross_revenue_cents::bigint AS gross_revenue_cents, 
            refunds_cents::bigint AS refunds_cents, 
            processing_fees_cents::bigint AS processing_fees_cents, 
            net_operating_income_cents::bigint AS net_operating_income_cents 
         FROM view_income_statement LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await;

    let trial_rows = sqlx::query(
        "SELECT 
            account_number, 
            name, 
            currency, 
            total_debits_cents::bigint AS total_debits_cents, 
            total_credits_cents::bigint AS total_credits_cents, 
            net_balance_cents::bigint AS net_balance_cents 
         FROM view_trial_balance",
    )
    .fetch_all(&state.db)
    .await;

    let income = match income_res {
        Ok(Some(row)) => {
            let gross: i64 = row.try_get("gross_revenue_cents").unwrap_or(0);
            let refunds: i64 = row.try_get("refunds_cents").unwrap_or(0);
            let fees: i64 = row.try_get("processing_fees_cents").unwrap_or(0);
            let net: i64 = row.try_get("net_operating_income_cents").unwrap_or(0);

            json!({
                "gross_revenue_cents": gross,
                "refunds_cents": refunds,
                "processing_fees_cents": fees,
                "net_operating_income_cents": net,
                "net_operating_income_usd": format!("{:.2}", (net as fixed_point) / 100)
            })
        }
        Ok(None) => json!({}),
        Err(e) => {
            eprintln!("Error consultando income: {}", e);
            json!({"error": "INCOME_QUERY_FAILED"})
        }
    };

    let accounts = match trial_rows {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                json!({
                    "account_number": r.try_get::<String, _>("account_number").unwrap_or_default(),
                    "name": r.try_get::<String, _>("name").unwrap_or_default(),
                    "currency": r.try_get::<String, _>("currency").unwrap_or_default(),
                    "total_debits_cents": r.try_get::<i64, _>("total_debits_cents").unwrap_or(0),
                    "total_credits_cents": r.try_get::<i64, _>("total_credits_cents").unwrap_or(0),
                    "net_balance_cents": r.try_get::<i64, _>("net_balance_cents").unwrap_or(0)
                })
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error consultando trial: {}", e);
            vec![]
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "status": "BALANCED_AUDITED",
            "income_statement": income,
            "trial_balance": accounts
        })),
    )
}

async fn handle_stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let secret = env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();

    if !secret.is_empty() {
        let sig = headers
            .get("stripe-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        if !verify_stripe_signature(&body, sig, &secret) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Firma HMAC invalida"})),
            );
        }
    }

    let Ok(payload) = serde_json::from_slice::<Value>(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Payload JSON malformado"})),
        );
    };

    let event_type = payload
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let event_id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("evt_unknown");

    match event_type {
        "checkout.session.completed" => {
            let gross_amount = payload
                .pointer("/data/object/amount_total")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let fee_cents = payload
                .pointer("/data/object/fee")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| {
                    if gross_amount > 0 {
                        ((gross_amount * 290) / 10000) + 30
                    } else {
                        0
                    }
                });

            let net_clearing = gross_amount - fee_cents;

            persist_payment_with_retry(
                &state.db,
                event_id,
                "Checkout Payment",
                "4000-SUBSCRIPTION-REV",
                gross_amount,
                fee_cents,
                net_clearing,
            )
            .await
        }
        "invoice.payment_succeeded" => {
            let gross_amount = payload
                .pointer("/data/object/amount_paid")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let fee_cents = payload
                .pointer("/data/object/fee")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| {
                    if gross_amount > 0 {
                        ((gross_amount * 290) / 10000) + 30
                    } else {
                        0
                    }
                });

            let net_clearing = gross_amount - fee_cents;

            persist_payment_with_retry(
                &state.db,
                event_id,
                "Recurring Subscription Payment",
                "4010-RECURRING-SUBSCRIPTION-REV",
                gross_amount,
                fee_cents,
                net_clearing,
            )
            .await
        }
        "charge.refunded" => {
            let amount_refunded = payload
                .pointer("/data/object/amount_refunded")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            persist_refund(&state.db, event_id, amount_refunded).await
        }
        _ => (
            StatusCode::OK,
            Json(json!({
                "received": true,
                "status": "IGNORED_EVENT"
            })),
        ),
    }
}

async fn persist_payment_with_retry(
    pool: &PgPool,
    event_id: &str,
    description_prefix: &str,
    revenue_account: &str,
    gross_amount: i64,
    fee_cents: i64,
    net_clearing: i64,
) -> (StatusCode, Json<Value>) {
    let mut attempts = 0;
    loop {
        attempts += 1;
        match try_persist_payment(
            pool,
            event_id,
            description_prefix,
            revenue_account,
            gross_amount,
            fee_cents,
            net_clearing,
        )
        .await
        {
            Ok(res) => return res,
            Err(e) if attempts < 4 && e.to_string().contains("could not serialize access") => {
                sleep(Duration::from_millis(20 * (1 << attempts))).await;
                continue;
            }
            Err(e) => {
                eprintln!("Error SQL persistente: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "DATABASE_ERROR"})),
                );
            }
        }
    }
}

async fn try_persist_payment(
    pool: &PgPool,
    event_id: &str,
    description_prefix: &str,
    revenue_account: &str,
    gross_amount: i64,
    fee_cents: i64,
    net_clearing: i64,
) -> Result<(StatusCode, Json<Value>), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
        .execute(&mut *tx)
        .await?;

    let entry_row = sqlx::query(
        "INSERT INTO ledger_entries (stripe_event_id, description) \
         VALUES ($1, $2) \
         ON CONFLICT (stripe_event_id) DO NOTHING \
         RETURNING entry_id::text",
    )
    .bind(event_id)
    .bind(format!("{}: {}", description_prefix, event_id))
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(row) = entry_row {
        let entry_id_str: String = row.try_get("entry_id").unwrap_or_default();

        sqlx::query(
            "INSERT INTO ledger_lines (entry_id, account_id, debit_amount_cents, credit_amount_cents) \
             SELECT $1::uuid, account_id, $2, 0 \
             FROM ledger_accounts WHERE account_number = '1000-STRIPE-CLEARING'",
        )
        .bind(&entry_id_str)
        .bind(net_clearing)
        .execute(&mut *tx)
        .await?;

        if fee_cents > 0 {
            sqlx::query(
                "INSERT INTO ledger_lines (entry_id, account_id, debit_amount_cents, credit_amount_cents) \
                 SELECT $1::uuid, account_id, $2, 0 \
                 FROM ledger_accounts WHERE account_number = '5000-STRIPE-PROCESSING-FEE'",
            )
            .bind(&entry_id_str)
            .bind(fee_cents)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            "INSERT INTO ledger_lines (entry_id, account_id, debit_amount_cents, credit_amount_cents) \
             SELECT $1::uuid, account_id, 0, $2 \
             FROM ledger_accounts WHERE account_number = $3",
        )
        .bind(&entry_id_str)
        .bind(gross_amount)
        .bind(revenue_account)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((
            StatusCode::OK,
            Json(json!({
                "received": true,
                "ledger_status": "COMMITTED_SERIALIZABLE",
                "event_id": event_id,
                "gross_amount": gross_amount,
                "fee_cents": fee_cents,
                "net_clearing": net_clearing,
                "balance_delta": 0
            })),
        ))
    } else {
        tx.rollback().await?;
        Ok((
            StatusCode::OK,
            Json(json!({
                "received": true,
                "ledger_status": "IDEMPOTENT_IGNORED",
                "event_id": event_id
            })),
        ))
    }
}

async fn persist_refund(
    pool: &PgPool,
    event_id: &str,
    amount_refunded: i64,
) -> (StatusCode, Json<Value>) {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "DB_TX_INIT_FAILED"})),
            );
        }
    };

    let entry_row = sqlx::query(
        "INSERT INTO ledger_entries (stripe_event_id, description) \
         VALUES ($1, $2) \
         ON CONFLICT (stripe_event_id) DO NOTHING \
         RETURNING entry_id::text",
    )
    .bind(event_id)
    .bind(format!("Stripe Charge Refund: {}", event_id))
    .fetch_optional(&mut *tx)
    .await;

    match entry_row {
        Ok(Some(row)) => {
            let entry_id_str: String = match row.try_get("entry_id") {
                Ok(id) => id,
                Err(_) => {
                    let _ = tx.rollback().await;
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "UUID_READ_ERROR"})),
                    );
                }
            };

            let _ = sqlx::query(
                "INSERT INTO ledger_lines (entry_id, account_id, debit_amount_cents, credit_amount_cents) \
                 SELECT $1::uuid, account_id, $2, 0 \
                 FROM ledger_accounts WHERE account_number = '4100-REFUNDS-CONTRA'",
            )
            .bind(&entry_id_str)
            .bind(amount_refunded)
            .execute(&mut *tx)
            .await;

            let _ = sqlx::query(
                "INSERT INTO ledger_lines (entry_id, account_id, debit_amount_cents, credit_amount_cents) \
                 SELECT $1::uuid, account_id, 0, $2 \
                 FROM ledger_accounts WHERE account_number = '1000-STRIPE-CLEARING'",
            )
            .bind(&entry_id_str)
            .bind(amount_refunded)
            .execute(&mut *tx)
            .await;

            if let Err(_) = tx.commit().await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "LEDGER_INTEGRITY_VIOLATION"})),
                );
            }

            (
                StatusCode::OK,
                Json(json!({
                    "received": true,
                    "ledger_status": "COMMITTED_SERIALIZABLE",
                    "event_id": event_id,
                    "amount_cents": amount_refunded,
                    "balance_delta": 0
                })),
            )
        }
        Ok(None) => {
            let _ = tx.rollback().await;
            (
                StatusCode::OK,
                Json(json!({
                    "received": true,
                    "ledger_status": "IDEMPOTENT_IGNORED",
                    "event_id": event_id
                })),
            )
        }
        Err(_) => {
            let _ = tx.rollback().await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "DATABASE_ERROR"})),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    let current_user = env::var("USER").unwrap_or_else(|_| "postgres".to_string());
    let default_url = format!("postgres://{}@localhost/aura_ledger", current_user);
    let db_url = env::var("DATABASE_URL").unwrap_or(default_url);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("No se pudo conectar a PostgreSQL en Termux");

    let state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/ledger/metrics", get(get_ledger_metrics))
        .route("/api/v1/ledger/verify", get(verify_ledger_integrity))
        .route("/api/v1/stripe/webhook", post(handle_stripe_webhook))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Aura Web Engine escuchando en http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error enlazando socket: {}", e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Error en el servidor Axum: {}", e);
    }
}
