# Automatic Log Forwarding (2025-12-14)

Implemented bidirectional socket logging to support a unified debug terminal workflow.

**Feature:**
Commands like `kitchnsink --restart` automatically forward their logs to the `kitchnsink --debug` viewer session if it is running.
No extra flags are required on the ephemeral commands.

**Implementation Details:**
- **Daemon**: Splits socket connections. Spawns a reader task to broadcast incoming lines to the global `LOG_CHANNEL`.
- **Client**: `init_logging` proactively connects to the debug socket if it exists. Uses `SocketPublisherLayer` to write logs. Use `shutdown(Shutdown::Read)` to avoid echo/blocking.

**File:** `ks-bin/src/modules/logging.rs`
