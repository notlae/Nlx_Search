# 11 · Distribution: one .exe on GitHub Releases

Goal: someone downloads **one file** from your GitHub release, double-clicks it, and NLX_SEARCH runs and asks permission to take over Windows search.

## What you ship
| Asset | What it is | Who it's for |
|---|---|---|
| `NLX_SEARCH-setup.exe` | NSIS installer built by Tauri | Most users (recommended) |
| `NLX_SEARCH-portable.exe` | The raw release exe | People who want a single file, no installer |
| `SHA256SUMS.txt` | Checksums | Verification |

Tauri's `tauri build` produces the bundle installers and the plain exe in `src-tauri/target/release/`. Exact output names depend on `productName` and the Cargo package name, so check after your first build.

## WebView2 requirement
The UI runs in Microsoft WebView2.
- The **installer** can bootstrap WebView2 (set `webviewInstallMode` in the Tauri bundle config; check the docs for the Windows options).
- The **portable exe** needs WebView2 already installed. Many Windows 10 machines have it via Edge updates, but not all. On startup, detect a missing runtime and show a message with the Microsoft download link instead of crashing.

## Build on GitHub automatically (`.github/workflows/release.yml`)
Push a tag like `v0.1.0` and GitHub builds the Windows release for you. This also means you don't need a perfect local setup, and every release is built from clean source.

See the included `.github/workflows/release.yml`. Steps:
1. Create the repo, push the project.
2. `git tag v0.1.0 && git push origin v0.1.0`
3. Wait for the Action to finish. It creates a **draft release** with the installer attached.
4. Test the draft on a clean Windows 10 machine or VM, then publish.

## Windows SmartScreen and antivirus (important)
- An **unsigned** exe shows "Windows protected your PC / Unknown publisher". Users must click *More info → Run anyway*.
- A global keyboard hook makes antivirus heuristics suspicious. Expect some false positives.
- Mitigations, best first:
  1. **Code-sign** the exe. Options: a paid certificate, or a free open-source signing program such as SignPath Foundation (check current eligibility and terms).
  2. Open-source the code so people can inspect the hook.
  3. Publish `SHA256SUMS.txt` and the VirusTotal link in the release notes.
  4. Submit false-positive reports to Microsoft Defender and other vendors.
  5. Tell users plainly in the README what the hook does (only watches Win/S/Q state, never logs keys).

## Release notes template
```
## NLX_SEARCH vX.Y.Z
Download: NLX_SEARCH-setup.exe (recommended) or NLX_SEARCH-portable.exe
Requires: Windows 10 1809+ (WebView2 runtime)
First run: the app asks for your permission before changing how Win / Win+S behave.
Undo anytime: tray icon → Pause / Restore Windows Search, or Settings → Uninstall.
SHA256: ...
```

## Release checklist
- [ ] Version bumped in `tauri.conf.json`, `Cargo.toml`, `package.json`
- [ ] CI build green
- [ ] Installed and run on a **clean** Windows 10 VM
- [ ] Test matrix (doc 09) passes
- [ ] Uninstall removes autostart entry, scheduled task and app data (if chosen)
- [ ] Signed (if available)
- [ ] Checksums attached
- [ ] Draft reviewed, then published

## About "no mistakes"
Nobody can promise zero bugs. What you *can* do: build in CI, test on a clean VM every release, keep a one-click way to restore Windows search, and use the safety design in doc 12. The hook runs inside the app's process, so if the app crashes or is closed, Windows returns to normal behavior automatically.
