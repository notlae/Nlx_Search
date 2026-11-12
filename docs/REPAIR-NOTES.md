# NLX_SEARCH repair notes

This repair pass addresses the concrete defects found during static inspection of the supplied archive.

## Changes

- Folder/file rename now migrates indexed descendant paths and parent paths in one SQLite transaction.
- Favorites and usage-history paths are migrated for both the renamed item and descendants.
- A failed database migration attempts to rename the filesystem item back and reports if rollback also fails.
- Rename path rewriting uses SQLite `length()` so Unicode path segments are not offset using UTF-8 byte counts.
- Excluded directory matching now checks directory boundaries, avoiding accidental matches such as `Private` matching `PrivateStuff`.
- Indexing can be cancelled when indexing is disabled. A cancelled scan does not prune records from an incomplete generation.
- The indexer avoids repeatedly reading settings from disk for every visited entry.
- Keyboard takeover now defaults to off, and the global hook is not installed unless the user opted in before launch.
- Legacy settings without a theme are migrated with keyboard takeover and administrator startup disabled.
- Startup now uses only the current-user Run registry value; hidden elevated PowerShell and scheduled-task creation were removed.
- Startup no longer self-copies the executable into AppData.
- Added persistent Aurora, Midnight, Neon and Light themes with a refreshed glass-style interface.
- Settings UI now reports reindex/uninstall errors and handles a failed settings reload without throwing a second unhandled error.
- Rename focus timers are cleaned up; keyboard navigation shortcuts continue to work while the search box is focused without hijacking ordinary text-editing keys.
- README rewritten with project overview, stack, installation, shortcuts, privacy notes, roadmap, development checks and screenshot placeholders.

## Verification status

- JSON files: parsed successfully.
- TOML files: parsed successfully.
- TypeScript build: not verified. `npm install` timed out in this environment, leaving React, Tauri and Vite dependencies unavailable; `tsc` therefore reported missing dependency/type declarations.
- Rust build/tests: not run; Rust/Cargo are not installed in the inspection environment.
- Windows runtime tests: not run; this environment is not Windows. Build and test the release on the target Windows machine before distributing it.

## Required checks on Windows

Run from the project root:

```powershell
npm install
npm run typecheck
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cd ..
npm run tauri dev
```

Manually test nested-folder rename, Unicode folder names, favorites/history after rename, exclusion boundaries, cancellation during a scan, startup registration/removal, keyboard-hook opt-in and opt-out across restarts, and Windows Search recovery.
