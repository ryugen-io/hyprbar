# Installation Fix (2025-12-14)

Updated `install.sh` to prevent `Text file busy` errors.
The script now calls `kitchnsink --stop` (or `pkill`) and waits 1s before overwriting the binary.
Also updated `justfile` to run `./install.sh` as part of `just install`.

**File:** `install.sh`, `justfile`
