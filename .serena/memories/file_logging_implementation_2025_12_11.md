# File Logging Implementation (2025-12-11)

## Summary
Integrated `kitchn_lib`'s file logging capabilities into `ks-bin` to ensure compliance with the kitchn ecosystem's log file structure (e.g., `~/.local/state/kitchn/logs/...`).

## Technical Details

### 1. `KitchnFileLayer`
Created a custom `tracing_subscriber::Layer` in `ks-bin/src/modules/logging.rs` that intercepts `tracing` events and forwards them to `k_lib::logger::log_to_file`.
- Respects `write_by_default` setting from `layout.toml`.
- Uses synchronous file I/O (bridging tracing's async/sync boundary).

### 2. Architecture Changes
- **Shared Cookbook**: Refactored `run_server` and `init_logging` to accept `std::sync::Arc<Cookbook>`. This was necessary because `Cookbook` is not `Clone` and needs to be shared between the main logic (runner) and the logging subsystem.
- **LogTracer**: Enabled `tracing_log::LogTracer` to capture standard `log` crate macros (used by plugins/dishes) and route them through the tracing subscriber.

### 3. Usage
- Logs are automatically written to `~/.local/state/kitchn/logs/<YY>/<MM>/kitchnsink/`.
- App name override "kitchnsink" is properly applied.
- Dishes/Plugins using `log::info!` will have their logs captured automatically.
