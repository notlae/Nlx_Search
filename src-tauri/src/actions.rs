use crate::{db, settings, startup, state::AppState, win};
use base64::Engine;
use rusqlite::{params, OptionalExtension};
use std::os::windows::process::CommandExt;
use std::process::Command;
use tauri::{AppHandle, State};

fn lock<'a>(state: &'a State<AppState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.db.lock().map_err(|e| e.to_string())
}

/// Only paths that are in the index may be acted on (a compromised UI cannot target arbitrary files).
fn indexed_kind(state: &State<AppState>, path: &str) -> Result<i64, String> {
    let db = lock(state)?;
    db.query_row("SELECT kind FROM items WHERE path=?1", params![path], |r| r.get::<_, i64>(0))
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "That item is not in the index (try reindexing)".to_string())
}

#[tauri::command]
pub fn open_item(state: State<AppState>, path: String) -> Result<(), String> {
    indexed_kind(&state, &path)?;
    Command::new("explorer.exe").arg(&path).spawn().map_err(|e| e.to_string())?;
    let db = lock(&state)?;
    db.execute(
        "INSERT INTO usage(path,launches,last_used) VALUES(?1,1,?2)
         ON CONFLICT(path) DO UPDATE SET launches=launches+1, last_used=excluded.last_used",
        params![path, db::now_secs()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_location(state: State<AppState>, path: String) -> Result<(), String> {
    if indexed_kind(&state, &path)? == 2 && path.starts_with("shell:") {
        return Err("Store apps have no file location".into());
    }
    Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn run_as_admin(state: State<AppState>, path: String) -> Result<(), String> {
    if indexed_kind(&state, &path)? != 2 || path.starts_with("shell:") {
        return Err("Run as administrator works for desktop apps only".into());
    }
    let cmd = format!("Start-Process -FilePath '{}' -Verb RunAs", path.replace('\'', "''"));
    win::hidden("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &cmd])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_properties(state: State<AppState>, path: String) -> Result<(), String> {
    if indexed_kind(&state, &path)? == 2 && path.starts_with("shell:") {
        return Err("Store apps have no file properties".into());
    }
    win::show_properties(path);
    Ok(())
}

fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

#[tauri::command]
pub fn delete_item(state: State<AppState>, path: String) -> Result<(), String> {
    if indexed_kind(&state, &path)? == 2 {
        return Err("Apps cannot be deleted from here (use Windows Settings to uninstall)".into());
    }
    trash::delete(&path).map_err(|e| format!("Could not move to Recycle Bin: {e}"))?;
    let db = lock(&state)?;
    db.execute(
        "DELETE FROM items WHERE path=?1 OR path LIKE ?2 ESCAPE '\\'",
        params![path, format!("{}\\%", like_escape(&path))],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn rename_item(state: State<AppState>, path: String, new_name: String) -> Result<(), String> {
    if indexed_kind(&state, &path)? == 2 {
        return Err("Apps cannot be renamed".into());
    }
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() || new_name == "." || new_name == ".."
        || new_name.ends_with('.') || new_name.ends_with(' ') {
        return Err("Invalid name".into());
    }
    if new_name.chars().any(|c| "\\/:*?\"<>|".contains(c) || c.is_control()) {
        return Err("A name can't contain any of: \\ / : * ? \" < > |".into());
    }

    let old = std::path::PathBuf::from(&path);
    let parent = old.parent().ok_or("No parent folder")?;
    let target = parent.join(&new_name);
    if target.exists() { return Err("An item with that name already exists".into()); }
    let new_path = target.to_string_lossy().to_string();
    let old_prefix = format!("{}\\", path.trim_end_matches(['\\', '/']));
    let new_prefix = format!("{}\\", new_path.trim_end_matches(['\\', '/']));

    std::fs::rename(&old, &target).map_err(|e| e.to_string())?;
    let db_result = (|| -> Result<(), String> {
        let mut conn = lock(&state)?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE items SET name=?1,name_lc=?2,path=?3 WHERE path=?4",
            params![new_name, new_name.to_lowercase(), new_path, path],
        ).map_err(|e| e.to_string())?;
        // Update all descendants and their parent paths using literal prefixes.
        tx.execute(
            "UPDATE items SET path=?1 || substr(path, length(?4)+1),
             parent=CASE WHEN parent=?2 THEN ?3
                          WHEN substr(parent,1,length(?4))=?4 THEN ?1 || substr(parent,length(?4)+1)
                          ELSE parent END
             WHERE substr(path,1,length(?4))=?4",
            params![new_prefix, path, new_path, old_prefix],
        ).map_err(|e| e.to_string())?;
        for table in ["favorites", "usage"] {
            let sql = format!(
                "UPDATE {table} SET path=?1 WHERE path=?2"
            );
            tx.execute(&sql, params![new_path, path]).map_err(|e| e.to_string())?;
            let sql = format!(
                "UPDATE {table} SET path=?1 || substr(path,length(?2)+1) WHERE substr(path,1,length(?2))=?2"
            );
            tx.execute(&sql, params![new_prefix, old_prefix])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    })();
    if let Err(error) = db_result {
        return match std::fs::rename(&target, &old) {
            Ok(()) => Err(error),
            Err(rollback) => Err(format!("Database update failed ({error}); filesystem rollback also failed ({rollback}). Reindex this drive.")),
        };
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, path: String) -> Result<bool, String> {
    indexed_kind(&state, &path)?;
    let db = lock(&state)?;
    let removed = db
        .execute("DELETE FROM favorites WHERE path=?1", params![path])
        .map_err(|e| e.to_string())?;
    if removed > 0 {
        return Ok(false);
    }
    db.execute("INSERT INTO favorites(path) VALUES(?1)", params![path]).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn get_thumb(state: State<AppState>, path: String) -> Result<Option<String>, String> {
    if indexed_kind(&state, &path)? != 3 {
        return Ok(None);
    }
    let ext = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => return Ok(None),
    };
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() > 1_500_000 {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}

#[tauri::command]
pub fn drive_info() -> Vec<win::Drive> {
    win::drive_info()
}

#[tauri::command]
pub fn index_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let db = lock(&state)?;
    Ok(crate::indexer::status(&db))
}

#[tauri::command]
pub fn reindex() {
    crate::indexer::request_reindex();
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    crate::window::hide(&app);
}

#[tauri::command]
pub fn set_sticky(sticky: bool) {
    crate::window::STICKY.store(sticky, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
pub fn get_settings() -> settings::Settings {
    settings::load()
}

/// Saves settings and applies side effects (hook toggle, startup entries, indexing).
#[tauri::command]
pub fn save_settings(app: AppHandle, new: settings::Settings) -> Result<settings::Settings, String> {
    let old = settings::load();
    let mut s = new;
    // Elevated scheduled-task startup is intentionally disabled in this build.
    s.admin_mode = false;
    crate::hook::TAKEOVER.store(s.hook_enabled, std::sync::atomic::Ordering::SeqCst);

    // Turning the toggle ON from the Settings page must also actually start
    // the hook thread now, not just flip the atomic and hope for a restart.
    if s.hook_enabled {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        crate::hook::start(tx); // no-op if a hook is already running
        let h = app.clone();
        std::thread::spawn(move || {
            for _ in rx {
                crate::window::toggle(&h);
            }
        });
    }

    let startup_changed = old.autostart != s.autostart || old.admin_mode != s.admin_mode;
    let mut warning: Option<String> = None;
    if startup_changed {
        if let Err(e) = startup::apply(&s) {
            // roll the startup-related flags back, keep the rest
            s.autostart = old.autostart;
            s.admin_mode = false;
            warning = Some(e);
        }
    }
    settings::save(&s)?;
    if !s.index_enabled {
        crate::indexer::cancel_scan();
    } else if s.first_run_done && (!old.index_enabled || !old.first_run_done) {
        crate::indexer::request_reindex();
    }
    match warning {
        Some(w) => Err(format!("Saved, but startup was not changed: {w}")),
        None => Ok(s),
    }
}

/// Removes startup entries, turns the takeover off and quits.
#[tauri::command]
pub fn uninstall_cleanup(app: AppHandle) -> Result<(), String> {
    let mut s = settings::load();
    s.autostart = false;
    s.admin_mode = false;
    s.hook_enabled = false;
    startup::remove_all()?;
    crate::hook::TAKEOVER.store(false, std::sync::atomic::Ordering::SeqCst);
    settings::save(&s)?;
    app.exit(0);
    Ok(())
}