//! Installed applications: Start Menu shortcuts + Store (UWP) apps.
use crate::win;
use std::collections::HashSet;
use walkdir::WalkDir;

pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub parent: String,
}

pub fn collect() -> Vec<AppEntry> {
    let mut out: Vec<AppEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // 1) classic apps: Start Menu .lnk files
    let mut roots = Vec::new();
    if let Ok(p) = std::env::var("PROGRAMDATA") {
        roots.push(format!("{}\\Microsoft\\Windows\\Start Menu\\Programs", p));
    }
    if let Ok(p) = std::env::var("APPDATA") {
        roots.push(format!("{}\\Microsoft\\Windows\\Start Menu\\Programs", p));
    }
    for root in roots {
        for e in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            if !e.file_type().is_file() {
                continue;
            }
            let p = e.path();
            if p.extension().map(|x| x.to_string_lossy().to_lowercase()) != Some("lnk".into()) {
                continue;
            }
            let name = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let lc = name.to_lowercase();
            if name.is_empty() || lc.contains("uninstall") || !seen.insert(lc) {
                continue;
            }
            out.push(AppEntry {
                name,
                path: p.to_string_lossy().to_string(),
                parent: p.parent().map(|x| x.to_string_lossy().to_string()).unwrap_or_default(),
            });
        }
    }

    // 2) Store / UWP apps (and anything else the Start menu lists)
    let script = "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | ConvertTo-Json -Compress";
    if let Ok(o) = win::hidden("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
    {
        let text = String::from_utf8_lossy(&o.stdout).trim_start_matches('\u{feff}').to_string();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            let list: Vec<serde_json::Value> = match v {
                serde_json::Value::Array(a) => a,
                obj @ serde_json::Value::Object(_) => vec![obj],
                _ => vec![],
            };
            for item in list {
                let name = item.get("Name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let id = item.get("AppID").and_then(|x| x.as_str()).unwrap_or("").to_string();
                if name.is_empty() || id.is_empty() {
                    continue;
                }
                if !seen.insert(name.to_lowercase()) {
                    continue;
                }
                out.push(AppEntry {
                    name,
                    path: format!("shell:AppsFolder\\{}", id),
                    parent: "Installed app".to_string(),
                });
            }
        }
    }
    out
}
