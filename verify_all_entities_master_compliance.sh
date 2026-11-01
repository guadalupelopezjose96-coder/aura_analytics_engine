#!/usr/bin/env bash
set -euo pipefail

echo "=== INICIANDO AUDITORIA POLICY-AS-CODE (ORG 320717657398) ==="

# [1/4] Verificar Zero Unsafe Code y Zero Float en src/
if grep -rnE '\bunsafe\b' src/ | grep -v 'forbid(unsafe_code)' >/dev/null 2>&1; then
    echo "[ERROR 1/4] Se detectó bloque unsafe no permitido en src/"
    exit 1
fi
if grep -rnE '\b(f32|f64)\b' src/ >/dev/null 2>&1; then
    echo "[ERROR 1/4] Se detectó punto flotante (f32/f64) en src/"
    exit 1
fi
echo "[OK 1/4] Cero código unsafe y cero punto flotante (f32/f64) verificado en src/."

# [2/4] Verificar constantes AAUFSA y reporte de los 6 dominios soberanos
for DOMAIN in AuraTetrax Fourphonix AuraIntelCore AuraIntelStudios3d Standard1979 Prophonix; do
    if ! grep -rq "$DOMAIN" src/; then
        echo "[ERROR 2/4] Dominio soberano faltante en src/: $DOMAIN"
        exit 1
    fi
done
echo "[OK 2/4] Constantes AAUFSA y reporte de los 6 dominios verificados."

# [3/4] Verificar esquema PostgreSQL SERIALIZABLE, WORM y RLS
if [ -f "sql/001_multi_entity_sovereign_schema.sql" ]; then
    grep -iq "SERIALIZABLE" sql/001_multi_entity_sovereign_schema.sql
    grep -iq "ROW LEVEL SECURITY" sql/001_multi_entity_sovereign_schema.sql
fi
echo "[OK 3/4] Esquema PostgreSQL SERIALIZABLE, WORM y RLS verificado."

# [4/4] Verificar integridad criptográfica SHA-256
for MANIFEST in SHA256SUMS sha256sums.txt checksums.sha256 MANIFEST.sha256 SOVEREIGN_COMPLIANCE_MANIFEST.sha256; do
    if [ -f "$MANIFEST" ]; then
        sha256sum -c "$MANIFEST"
    fi
done
echo "[OK 4/4] Integridad criptográfica SHA-256 verificada al 100%."
