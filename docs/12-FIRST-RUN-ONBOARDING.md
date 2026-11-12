# 12 · First-run Onboarding and Safe Takeover

When a stranger runs the exe for the first time, NLX_SEARCH must **explain and ask**, never silently hijack the Win key.

## Flow
```
Launch exe
  ├─ already installed in %LOCALAPPDATA%\Programs\NLX_SEARCH ? → normal start
  └─ first run → Onboarding window
        Step 1  Welcome
        Step 2  Permissions (toggles, see below)
        Step 3  Index setup (which drives/folders)
        Step 4  Test the Windows key only if the user opted in; otherwise open from the shortcut or tray.
        Done    Window hides to tray; launcher is live
```

## Step 2: permission screen
Plain-language copy, one toggle each (defaults ON except elevation):

| Toggle | Explanation shown to the user |
|---|---|
| **Open NLX_SEARCH with the Windows key, Win+S and Win+Q** | "Optional, disabled by default. Restart NLX_SEARCH after changing this setting." |
| **Start with Windows** | "Runs quietly in the tray at login." |
| **Index my files** | "Builds a local search index on this PC. Nothing leaves your computer." |
| **Work inside admin windows** (off by default) | "Needs a one-time Windows permission prompt (UAC). Lets the Win key work even while an admin app is focused." |

Always show beneath the toggles:
> NLX_SEARCH only watches whether the Win, S and Q keys are pressed. It never records what you type, and it makes no network connections.
> You can undo everything from the tray icon at any time.

Buttons: **Enable** and **Not now (open from shortcut or tray)**. "Not now" must still leave a working fallback hotkey.

## Self-install (portable exe → stable path)
Autostart and scheduled tasks need a stable path, but a downloaded exe sits in `Downloads`. On consent:
1. Copy the exe to `%LOCALAPPDATA%\Programs\NLX_SEARCH\NLX_SEARCH.exe`.
2. Register autostart pointing to that copy (HKCU Run key via `tauri-plugin-autostart`, or the registry command in doc 03).
3. Relaunch from the installed copy and exit the original.
Skip this when installed by the setup exe.

## Optional elevated mode (UAC once)
Creating a "highest privileges at logon" scheduled task needs admin rights:
1. App relaunches itself elevated with `ShellExecuteExW` verb `runas`, flag `--install-task`.
2. Elevated instance runs: `schtasks /Create /TN "NLX_SEARCH" /TR "\"<path>\"" /SC ONLOGON /RL HIGHEST /F`
3. It exits; the normal instance continues. Future logins start the app elevated with **no** UAC prompt.
Removal: `schtasks /Delete /TN "NLX_SEARCH" /F` (needs elevation too).

## Safety design (this is what prevents "locked out of Start")
- The keyboard hook lives **inside the process**. App closed or crashed → hook gone → Windows behaves normally.
- Tray menu: **Pause / Resume Windows-key takeover**, **Restore Windows Search**, **Open Settings**, **Quit**.
- Launch flag `--no-hook` starts the app without the hook.
- Settings → **Uninstall NLX_SEARCH**: unhooks, removes autostart entry and scheduled task, optionally deletes index/data, deletes the installed copy.
- When takeover is disabled, launch NLX_SEARCH from its app shortcut or system tray.
- Hook health check: if the hook thread dies, restart it, and after 3 failures disable it and notify the user.

## Settings to persist (`%LOCALAPPDATA%\NLX_SEARCH\settings.json`)
```json
{
  "firstRunDone": true,
  "hookEnabled": true,
  "autostart": true,
  "elevatedTask": false,
  "fallbackHotkey": null,
  "excludedPaths": [],
  "indexedDrives": ["C:", "D:"]
}
```

## "Crazy" polish ideas (after it works)
Smooth open animation (<150 ms), calculator and unit conversion inside the search bar, clipboard history, web-search fallback line, app-specific commands (`> restart explorer`), per-category result shortcuts, theme packs, pig mascot idle/empty-state animation. Add these only after the core hook, index and actions are solid.
