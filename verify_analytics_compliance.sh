#!/usr/bin/env bash
set -Eeuo pipefail

echo "[STEP 1/5] Escaneo de Secretos..."
SECRET_MATCHES=$(grep -rnE "(password|secret|bearer|aws_secret_access_key|private_key|api_key)\s*[:=]\s*['\"][A-Za-z0-9/+=]{8,}['\"]" \
    --exclude="verify_analytics_compliance.sh" \
    --exclude-dir={.git,target} . || true)

if [ -n "$SECRET_MATCHES" ]; then
    echo "[FATAL] Secretos detectados"
    exit 1
fi
echo "[OK] Sin secretos."

echo "[STEP 2/5] Auditoría de Código Rust..."
if grep -rn "unsafe" src/ 2>/dev/null; then exit 1; fi
if grep -rnE "\b(f32|f64)\b" src/ 2>/dev/null; then exit 1; fi
if grep -rnE "(\.unwrap\(\)|\.expect\()" src/ 2>/dev/null; then exit 1; fi
echo "[OK] Rust seguro y determinista."

echo "[STEP 3/5] Verificación SQL..."
SQL_FILE="sql/ddl_immutable_analytics.sql"
grep -q "ON UPDATE TO certified_analysis_journal DO INSTEAD NOTHING" "$SQL_FILE"
grep -q "ENABLE ROW LEVEL SECURITY" "$SQL_FILE"
echo "[OK] SQL inmutable verificado."

echo "[STEP 4/5] Hash y Manifiesto de Integridad..."
MANIFEST_HASH=$(sha256sum Cargo.toml src/lib.rs src/main.rs sql/ddl_immutable_analytics.sql | sha256sum | awk '{print $1}')
echo "Hash: ${MANIFEST_HASH}"
echo "[OK] Verificación completada con éxito."
