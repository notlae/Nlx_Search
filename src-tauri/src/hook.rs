//! Global keyboard hook that takes over Win, Win+S, Win+Q (and Alt+Space as a fallback).
//!
//! SAFETY NOTES
//! * The hook lives inside this process. If the app exits or crashes, Windows is back to normal.
//! * The callback only looks at Win/Shift/Ctrl/Alt/S/Q/Space state. Nothing is logged or stored.
//! * The callback is tiny: atomics + a channel send + SendInput. No I/O.
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::SeqCst};
use std::sync::{mpsc::Sender, OnceLock};
use std::thread;

/// Take over Win / Win+S / Win+Q. Toggle at runtime (tray, settings).
pub static TAKEOVER: AtomicBool = AtomicBool::new(false);
/// Alt+Space always works as a fallback hotkey while the hook is installed.
pub static FALLBACK: AtomicBool = AtomicBool::new(true);
/// Set to true the first time `start()` actually installs the hook.
/// Prevents double-installing the hook (leaked hooks, double-firing) when
/// the user toggles takeover on more than once in a session.
pub static HOOK_STARTED: AtomicBool = AtomicBool::new(false);

static TX: OnceLock<Sender<()>> = OnceLock::new();
static WIN_DOWN: AtomicBool = AtomicBool::new(false);
static OTHER_KEY: AtomicBool = AtomicBool::new(false);
static CONSUMED: AtomicBool = AtomicBool::new(false);
static SHIFT: AtomicBool = AtomicBool::new(false);
static CTRL: AtomicBool = AtomicBool::new(false);
static ALT: AtomicBool = AtomicBool::new(false);
static SWALLOW_SPACE_UP: AtomicBool = AtomicBool::new(false);
static _WIN_VK: AtomicU32 = AtomicU32::new(0);

const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYDOWN: usize = 0x0100;
const WM_KEYUP: usize = 0x0101;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_SYSKEYUP: usize = 0x0105;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_S: u32 = 0x53;
const VK_Q: u32 = 0x51;
const VK_SPACE: u32 = 0x20;
const VK_DUMMY: u16 = 0xE8; // unassigned key, used as the "mask" key
const LLKHF_ALTDOWN: u32 = 0x20;
const LLKHF_INJECTED: u32 = 0x10;
const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_KEYUP: u32 = 2;

#[repr(C)]
struct KbdLlHookStruct {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
struct KeybdInput {
    vk: u16,
    scan: u16,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
struct MouseInput {
    dx: i32,
    dy: i32,
    data: u32,
    flags: u32,
    time: u32,
    extra: usize,
}

#[repr(C)]
union InputUnion {
    ki: std::mem::ManuallyDrop<KeybdInput>,
    mi: std::mem::ManuallyDrop<MouseInput>,
}

#[repr(C)]
struct Input {
    kind: u32,
    u: InputUnion,
}

type HookProc = unsafe extern "system" fn(i32, usize, isize) -> isize;

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(id: i32, f: Option<HookProc>, hmod: isize, tid: u32) -> isize;
    fn UnhookWindowsHookEx(h: isize) -> i32;
    fn CallNextHookEx(h: isize, code: i32, w: usize, l: isize) -> isize;
    fn GetMessageW(msg: *mut [u64; 6], hwnd: isize, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const [u64; 6]) -> i32;
    fn DispatchMessageW(msg: *const [u64; 6]) -> isize;
    fn SendInput(n: u32, inputs: *const Input, size: i32) -> u32;
    fn GetAsyncKeyState(vk: i32) -> i16;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> isize;
}

fn trigger() {
    if let Some(tx) = TX.get() {
        let _ = tx.send(());
    }
}

fn key(vk: u16, up: bool) -> Input {
    Input {
        kind: INPUT_KEYBOARD,
        u: InputUnion {
            ki: std::mem::ManuallyDrop::new(KeybdInput {
                vk,
                scan: 0,
                flags: if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                extra: 0,
            }),
        },
    }
}

/// Swallow the real Win key-up and inject: dummy-down, dummy-up, Win-up (in one batch).
/// Windows then sees Win+<other key>, so the Start menu does not open.
/// Returns true when the injection worked (caller swallows the real key-up).
fn release_with_mask(win_vk: u32) -> bool {
    let inputs = [key(VK_DUMMY, false), key(VK_DUMMY, true), key(win_vk as u16, true)];
    let sent = unsafe { SendInput(3, inputs.as_ptr(), std::mem::size_of::<Input>() as i32) };
    sent == 3
}

fn is_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16) & 0x8000 != 0 }
}

/// Returns true if the event must be swallowed.
fn handle(kb: &KbdLlHookStruct, msg: usize) -> bool {
    let vk = kb.vk_code;
    let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
    if !down && !up {
        return false;
    }

    match vk {
        0xA0 | 0xA1 | 0x10 => SHIFT.store(down, SeqCst),
        0xA2 | 0xA3 | 0x11 => CTRL.store(down, SeqCst),
        0xA4 | 0xA5 | 0x12 => ALT.store(down, SeqCst),
        _ => {}
    }

    // Fallback hotkey: Alt+Space
    if vk == VK_SPACE {
        if down
            && kb.flags & LLKHF_ALTDOWN != 0
            && FALLBACK.load(SeqCst)
            && !CTRL.load(SeqCst)
            && !SHIFT.load(SeqCst)
            && !WIN_DOWN.load(SeqCst)
        {
            SWALLOW_SPACE_UP.store(true, SeqCst);
            trigger();
            return true;
        }
        if up && SWALLOW_SPACE_UP.swap(false, SeqCst) {
            return true;
        }
    }

    if !TAKEOVER.load(SeqCst) {
        WIN_DOWN.store(false, SeqCst);
        return false;
    }

    let is_win = vk == VK_LWIN || vk == VK_RWIN;
    if is_win {
        if down {
            if !WIN_DOWN.swap(true, SeqCst) {
                OTHER_KEY.store(false, SeqCst);
                CONSUMED.store(false, SeqCst);
                _WIN_VK.store(vk, SeqCst);
                // resync modifiers in case we missed a key-up (e.g. elevated window had focus)
                SHIFT.store(is_down(0x10), SeqCst);
                CTRL.store(is_down(0x11), SeqCst);
                ALT.store(is_down(0x12), SeqCst);
            }
            return false;
        }
        if up && WIN_DOWN.swap(false, SeqCst) {
            let other = OTHER_KEY.load(SeqCst);
            let consumed = CONSUMED.load(SeqCst);
            if !other {
                trigger(); // Win pressed alone
                return release_with_mask(vk);
            }
            if consumed {
                return release_with_mask(vk); // we ate Win+S / Win+Q, so also cancel Start
            }
        }
        return false;
    }

    if down && WIN_DOWN.load(SeqCst) {
        OTHER_KEY.store(true, SeqCst);
        // Only plain Win+S / Win+Q. Win+Shift+S (Snipping) etc. stay untouched.
        if (vk == VK_S || vk == VK_Q)
            && !SHIFT.load(SeqCst)
            && !CTRL.load(SeqCst)
            && !ALT.load(SeqCst)
        {
            CONSUMED.store(true, SeqCst);
            trigger();
            return true;
        }
    }
    false
}

unsafe extern "system" fn proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 {
        let kb = &*(lparam as *const KbdLlHookStruct);
        if kb.flags & LLKHF_INJECTED == 0 && handle(kb, wparam) {
            return 1;
        }
    }
    CallNextHookEx(0, code, wparam, lparam)
}

/// Installs the hook on a dedicated thread with its own message loop.
/// Safe to call more than once: a no-op if the hook is already running.
pub fn start(tx: Sender<()>) {
    if HOOK_STARTED.swap(true, SeqCst) {
        return; // already installed, don't leak a second hook
    }
    let _ = TX.set(tx);
    let _ = thread::Builder::new().name("nlx-hook".into()).spawn(|| unsafe {
        let hmod = GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc), hmod, 0);
        if hook == 0 {
            eprintln!("NLX_SEARCH: could not install keyboard hook");
            return;
        }
        let mut msg = [0u64; 6];
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWindowsHookEx(hook);
    });
}