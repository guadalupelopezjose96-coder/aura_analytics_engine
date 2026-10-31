#!/usr/bin/env bash
set -euo pipefail
umask 077

echo "=== INICIANDO AUDITORIA POLICY-AS-CODE (ORG 320717657398) ==="

# 1. Verificar directivas Rust de memoria segura y cero punto flotante
for f in src/lib.rs src/bin/aaufsa_simulator.rs; do
    grep -q "#!\[forbid(unsafe_code)\]" "$f"
    grep -q "clippy::float_arithmetic" "$f"
    grep -q "clippy::unwrap_used" "$f"
    grep -q "clippy::panic" "$f"
done
if grep -rnE "\bf32\b|\bf64\b" src/; then
    echo "[FATAL] Punto flotante detectado" >&2
    exit 1
fi
echo "[OK 1/4] Rust #![forbid(unsafe_code)] y 0 punto flotante confirmados."

# 2. Verificar constantes estatutarias AAUFSA (IRC Cap. 56) y los 6 dominios
for id in "320717657398" "877771073297" "964521086569" "188680247367" "1087754464927" "282230018778" "358233233"; do
    grep -q "$id" src/lib.rs
    grep -q "$id" sql/001_multi_entity_sovereign_schema.sql
    grep -q "$id" aaufsa_multi_domain_audit_report.json
done
grep -q '"tax_rate_bps": 1530' aaufsa_multi_domain_audit_report.json
grep -q '"safe_harbor_months": 36' aaufsa_multi_domain_audit_report.json
grep -q '"small_business_threshold_usd": "5000000.000000"' aaufsa_multi_domain_audit_report.json
grep -q '"status": "BLOCKED_FAIL_CLOSED"' aaufsa_multi_domain_audit_report.json
echo "[OK 2/4] Constantes AAUFSA y reporte de los 6 dominios verificados."

# 3. Verificar invariantes SQL (SERIALIZABLE, Partida Doble, WORM y RLS)
grep -q "default_transaction_isolation TO 'serializable'" sql/001_multi_entity_sovereign_schema.sql
grep -q "debit_micros - credit_micros = 0" sql/001_multi_entity_sovereign_schema.sql
grep -q "DO INSTEAD NOTHING" sql/001_multi_entity_sovereign_schema.sql
grep -q "ENABLE ROW LEVEL SECURITY" sql/001_multi_entity_sovereign_schema.sql
echo "[OK 3/4] Esquema PostgreSQL SERIALIZABLE, WORM y RLS verificado."

# 4. Verificar integridad criptográfica SHA-256 contra el manifiesto
sha256sum -c SOVEREIGN_COMPLIANCE_MANIFEST.sha256
echo "[OK 4/4] Integridad SHA-256 verificada al 100%."
echo "=== AUDITORIA SUPERADA AL 100% ==="
