#!/data/data/com.termux/files/usr/bin/bash
LEDGER_DB="/data/data/com.termux/files/home/aura_analytics_engine/saas_billing_portal/ledger.db"

while true; do
  LAST_ROW=$(sqlite3 "$LEDGER_DB" "SELECT cycle_id, record_hash FROM aaufsa_vpp_operational_ledger ORDER BY cycle_id DESC LIMIT 1;")
  CYCLE_ID=$(echo "$LAST_ROW" | cut -d'|' -f1)
  PREV_HASH=$(echo "$LAST_ROW" | cut -d'|' -f2)

  NEXT_CYCLE=$((CYCLE_ID + 1))

  # Modular corriente: si se superan los ciclos de inyección, estabilizar en 0 mA
  CURRENT_MA=0
  INJECTION_MW=0

  PAYLOAD=$(cat <<JSON
{
  "cycle_id": $NEXT_CYCLE,
  "previous_ledger_hash": "$PREV_HASH",
  "grid_frequency_chz": 6000,
  "der_node": {
    "node_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    "domain_id": 188680247367,
    "injection_capacity_mw": $INJECTION_MW,
    "rated_current_ma": $CURRENT_MA
  },
  "fiscal_event": {
    "domain_id": 188680247367,
    "asset_id": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    "is_cloud_provider_infrastructure": true,
    "displacement_type": "DirectReplacement",
    "taxonomy": {
      "d01_cognitive_automation": true,
      "d02_robotic_actuation": false,
      "d03_predictive_scheduling": false,
      "d04_autonomous_logistics": false,
      "d05_synthetic_customer_agent": false,
      "d06_algorithmic_underwriting": false,
      "d07_automated_code_synthesis": false,
      "d08_vpp_grid_autonomy": false,
      "d09_zero_touch_provisioning": false,
      "d10_continuous_compliance_audit": false,
      "d11_automated_treasury_routing": false,
      "d12_neural_quality_control": false,
      "d13_autonomous_cyber_defense": false,
      "d14_generative_design_execution": false
    },
    "displacement_duration_days": 30,
    "baseline_payroll_cents": 5000000,
    "safe_harbor_reassignment_months": 6,
    "safe_harbor_breached_clawback": false,
    "consolidated_group_revenue_cents": 100000000
  }
}
JSON
)

  RESP=$(curl -s -w "\n%{http_code}" -X POST http://127.0.0.1:3000/api/v1/dispatch \
    -H "Content-Type: application/json" \
    -H "X-API-Key: aura_live_cliente_premium" \
    -d "$PAYLOAD")

  HTTP_CODE=$(echo "$RESP" | tail -n 1)

  if [ "$HTTP_CODE" -eq 200 ]; then
    echo "[$(date '+%T')] Despachado ciclo $NEXT_CYCLE (OK 200)"
  else
    echo "[$(date '+%T')] Error despachando ciclo $NEXT_CYCLE (HTTP $HTTP_CODE)"
  fi

  sleep 5
done
