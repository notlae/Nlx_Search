# 04 · Indexer and Search

## Goals
- Results in **< 30 ms** for typical queries on ~1M files
- Substring matching ("capc" finds CapCut), live updates, low idle CPU/RAM

## Strategy by phase
| Phase | Method | Admin? |
|---|---|---|
| 1 | Directory walk (`ignore`/`walkdir` crate) into SQLite | No |
| 2 | Watch changes (`notify` crate) | No |
| 3 | NTFS **USN journal** for live changes | Usually yes |
| 4 | Fast full scan by reading the NTFS MFT (Everything-style) | Yes |

Start with phase 1 and 2. Phases 3 and 4 are optimizations.

## Schema (`schema.sql`)
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE IF NOT EXISTS items (
  id     INTEGER PRIMARY KEY,
  name   TEXT NOT NULL,
  path   TEXT NOT NULL UNIQUE,
  parent TEXT NOT NULL,
  ext    TEXT,
  kind   INTEGER NOT NULL,   -- 0 file, 1 folder, 2 app, 3 image, 4 video, 5 audio
  size   INTEGER,
  mtime  INTEGER             -- unix seconds
);

-- trigram tokenizer = substring search (needs SQLite >= 3.34, queries >= 3 chars)
CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
  name, content='items', content_rowid='id', tokenize='trigram'
);

CREATE TABLE IF NOT EXISTS usage (
  path TEXT PRIMARY KEY, launches INTEGER DEFAULT 0, last_used INTEGER
);
CREATE TABLE IF NOT EXISTS favorites (path TEXT PRIMARY KEY);
CREATE TABLE IF NOT EXISTS recent_searches (q TEXT PRIMARY KEY, ts INTEGER);
```
Keep `items_fts` in sync with triggers (insert/update/delete) or rebuild after bulk scans.

Use `rusqlite` with the `bundled` feature so you control the SQLite version.

## Query logic
1. Query length **≥ 3**: `SELECT ... FROM items_fts JOIN items ... WHERE items_fts MATCH ?` with the query quoted.
2. Length **1–2**: `WHERE name LIKE ? ESCAPE '\'` with a prefix match, limit 50.
3. Apply the category filter (`kind IN (...)`).
4. Cap candidates at ~200, rank in Rust, return top ~40.

## Ranking
```
score =  1000 if name == query (case-insensitive)
       +  600 if name starts_with query
       +  300 if a word in name starts_with query
       +  100 if name contains query
       +   40 if path contains query
       +  min(launches, 50) * 8          // frequency
       +  recency boost (decays over 30 days)
       +  150 if kind == app              // apps first for short queries
       +   80 if favorite
       -  penalty for very deep paths / system folders
```
**Best Match** = highest score overall. Group the remaining results into sections: Applications, Folders, Files, Images.

## Installed apps
- Classic: scan `%ProgramData%\Microsoft\Windows\Start Menu\Programs` and `%AppData%\Microsoft\Windows\Start Menu\Programs` for `.lnk`, resolve targets via the `IShellLink` COM interface.
- UWP/Store apps: enumerate `shell:AppsFolder` (same data as PowerShell `Get-StartApps`).
- Launch UWP: `explorer.exe shell:AppsFolder\<AppUserModelID>`.

## Exclusions
Default exclude: `Windows\`, `$Recycle.Bin`, `System Volume Information`, `node_modules`, `.git`, `AppData\Local\Temp`, pagefile/hiberfil. Make the list editable in Settings.

## Icons and thumbnails
- Icons: shell APIs (`SHGetFileInfo` or `IShellItemImageFactory`), cached to PNG under `%LOCALAPPDATA%\NLX_SEARCH\icons\<hash>.png`.
- Images/videos: thumbnails via `IShellItemImageFactory` (32 to 48 px). Load lazily, only for visible rows.

## Status bar data
- Free space: `GetDiskFreeSpaceExW` per fixed drive.
- Indexing: state (`idle`/`scanning`) + total item count, pushed through the `nlx://index-status` event.

## Performance checklist
- Batch inserts in transactions of 10k rows
- Scan on a low-priority thread
- Debounce UI queries about 30 ms and cancel stale ones (use a query-id counter)
- Never block the UI thread
