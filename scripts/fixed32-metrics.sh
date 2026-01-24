#!/bin/bash
set -e

# Find workspace root (directory containing Cargo.toml)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default paths relative to workspace root
TESTS_DIR="${WORKSPACE_ROOT}/lp-glsl/apps/fixed32-metrics/glsl"
OUTPUT_DIR="${WORKSPACE_ROOT}/docs/reports/fixed32"
FORMAT="Fixed16x16"

# Run the app with defaults
cd "$WORKSPACE_ROOT"
cargo run --bin fixed32-metrics -- \
    --tests-dir "$TESTS_DIR" \
    --output-dir "$OUTPUT_DIR" \
    --format "$FORMAT" \
    "$@"
