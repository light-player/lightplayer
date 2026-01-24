# Phase 7: Create Wrapper Script

## Description

Create a wrapper script in `scripts/` that runs the app with good defaults for tests directory and output directory.

## Implementation

- Create `scripts/fixed32-metrics.sh`
- Script should:
  - Set default `--tests-dir` to `lp-glsl/apps/fixed32-metrics/glsl` (relative to workspace root)
  - Set default `--output-dir` to `docs/reports/fixed32` (relative to workspace root)
  - Set default `--format` to `Fixed16x16`
  - Run `cargo run --bin fixed32-metrics` with the arguments
  - Handle workspace root detection (find Cargo.toml or use current directory)
- Make script executable

## Success Criteria

- Script runs successfully with defaults
- Default paths are correct relative to workspace root
- Script is executable
- App runs correctly when invoked via script
