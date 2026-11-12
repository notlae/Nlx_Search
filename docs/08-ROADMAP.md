# 08 · Roadmap

## Phase 0: Scaffold (day 1)
- [ ] `npm create tauri-app@latest` (React + TS)
- [ ] Add `window-vibrancy`, `rusqlite` (bundled), `windows`, `trash`, `notify`, `walkdir`/`ignore`
- [ ] Plugins: single-instance, autostart, global-shortcut (fallback hotkey, e.g. `Alt+Space`)
- [ ] Transparent, frameless, hidden window; tray icon with Show/Quit

## Phase 1: Static UI (days 2 to 4)
- [ ] `tokens.css`, title bar, search bar, sidebar, result rows, status bar
- [ ] Layout with **no right panel**; results fill the full width
- [ ] Mock data matching the CapCut example
- [ ] Keyboard navigation, selection styles, inline hover actions, context menu

## Phase 2: Window behavior (days 4 to 5)
- [ ] `window.rs`: toggle, center on cursor monitor, focus, hide on blur
- [ ] Acrylic applied with solid fallback
- [ ] Esc behavior, instant show

## Phase 3: Windows integration (days 5 to 7)
- [ ] `hook.rs` for Win, Win+S, Win+Q
- [ ] Dummy-key trick verified (Start never flashes)
- [ ] Tray "Pause hook" + `--no-hook` flag
- [ ] Hide taskbar search box; optional BingSearch registry tweak
- [ ] Elevated mode via scheduled task (optional)

## Phase 4: Index and search (days 7 to 12)
- [ ] SQLite schema + FTS5 trigram
- [ ] Walker with exclusions; app scanner (Start Menu + UWP)
- [ ] `search` command with ranking and sections
- [ ] Icons and thumbnails with cache
- [ ] File watcher for live updates
- [ ] Status bar: drives, index state, file count

## Phase 5: Actions (days 12 to 14)
- [ ] Open, Open Location, Copy Path, Rename, Delete (Recycle Bin), Properties, Run as admin
- [ ] Usage tracking (launch counts), favorites, recent searches

## Phase 6: Settings and polish (days 14 to 17)
- [ ] Settings view: autostart, hook toggle, exclusions, reindex, clear history
- [ ] Empty/error/loading states
- [ ] Performance pass: query latency, idle RAM, show time

## Phase 7: Release
- [ ] Code signing, installer (NSIS/MSI), WebView2 bootstrap
- [ ] Manual test matrix passes
- [ ] README, screenshots, changelog

## Later ideas
USN journal live indexing, MFT fast scan, calculator and unit conversion, clipboard history, plugin API, themes.

## Definition of done per phase
Builds clean, no clippy warnings, relevant tests in `09-TESTING-TROUBLESHOOTING.md` pass.
