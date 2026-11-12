use rusqlite::Connection;
use std::time::Duration;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
  id      INTEGER PRIMARY KEY,
  name    TEXT NOT NULL,
  name_lc TEXT NOT NULL,
  path    TEXT NOT NULL UNIQUE,
  parent  TEXT NOT NULL,
  kind    INTEGER NOT NULL,   -- 0 file, 1 folder, 2 app, 3 image, 4 video, 5 audio
  size    INTEGER,
  mtime   INTEGER,
  gen     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_items_kind ON items(kind);
CREATE TABLE IF NOT EXISTS usage (
  path TEXT PRIMARY KEY, launches INTEGER NOT NULL DEFAULT 0, last_used INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS favorites (path TEXT PRIMARY KEY);
";

pub fn open() -> rusqlite::Result<Connection> {
    let dir = crate::settings::data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let conn = Connection::open(dir.join("index.db"))?;
    conn.busy_timeout(Duration::from_secs(10))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn kind_name(k: i64) -> &'static str {
    match k {
        1 => "folder",
        2 => "app",
        3 => "image",
        4 => "video",
        5 => "audio",
        _ => "file",
    }
}

pub fn kind_from_ext(ext: &str) -> i64 {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "heic" => 3,
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "wmv" | "flv" | "m4v" => 4,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => 5,
        _ => 0,
    }
}

pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
