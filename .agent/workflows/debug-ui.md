---
description: Debug the UI by capturing a screenshot and validating the configuration
---

1. Run the debug inspector to capture a screenshot and dump the configuration for verification.
// turbo
2. tools/.venv/bin/python3 tools/debug_view.py --dump-json

3. Review the JSON output above for any configuration errors.
4. A screenshot has been saved to `tools/.screenshots/`. Open the latest file in that directory to verify the visual state.
