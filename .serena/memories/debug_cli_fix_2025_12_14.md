# Debug CLI Logic Fix (2025-12-14)

Fixed a bug in `ks-bin` where `kitchnsink --debug [flag]` would:
1. Ignore the flag and spawn a new daemon/viewer (Detached Debug Mode trap in `main.rs`).
2. Clobber the existing debug socket, disconnecting any active viewer (`logging.rs` socket binding).

**Fix Details:**
- `main.rs`: Modified "Detached Debug" condition to exclude cases where action flags (`--start`, `--restart`, etc.) are present.
- `main.rs`: Only requested socket binding for `InternalRun` subcommand.
- `logging.rs`: Added `bind_socket` parameter to `init_logging` to conditionally bind the unix listener.

This allows users to run `kitchnsink --debug` in one terminal and `kitchnsink --debug --restart` in another without issues.
