# 09 · Testing and Troubleshooting

## Manual test matrix
| # | Test | Expected |
|---|---|---|
| 1 | Press `Win` alone | NLX_SEARCH opens; Start menu does **not** |
| 2 | `Win+S`, `Win+Q` | NLX_SEARCH opens; Windows Search does not |
| 3 | `Win+E`, `Win+D`, `Win+R`, `Win+L` | Behave normally |
| 4 | Hold `Win`, press `Tab`/`Arrow` | Normal Windows behavior |
| 5 | Press `Win` while launcher is open | Launcher hides |
| 6 | Type "capcut" | Results < 50 ms; Best Match is the app |
| 7 | Arrow keys + Enter | Opens the selected item |
| 8 | Hover a row | Inline actions appear; no side panel anywhere |
| 9 | Delete a test file | Confirms, then lands in Recycle Bin |
| 10 | Focus an elevated window, press `Win` | Documented UIPI limitation (or works in elevated mode) |
| 11 | Tray → Pause hook, press `Win` | Normal Start opens |
| 12 | Launch twice | Only one instance; window shows |
| 13 | Sleep/resume, lock/unlock | Hook still works |
| 14 | Multi-monitor | Window appears on the cursor's monitor |
| 15 | Idle for 10 min | Near-zero CPU, stable RAM |

## Troubleshooting
| Problem | Likely cause | Fix |
|---|---|---|
| Start menu still flashes on `Win` | Injected dummy key delivered after Win-up | Swallow real Win-up, then inject dummy-down, dummy-up, Win-up (see doc 03) |
| Hook stops working randomly | Callback too slow, or hook handle dropped | Keep callback trivial; keep the thread alive; reinstall hook on `WM_WTSSESSION_CHANGE` |
| Hook ignored in admin windows | UIPI | Run elevated via scheduled task |
| Window shows but has no keyboard focus | Foreground restrictions | `AttachThreadInput` workaround; confirm trigger came from user input |
| White box behind UI | Opaque `body` background | `background: transparent` on html/body/#root |
| Blur has square corners | Acrylic fills the window rect | See doc 05 limits |
| `apply_acrylic` fails | Unsupported Windows build | Fall back to solid `--bg` |
| Antivirus flags exe | Global keyboard hook heuristics | Code-sign; submit false-positive report |
| Slow first search | Index still building | Show indexing state; cache recent |
| UWP apps missing | Only scanning `.lnk` files | Add `shell:AppsFolder` enumeration |
| Build error on `windows` crate types | API differs across versions | Check docs.rs for the pinned version |

## Dev commands
```bash
npm run tauri dev
npm run tauri build
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

## Performance targets
| Metric | Target |
|---|---|
| Show time after hotkey | < 50 ms |
| Query latency (1M items) | < 30 ms |
| Idle RAM | as low as practical (measure, then set a budget) |
| Idle CPU | ~0% |
