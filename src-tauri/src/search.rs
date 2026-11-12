use crate::{db, state::AppState};
use rusqlite::{params_from_iter, Connection, Row};
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub parent: String,
    pub kind: String,
    pub size: Option<i64>,
    pub mtime: Option<i64>,
    pub fav: bool,
    pub score: i64,
    #[serde(skip)]
    pub kind_code: i64,
    #[serde(skip)]
    pub launches: i64,
    #[serde(skip)]
    pub last_used: i64,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub query_id: u64,
    pub best: Option<Item>,
    pub apps: Vec<Item>,
    pub folders: Vec<Item>,
    pub files: Vec<Item>,
    pub images: Vec<Item>,
    pub recent: Vec<Item>,
}

const COLS: &str = "i.id,i.name,i.path,i.parent,i.kind,i.size,i.mtime,
    COALESCE(u.launches,0),COALESCE(u.last_used,0),
    EXISTS(SELECT 1 FROM favorites f WHERE f.path=i.path)";

fn map_row(r: &Row) -> rusqlite::Result<Item> {
    let kind_code: i64 = r.get(4)?;
    Ok(Item {
        id: r.get(0)?,
        name: r.get(1)?,
        path: r.get(2)?,
        parent: r.get(3)?,
        kind: db::kind_name(kind_code).to_string(),
        size: r.get(5)?,
        mtime: r.get(6)?,
        launches: r.get(7)?,
        last_used: r.get(8)?,
        fav: r.get::<_, i64>(9)? != 0,
        score: 0,
        kind_code,
    })
}

fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn filter_clause(filter: &str) -> &'static str {
    match filter {
        "apps" => " AND i.kind = 2",
        "files" => " AND i.kind IN (0,3,4,5)",
        "folders" => " AND i.kind = 1",
        "images" => " AND i.kind = 3",
        "videos" => " AND i.kind = 4",
        "audio" => " AND i.kind = 5",
        _ => "",
    }
}

fn score(it: &Item, tokens: &[String], now: i64) -> i64 {
    let name = it.name.to_lowercase();
    let first = tokens.first().map(|s| s.as_str()).unwrap_or("");
    let mut s: i64 = 0;
    if name == first {
        s += 1000;
    } else if name.starts_with(first) {
        s += 600;
    } else if name.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(first)) {
        s += 300;
    } else {
        s += 100;
    }
    if it.kind_code == 2 {
        s += 150;
    }
    if it.fav {
        s += 80;
    }
    s += it.launches.min(50) * 8;
    if it.last_used > 0 {
        let days = (now - it.last_used) / 86_400;
        if days < 30 {
            s += (30 - days) * 5;
        }
    }
    s -= (name.chars().count() as i64) / 2;
    s -= (it.path.matches('\\').count() as i64) * 3;
    s
}

pub fn run(conn: &Connection, query: &str, filter: &str, query_id: u64) -> rusqlite::Result<SearchResponse> {
    let mut resp = SearchResponse { query_id, ..Default::default() };
    let tokens: Vec<String> = query.split_whitespace().take(4).map(|t| t.to_lowercase()).collect();

    if tokens.is_empty() {
        let sql = format!(
            "SELECT {COLS} FROM usage u JOIN items i ON i.path=u.path
             WHERE 1=1{} ORDER BY u.last_used DESC LIMIT 10",
            filter_clause(filter)
        );
        let mut st = conn.prepare(&sql)?;
        resp.recent = st.query_map([], map_row)?.filter_map(|r| r.ok()).collect();
        return Ok(resp);
    }

    let mut conds = Vec::new();
    let mut params: Vec<String> = Vec::new();
    for t in &tokens {
        conds.push("i.name_lc LIKE ? ESCAPE '\\'");
        params.push(format!("%{}%", like_escape(t)));
    }
    params.push(tokens[0].clone()); // exact match
    params.push(format!("{}%", like_escape(&tokens[0]))); // prefix match

    let sql = format!(
        "SELECT {COLS} FROM items i LEFT JOIN usage u ON u.path=i.path
         WHERE {}{}
         ORDER BY (i.name_lc = ?) DESC, (i.name_lc LIKE ? ESCAPE '\\') DESC,
                  (i.kind = 2) DESC, length(i.name_lc) ASC
         LIMIT 300",
        conds.join(" AND "),
        filter_clause(filter)
    );
    let mut st = conn.prepare(&sql)?;
    let mut items: Vec<Item> = st
        .query_map(params_from_iter(params.iter()), map_row)?
        .filter_map(|r| r.ok())
        .collect();

    let now = db::now_secs();
    for it in items.iter_mut() {
        it.score = score(it, &tokens, now);
    }
    items.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));

    if !items.is_empty() {
        resp.best = Some(items.remove(0));
    }
    for it in items {
        match it.kind_code {
            2 if resp.apps.len() < 5 => resp.apps.push(it),
            1 if resp.folders.len() < 6 => resp.folders.push(it),
            3 if resp.images.len() < 6 => resp.images.push(it),
            0 | 4 | 5 if resp.files.len() < 8 => resp.files.push(it),
            _ => {}
        }
    }
    Ok(resp)
}

pub fn favorites(conn: &Connection) -> rusqlite::Result<Vec<Item>> {
    let sql = format!("SELECT {COLS} FROM favorites fv JOIN items i ON i.path=fv.path LEFT JOIN usage u ON u.path=i.path ORDER BY i.name_lc LIMIT 100");
    let mut st = conn.prepare(&sql)?;
    let v = st.query_map([], map_row)?.filter_map(|r| r.ok()).collect();
    Ok(v)
}

#[tauri::command]
pub fn search(state: State<AppState>, query: String, filter: String, query_id: u64) -> Result<SearchResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    run(&db, &query, &filter, query_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_favorites(state: State<AppState>) -> Result<Vec<Item>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    favorites(&db).map_err(|e| e.to_string())
}
