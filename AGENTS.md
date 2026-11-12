# AGENTS.md: rules for AI assistants on NLX_SEARCH

## Project
Tauri v2 desktop app for Windows 10 that replaces Start-menu search. Rust backend in `src-tauri/`, React + TypeScript UI in `src/`.

## Hard UI rules (never violate)
1. **No right-side preview/details panel.** Results use the full central width.
2. Keep: left sidebar, top search bar, pig logo, bottom status bar.
3. **No gradients.** No neon, no glowing borders, no oversized buttons.
4. Colors only from `docs/02-UI-SPEC.md` tokens.
5. Actions (Open, Open Location, Copy Path, Rename, Delete, Properties) appear inline or in a compact context menu, never in a permanent panel.

## Engineering rules
- The keyboard hook callback must do **no I/O and no allocation-heavy work**. Only flip atomics and send on a channel.
- Never log keystrokes. The hook may only read Win, S, Q state.
- File operations live in Rust commands, validate paths, and delete to the **Recycle Bin**, never permanent.
- Do not enable broad Tauri `fs` or `shell` plugin permissions. Expose narrow commands only.
- Keep IPC types in sync with `docs/06-IPC-COMMANDS.md`.
- Verify crate/API versions in official docs; do not guess signatures.

## Definition of done
- `cargo clippy` and `cargo fmt --check` clean
- `npm run build` clean
- Manual test matrix in `docs/09-TESTING-TROUBLESHOOTING.md` passes for touched areas
