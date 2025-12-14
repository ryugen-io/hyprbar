# Debug Logging Implementation for CLI Flags (2025-12-14)

Implemented `tracing::debug!` logs for the `--start`, `--restart`, and `--autostart` CLI flags in `ks-bin`.
This allows better visibility into daemon control operations when the `--debug` flag is used.

**Files Modified:**
- `ks-bin/src/main.rs`: Added logs before calling daemon functions.
- `ks-bin/src/modules/autostart.rs`: Added logs for script existence check and creation/removal actions.

**Usage:**
Run with `--debug` combined with other flags to see the output.
