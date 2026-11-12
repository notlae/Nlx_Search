# 07 · Security and Privacy

NLX_SEARCH can optionally install a global low-level keyboard hook to take over Windows search shortcuts. This feature is **off by default**.

## Principles
1. **No keystroke logging.** The hook only tracks modifier/trigger-key state needed for the configured shortcut. It does not record typed text.
2. **Local-first.** Index and settings are stored under `%LOCALAPPDATA%\NLX_SEARCH`.
3. **Least privilege.** Normal search and startup do not require administrator rights.
4. **Opt-in keyboard integration.** The hook is installed only when the user enables takeover and launches the app.
5. **Transparent startup.** Optional startup uses the current-user Run registry value. No elevated scheduled task, self-copy, or PowerShell startup script is used.

## File-operation safety
- Actions are restricted to indexed paths.
- Rename rejects Windows-invalid filename characters and updates indexed descendants transactionally.
- Delete moves items to the Recycle Bin and requires UI confirmation.

## Hook safety
- Keep Windows Start/Search recovery available through Settings and the tray menu.
- If takeover is changed, restart the app so the hook is installed or unloaded cleanly.
- `--no-hook` starts without the global hook.

## Distribution
- Build reproducibly and publish checksums.
- Sign release binaries when a trusted code-signing certificate is available.
- Submit suspected false positives to Microsoft for review; never ask users to disable antivirus.
