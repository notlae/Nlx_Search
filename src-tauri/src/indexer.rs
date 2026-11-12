//! Background indexer: walks fixed drives + collects apps into SQLite.
use crate::{apps, db, settings, win};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::time::{Duration, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use walkdir::{DirEntry, WalkDir};

static REINDEX: AtomicBool = AtomicBool::new(false);
static SCANNING: AtomicBool = AtomicBool::new(false);
static CANCEL_SCAN: AtomicBool = AtomicBool::new(false);

pub fn request_reindex() {
    REINDEX.store(true, SeqCst);
}

pub fn cancel_scan() {
    CANCEL_SCAN.store(true, SeqCst);
}

pub fn is_scanning() -> bool {
    SCANNING.load(SeqCst)
}

struct Row {
    name: String,
    path: String,
    parent: String,
    kind: i64,
    size: Option<i64>,
    mtime: Option<i64>,
}

pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("nlx-indexer".into())
        .spawn(move || {
            // first pass shortly after startup (kept off the critical path)
            let mut wait_secs = 3u64;
            loop {
                for _ in 0..wait_secs {
                    if REINDEX.swap(false, SeqCst) {
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
                let s = settings::load();
                if s.first_run_done && s.index_enabled {
                    if let Err(e) = run_scan(&app, &s) {
                        eprintln!("NLX_SEARCH: index scan failed: {e}");
                    }
                }
                wait_secs = 3600; // rescan hourly, or immediately when requested
            }
        })
        .ok();
}

fn skip(e: &DirEntry, excluded: &[String]) -> bool {
    if !e.file_type().is_dir() {
        return false;
    }
    let name = e.file_name().to_string_lossy().to_lowercase();
    if matches!(
        name.as_str(),
        "$recycle.bin" | "system volume information" | "node_modules" | ".git" | "appdata"
            | "windowsapps" | "$winreagent" | "recovery" | "__pycache__" | "$windows.~bt"
    ) {
        return true;
    }
    if e.depth() == 1
        && matches!(name.as_str(), "windows" | "program files" | "program files (x86)" | "programdata")
    {
        return true;
    }
    if !excluded.is_empty() {
        let p = e.path().to_string_lossy().trim_end_matches(['\\', '/']).to_lowercase();
        if excluded.iter().any(|raw| {
            let x = raw.trim_end_matches(['\\', '/']);
            p == x || p.strip_prefix(x).is_some_and(|tail| tail.starts_with('\\') || tail.starts_with('/'))
        }) {
            return true;
        }
    }
    false
}

fn junk_ext(ext: &str) -> bool {
    matches!(
        ext,
        "dll" | "sys" | "tmp" | "dat" | "pyc" | "class" | "map" | "lib" | "obj" | "pdb" | "etl"
            | "mui" | "cat" | "manifest" | "ldb" | "lock" | "cab" | "msi" | "pf"
    )
}

fn flush(conn: &mut Connection, batch: &mut Vec<Row>, gen: i64) -> rusqlite::Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    {
        let mut st = tx.prepare_cached(
            "INSERT INTO items(name,name_lc,path,parent,kind,size,mtime,gen)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(path) DO UPDATE SET
               name=excluded.name, name_lc=excluded.name_lc, parent=excluded.parent,
               kind=excluded.kind, size=excluded.size, mtime=excluded.mtime, gen=excluded.gen",
        )?;
        for r in batch.iter() {
            st.execute(params![r.name, r.name.to_lowercase(), r.path, r.parent, r.kind, r.size, r.mtime, gen])?;
        }
    }
    tx.commit()?;
    batch.clear();
    Ok(())
}

fn count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0)).unwrap_or(0)
}

fn emit_status(app: &AppHandle, state: &str, n: i64) {
    let _ = app.emit("nlx://index-status", serde_json::json!({ "state": state, "count": n }));
}

pub fn run_scan(app: &AppHandle, s: &settings::Settings) -> rusqlite::Result<()> {
    if SCANNING.swap(true, SeqCst) { return Ok(()); }
    CANCEL_SCAN.store(false, SeqCst);
    let result = scan_inner(app, s);
    SCANNING.store(false, SeqCst);
    result
}

fn scan_inner(app: &AppHandle, s: &settings::Settings) -> rusqlite::Result<()> {
    let mut conn = db::open()?;
    // generation id: strictly increasing per scan
    let gen: i64 = conn
        .query_row("SELECT COALESCE(MAX(gen),0)+1 FROM items", [], |r| r.get(0))
        .unwrap_or(1);
    let excluded: Vec<String> = s.excluded_paths.iter().map(|p| p.to_lowercase()).collect();
    let mut batch: Vec<Row> = Vec::with_capacity(5000);
    emit_status(app, "scanning", count(&conn));

    // apps first so they are searchable immediately
    for a in apps::collect() {
        batch.push(Row { name: a.name, path: a.path, parent: a.parent, kind: 2, size: None, mtime: None });
    }
    flush(&mut conn, &mut batch, gen)?;
    emit_status(app, "scanning", count(&conn));

    let mut seen_since_emit = 0u32;
    for root in win::fixed_drives() {
        for entry in WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !skip(e, &excluded))
        {
            if CANCEL_SCAN.load(SeqCst) {
                emit_status(app, "idle", count(&conn));
                return Ok(()); // Never prune records after an incomplete/cancelled scan.
            }
            let Ok(e) = entry else { continue };
            if e.depth() == 0 {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            let is_dir = e.file_type().is_dir();
            let mut kind = 1i64;
            if !is_dir {
                let ext = Path::new(&name)
                    .extension()
                    .map(|x| x.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if junk_ext(&ext) {
                    continue;
                }
                kind = db::kind_from_ext(&ext);
            }
            let (size, mtime) = match e.metadata() {
                Ok(m) => (
                    if is_dir { None } else { Some(m.len() as i64) },
                    m.modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64),
                ),
                Err(_) => (None, None),
            };
            batch.push(Row {
                name,
                path: e.path().to_string_lossy().to_string(),
                parent: e.path().parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                kind,
                size,
                mtime,
            });
            if batch.len() >= 5000 {
                flush(&mut conn, &mut batch, gen)?;
                seen_since_emit += 1;
                if seen_since_emit >= 4 {
                    seen_since_emit = 0;
                    emit_status(app, "scanning", count(&conn));
                }
            }
        }
    }
    flush(&mut conn, &mut batch, gen)?;
    // anything not seen in this pass no longer exists
    conn.execute("DELETE FROM items WHERE gen < ?1", params![gen])?;
    emit_status(app, "idle", count(&conn));
    Ok(())
}

pub fn status(conn: &Connection) -> serde_json::Value {
    serde_json::json!({
        "state": if is_scanning() { "scanning" } else { "idle" },
        "count": count(conn)
    })
}
