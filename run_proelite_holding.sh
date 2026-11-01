#!/usr/bin/env bash
set -euo pipefail

echo "================================================================="
echo "  EJECUTANDO ORQUESTACIÓN PRO-ELITE (ORG 320717657398 - 36 SVCS) "
echo "================================================================="
cargo test --release --quiet
cargo run --release --quiet --bin proelite_master_orchestrator
cargo run --release --quiet --bin gemini_cognitive_prod
cargo run --release --quiet --bin middleware_compliance_prod
cargo run --release --quiet --bin auraintel_studios3d_prod
cargo run --release --quiet --bin sovereign_introspection_prod
./verify_all_entities_master_compliance.sh
echo "================================================================="
echo "  HOLDING SINCRONIZADO Y AUDITADO AL 100%                        "
echo "================================================================="
