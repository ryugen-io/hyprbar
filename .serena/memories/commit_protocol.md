# Commit Protocol

The user has enforced a strict rule for git commits to avoid shell issues with multiline messages.

**Rule:** ALWAYS use a temporary file for git commits with the `-F` flag.

**Workflow:**
1. Write the commit message to a temporary file (e.g., `.git_commit_msg_temp`).
2. Run `git commit -F .git_commit_msg_temp`.
3. (Optional) Delete the temporary file.

**Do NOT** use `git commit -m "..."` for multiline messages.
