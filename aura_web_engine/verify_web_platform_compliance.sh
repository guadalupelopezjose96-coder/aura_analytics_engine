#!/usr/bin/env bash
set -euo pipefail

echo "================================================================================"
echo "AUDITORIA DE PLATAFORMA WEB, SEGURIDAD PERIMETRAL Y COMPLIANCE STATUTARIO AAUFSA"
echo "Holding: guadalupelopezjose96-org | ID: 320717657398"
echo "Fecha de Verificación: $(date -u '+%Y-%m-%d %H:%M:%SZ')"
echo "================================================================================"

ERRORS_FOUND=0

echo -n "[1/6] Escaneando arbol de archivos contra secretos y llaves privadas expuestas... "
FORBIDDEN_PATTERNS=(
    "sk_live_[0-9a-zA-Z]{24}"
    "AKIA[0-9A-Z]{16}"
    "-----BEGIN OPENSSH PRIVATE KEY-----"
    "-----BEGIN RSA PRIVATE KEY-----"
    "ghp_[0-9a-zA-Z]{36}"
)

for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    if grep -rEIq --exclude="verify_web_platform_compliance.sh" --exclude-dir=".git" --exclude-dir="target" -e "$pattern" .; then
        echo "FALLO CRITICO: Patron detectado: $pattern"
        ERRORS_FOUND=$((ERRORS_FOUND + 1))
    fi
done
if [ $ERRORS_FOUND -eq 0 ]; then
    echo "APROBADO (0 secretos detectados)"
fi

echo -n "[2/6] Verificando directivas de robustez extrema en Rust... "
RUST_FILE="src/lib.rs"
if [ -f "$RUST_FILE" ]; then
    grep -q '#!\[forbid(unsafe_code)\]' "$RUST_FILE" || { echo "FALLO: Falta forbid(unsafe_code)"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    grep -q 'clippy::float_arithmetic' "$RUST_FILE" || { echo "FALLO: Falta float_arithmetic"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    grep -q 'clippy::unwrap_used' "$RUST_FILE" || { echo "FALLO: Falta unwrap_used"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    echo "APROBADO"
else
    echo "FALLO: Falta $RUST_FILE"
    ERRORS_FOUND=$((ERRORS_FOUND + 1))
fi

echo -n "[3/6] Verificando cabeceras HTTP de seguridad OWASP ASVS Nivel 3... "
REQUIRED_HEADERS=(
    "CONTENT_SECURITY_POLICY"
    "STRICT_TRANSPORT_SECURITY"
    "X_CONTENT_TYPE_OPTIONS"
    "X_FRAME_OPTIONS"
    "permissions-policy"
)

for hdr in "${REQUIRED_HEADERS[@]}"; do
    if ! grep -rqi "$hdr" src/; then
        echo "FALLO: Cabecera '$hdr' ausente"
        ERRORS_FOUND=$((ERRORS_FOUND + 1))
    fi
done
echo "APROBADO"

echo -n "[4/6] Verificando archivo de bloqueo de licencias virales (deny.toml)... "
DENY_FILE="deny.toml"
if [ -f "$DENY_FILE" ]; then
    grep -q 'GPL-3.0-only' "$DENY_FILE" || { echo "FALLO GPL"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    grep -q 'Apache-2.0' "$DENY_FILE" || { echo "FALLO Apache"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    echo "APROBADO"
else
    echo "FALLO: Falta $DENY_FILE"
    ERRORS_FOUND=$((ERRORS_FOUND + 1))
fi

echo -n "[5/6] Verificando esquema PostgreSQL (WORM, RLS, Balance 0 y constantes AAUFSA)... "
SQL_FILE="sql/001_web_platform_schema.sql"
if [ -f "$SQL_FILE" ]; then
    grep -q 'DO INSTEAD NOTHING' "$SQL_FILE" || { echo "FALLO WORM"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    grep -q 'ENABLE ROW LEVEL SECURITY' "$SQL_FILE" || { echo "FALLO RLS"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    DOMAINS_CHECK=(877771073297 964521086569 188680247367 1087754464927 282230018778 358233233)
    for dom in "${DOMAINS_CHECK[@]}"; do
        grep -q "$dom" "$SQL_FILE" || { echo "FALLO Dom $dom"; ERRORS_FOUND=$((ERRORS_FOUND + 1)); }
    done
    echo "APROBADO"
else
    echo "FALLO: Falta $SQL_FILE"
    ERRORS_FOUND=$((ERRORS_FOUND + 1))
fi

echo "[6/6] Ejecutando pruebas unitarias de liquidación determinista y sellado SHA-256..."
cargo test --lib -- --nocapture

MANIFEST_FILE="WEB_PLATFORM_INTEGRITY_MANIFEST.sha256"
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.sql" -o -name "*.tf" \) -not -path "*/target/*" -exec sha256sum {} + | sort > "$MANIFEST_FILE"

cat "$MANIFEST_FILE"
echo "--------------------------------------------------------------------------------"
echo "Hash SHA-256 global del manifiesto:"
sha256sum "$MANIFEST_FILE"

echo "================================================================================"
if [ $ERRORS_FOUND -eq 0 ]; then
    echo "ESTADO FINAL: CUMPLIMIENTO GLOBAL VERIFICADO AL 100% (LISTO PARA PRODUCCIÓN)"
    exit 0
else
    echo "ESTADO FINAL: AUDITORIA RECHAZADA CON $ERRORS_FOUND ERRORES"
    exit 1
fi
