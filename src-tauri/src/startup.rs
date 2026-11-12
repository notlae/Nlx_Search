//! Transparent per-user startup integration.
//! This build uses only the current-user Run registry value. It does not
//! self-copy, create elevated scheduled tasks, or launch PowerShell scripts.
use crate::win;
use std::path::PathBuf;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "NLX_SEARCH";
const LEGACY_TASK: &str = "NLX_SEARCH";

fn run_key_add(exe: &PathBuf) -> Result<(), String> {
    let status = win::hidden("reg")
        .args(["add", RUN_KEY, "/v", VALUE, "/t", "REG_SZ", "/d"])
        .arg(format!("\"{}\"", exe.to_string_lossy()))
        .arg("/f")
        .status()
        .map_err(|e| format!("Could not start registry utility: {e}"))?;
    if status.success() { Ok(()) } else { Err("Could not write the current-user startup entry".into()) }
}

fn run_key_remove() -> Result<(), String> {
    let _status = win::hidden("reg")
        .args(["delete", RUN_KEY, "/v", VALUE, "/f"])
        .status()
        .map_err(|e| format!("Could not start registry utility: {e}"))?;
    // reg.exe returns non-zero when the value is already absent.
    Ok(())
}

pub fn cleanup_legacy_task() {
    // Clean up a task created by older NLX_SEARCH builds, if present.
    // No elevation or script execution is used.
    let _ = win::hidden("schtasks")
        .args(["/Delete", "/TN", LEGACY_TASK, "/F"])
        .status();
}

pub fn apply(s: &crate::settings::Settings) -> Result<(), String> {
    cleanup_legacy_task();
    if !s.autostart {
        return run_key_remove();
    }
    let exe = std::env::current_exe().map_err(|e| format!("Could not locate NLX_SEARCH executable: {e}"))?;
    run_key_add(&exe)
}

pub fn remove_all() -> Result<(), String> {
    run_key_remove()?;
    cleanup_legacy_task();
    Ok(())
}
