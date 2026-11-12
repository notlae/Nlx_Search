use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub first_run_done: bool,
    pub hook_enabled: bool,
    pub autostart: bool,
    pub index_enabled: bool,
    pub admin_mode: bool,
    pub excluded_paths: Vec<String>,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            first_run_done: false,
            hook_enabled: false,
            autostart: false,
            index_enabled: true,
            admin_mode: false,
            excluded_paths: vec![],
            theme: "aurora".into(),
        }
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("NLX_SEARCH")
}

fn file() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn load() -> Settings {
    let Ok(raw) = fs::read_to_string(file()) else { return Settings::default(); };
    let has_theme = serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| v.get("theme").cloned())
        .is_some();
    let mut settings = serde_json::from_str::<Settings>(&raw).unwrap_or_default();
    // Migration from pre-theme builds: disable the old always-on keyboard hook
    // and elevated startup mode so users explicitly opt in again.
    if !has_theme {
        crate::startup::cleanup_legacy_task();
        settings.hook_enabled = false;
        settings.admin_mode = false;
        let _ = save(&settings);
    }
    settings
}

pub fn save(s: &Settings) -> Result<(), String> {
    fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(file(), json).map_err(|e| e.to_string())
}
