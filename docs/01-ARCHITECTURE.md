# 01 · Architecture

## Overview
```
┌──────────────────────────── NLX_SEARCH.exe ────────────────────────────┐
│ Rust (src-tauri)                           WebView2 (src/, React + TS)  │
│                                                                          │
│  hook thread ──(mpsc)──▶ window controller ──emit──▶ UI (search bar)    │
│  (WH_KEYBOARD_LL)         show/hide/center/focus                         │
│                                                                          │
│  indexer thread ──▶ SQLite (items + FTS5) ◀── search command ◀── invoke │
│  USN/walkdir watcher        %LOCALAPPDATA%\NLX_SEARCH\index.db           │
│                                                                          │
│  app scanner (Start Menu .lnk + UWP AppsFolder)                          │
│  actions: open / open location / copy path / rename / delete / props     │
│  tray, autostart, single-instance                                        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Folder layout
```
nlx-search/
├─ src/                      # React UI
│  ├─ components/            # SearchBar, Sidebar, ResultRow, Section, StatusBar, ContextMenu
│  ├─ hooks/                 # useSearch (debounce + invoke), useKeyboardNav
│  ├─ styles/tokens.css      # CSS variables (colors, spacing)
│  ├─ assets/pig.png
│  └─ App.tsx
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs / lib.rs    # setup, plugins, window config
│  │  ├─ hook.rs             # Win key / Win+S / Win+Q interception
│  │  ├─ window.rs           # toggle, center, focus, acrylic
│  │  ├─ index/              # scanner.rs, watcher.rs, db.rs, schema.sql
│  │  ├─ search.rs           # query, ranking
│  │  ├─ apps.rs             # installed apps
│  │  ├─ actions.rs          # open, reveal, copy, rename, trash, properties
│  │  └─ icons.rs            # shell icon/thumbnail extraction + cache
│  ├─ capabilities/default.json
│  └─ tauri.conf.json
└─ docs/
```

## Data flow: one search
1. User presses **Win** → hook thread sends `()` on channel → window controller shows window and emits `nlx://shown`.
2. UI focuses the input; typing triggers `invoke("search", { query, filter })` after ~30 ms debounce.
3. Rust runs the SQLite query, ranks results, returns `Vec<SearchResult>` grouped by section.
4. UI renders sections: Best Match → Applications → Folders → Files → Images.
5. Enter runs `open`; Esc or focus loss hides the window.

## Key design decisions
| Decision | Reason |
|---|---|
| Tauri over Electron | ~10x smaller footprint, lower idle RAM, Rust for the hook and indexer |
| Window hidden, not closed | Instant show (<50 ms) after the first launch |
| SQLite FTS5 (trigram) | Substring search with no external service |
| Hook in its own thread | Low-level hooks need a message loop and must never block |
| Narrow IPC commands | Smaller attack surface than the fs/shell plugins |
