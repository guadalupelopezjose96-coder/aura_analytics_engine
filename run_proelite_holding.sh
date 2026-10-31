#!/usr/bin/env bash
set -euo pipefail

echo "================================================================="
echo "  EJECUTANDO ORQUESTACIÓN PRO-ELITE (ORG 320717657398 - 36 SVCS) "
echo "================================================================="
cargo test --release --quiet
cargo run --release --quiet --bin proelite-master-orchestrator
cargo run --release --quiet --bin gemini-cognitive-prod
cargo run --release --quiet --bin middleware-compliance-prod
cargo run --release --quiet --bin auraintel-studios3d-prod
cargo run --release --quiet --bin sovereign-introspection-prod
./verify_all_entities_master_compliance.sh
echo "================================================================="
echo "  HOLDING SINCRONIZADO Y AUDITADO AL 100%                        "
echo "================================================================="
