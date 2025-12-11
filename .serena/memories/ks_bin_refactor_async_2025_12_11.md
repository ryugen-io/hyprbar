# ks-bin Refactor and Async Architecture - 2025-12-11

## Modularization
The `ks-bin/src/main.rs` file was refactored into a modular structure under `ks-bin/src/modules/`.
- `cli.rs`: Command line arguments definition.
- `logging.rs`: Tracing subscriber and global log channels.
- `config.rs`: Configuration loading.
- `daemon.rs`: Process spawning (daemon & debug viewer).
- `watcher.rs`: Client-side log watcher.
- `build.rs`: `WASH` command logic (compiling dishes).
- `install.rs`: `LOAD` command logic (installing dishes).
- `runner.rs`: Main event loop and server logic.

## IO Best Practices
We adopted `tokio` for all I/O heavy operations to ensure async best practices:
- **Process Spawning**: Uses `tokio::process::Command` instead of `std::process::Command` (except where simple `spawn` without await is sufficient, but consistency is preferred).
- **File System**: Uses `tokio::fs` for file operations in `build.rs` and `install.rs`.
- **Async Await**: `main` function awaits these async operations properly.
- **Watcher**: Uses `AsyncReadExt` for reading the socket stream instead of blocking/polling.

## Rationale
This ensures that the `ks-bin` binary is robust, maintainable, and does not block the async runtime, paving the way for future concurrency features.
