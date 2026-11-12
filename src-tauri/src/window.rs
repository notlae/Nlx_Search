use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
use tauri::{AppHandle, Emitter, Manager};

/// While true the window does not hide on blur (onboarding, dialogs, rename).
pub static STICKY: AtomicBool = AtomicBool::new(false);
static SHOWN_AT_MS: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        SHOWN_AT_MS.store(now_ms(), SeqCst);
        let _ = w.center();
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit("nlx://shown", ());
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// Called by the hook (Win key etc.).
pub fn toggle(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    let visible = w.is_visible().unwrap_or(false);
    let focused = w.is_focused().unwrap_or(false);
    let _ = app.emit("nlx://triggered", ()); // lets the onboarding "test" step react
    if STICKY.load(SeqCst) {
        show(app);
        return;
    }
    if visible && focused {
        hide(app);
    } else {
        show(app);
    }
}

pub fn on_blur(app: &AppHandle) {
    if STICKY.load(SeqCst) {
        return;
    }
    // ignore the transient blur that can happen right after showing
    if now_ms().saturating_sub(SHOWN_AT_MS.load(SeqCst)) < 400 {
        return;
    }
    hide(app);
}
