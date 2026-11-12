# 03 · Windows Integration (replacing Windows Search)

> Windows has **no supported API** to replace Start/Search. The approach: intercept the triggers and show NLX_SEARCH. Verify behavior on your Windows 10 build.

## Triggers
| Trigger | Method | Notes |
|---|---|---|
| `Win` alone | `WH_KEYBOARD_LL` hook + dummy-key trick | Start opens on Win **key-up** only if no other key was pressed |
| `Win+S`, `Win+Q` | Same hook, swallow `S`/`Q` | Return 1 from the hook |
| Typing after Start | Covered, since Start no longer opens | |
| Taskbar search box | Hide: right-click taskbar → Search → Hidden | |
| Start **button click** | ❌ Not reliably interceptable | Use Open-Shell / StartAllBack, or accept |

`RegisterHotKey` can't be used for these: Explorer owns `Win+S` and the bare Win key (error 1409).

## `src-tauri/src/hook.rs` (sketch for `windows` crate)
```rust
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::{mpsc::Sender, OnceLock};
use std::thread;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static TX: OnceLock<Sender<()>> = OnceLock::new();
static WIN_DOWN: AtomicBool = AtomicBool::new(false);
static OTHER_KEY: AtomicBool = AtomicBool::new(false);
static CONSUMED: AtomicBool = AtomicBool::new(false);

const VK_DUMMY: u16 = 0xE8; // unassigned key

fn trigger() { if let Some(tx) = TX.get() { let _ = tx.send(()); } }

fn send_dummy() {
    let key = |up: bool| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT {
            wVk: VIRTUAL_KEY(VK_DUMMY), wScan: 0,
            dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
            time: 0, dwExtraInfo: 0,
        }},
    };
    let inputs = [key(false), key(true)];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32); }
}

unsafe extern "system" fn proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let injected = kb.flags.0 & LLKHF_INJECTED.0 != 0;
        if !injected {
            let vk = kb.vkCode as u16;
            let msg = wparam.0 as u32;
            let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            let is_win = vk == VK_LWIN.0 || vk == VK_RWIN.0;

            if is_win && down && !WIN_DOWN.load(SeqCst) {
                WIN_DOWN.store(true, SeqCst);
                OTHER_KEY.store(false, SeqCst);
                CONSUMED.store(false, SeqCst);
            } else if !is_win && down && WIN_DOWN.load(SeqCst) {
                OTHER_KEY.store(true, SeqCst);
                if vk == VK_S.0 || vk == VK_Q.0 {
                    CONSUMED.store(true, SeqCst);
                    trigger();
                    return LRESULT(1); // swallow
                }
            } else if is_win && up && WIN_DOWN.load(SeqCst) {
                WIN_DOWN.store(false, SeqCst);
                if !OTHER_KEY.load(SeqCst) { trigger(); send_dummy(); }
                else if CONSUMED.load(SeqCst) { send_dummy(); }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

pub fn start(tx: Sender<()>) {
    let _ = TX.set(tx);
    thread::spawn(|| unsafe {
        let h = GetModuleHandleW(None).expect("module");
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc), HINSTANCE(h.0), 0).expect("hook");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWindowsHookEx(hook);
    });
}
```
> Field types and `Result` returns differ between `windows` crate versions. Check docs.rs for the version you pin and adjust.

## Wiring in `lib.rs`
```rust
let (tx, rx) = std::sync::mpsc::channel();
hook::start(tx);
let handle = app.handle().clone();
std::thread::spawn(move || for _ in rx { window::toggle(&handle); });
```

## Why the dummy key works
Start opens on Win key-up only if no other key was pressed while Win was held. Injecting an unassigned key (`0xE8`) makes Windows treat the press as a combo (the same mask-key technique AutoHotkey uses), so Start stays closed. The `LLKHF_INJECTED` check stops the hook reacting to its own dummy key.

⚠️ **Ordering risk:** events injected inside a hook callback may be delivered after the Win-up. If Start still flashes in testing, swallow the real Win-up (`return LRESULT(1)`), then inject dummy-down, dummy-up, Win-up in that order.

## Foreground focus
Windows restricts `SetForegroundWindow`. Since the trigger is user input, `set_focus()` usually works. If not, use the `AttachThreadInput` workaround or briefly simulate an Alt press.

## Autostart, single instance, tray
- `tauri-plugin-autostart` (HKCU Run key), or:
  `reg add HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v NLX_SEARCH /d "C:\Path\NLX_SEARCH.exe" /f`
- `tauri-plugin-single-instance`: a second launch just shows the window.
- Tray icon: Show, Settings, Pause hook, Quit. **Always provide "Pause hook"**, so users can't be locked out of Start by a bug.

## Disable web results in Windows search (optional)
```bat
reg add HKCU\Software\Microsoft\Windows\CurrentVersion\Search /v BingSearchEnabled /t REG_DWORD /d 0 /f
```

## Elevated windows (UIPI)
A normal-integrity hook does not see keys while an **admin window** is focused (Task Manager, elevated terminal). Options:
1. Run NLX_SEARCH elevated through a Task Scheduler task ("Run with highest privileges", at logon). No UAC prompt each time.
2. Sign the exe and use `uiAccess=true`, installed under `Program Files`. Complex.

## What to avoid
- IFEO redirects of `SearchApp.exe`/`StartMenuExperienceHost.exe`: fragile and break on updates.
- Killing system processes.
- Slow work inside the hook callback. Windows silently drops slow hooks.
