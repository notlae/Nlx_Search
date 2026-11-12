#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod apps;
mod db;
mod hook;
mod indexer;
mod search;
mod settings;
mod startup;
mod state;
mod win;
mod window;

use std::sync::{mpsc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

/// Turns takeover on/off. Unlike before, turning it ON now also starts the
/// hook thread immediately if it isn't running yet, instead of only saving
/// a setting that would require an app restart to take effect.
fn set_takeover(app: &tauri::AppHandle, enabled: bool) {
    hook::TAKEOVER.store(enabled, std::sync::atomic::Ordering::SeqCst);

    if enabled {
        let (tx, rx) = mpsc::channel::<()>();
        hook::start(tx); // no-op if already running, thanks to HOOK_STARTED
        let h = app.clone();
        std::thread::spawn(move || {
            for _ in rx {
                window::toggle(&h);
            }
        });
    }

    let mut s = settings::load();
    s.hook_enabled = enabled;
    let _ = settings::save(&s);
}

fn main() {
    let no_hook = std::env::args().any(|a| a == "--no-hook");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app);
        }))
        .invoke_handler(tauri::generate_handler![
            search::search,
            search::list_favorites,
            actions::open_item,
            actions::open_location,
            actions::run_as_admin,
            actions::show_properties,
            actions::delete_item,
            actions::rename_item,
            actions::toggle_favorite,
            actions::get_thumb,
            actions::drive_info,
            actions::index_status,
            actions::reindex,
            actions::hide_window,
            actions::set_sticky,
            actions::get_settings,
            actions::save_settings,
            actions::uninstall_cleanup,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let cfg = settings::load();

            let conn = db::open()?;
            app.manage(state::AppState { db: Mutex::new(conn) });

            // ---- window: acrylic glass + hide on blur ----
            let w = app.get_webview_window("main").ok_or("main window missing")?;
            // Falls back to the CSS translucent panel if acrylic is unsupported.
            let _ = window_vibrancy::apply_acrylic(&w, Some((23, 25, 29, 150)));
            let h_blur = handle.clone();
            w.on_window_event(move |e| {
                if let WindowEvent::Focused(false) = e {
                    window::on_blur(&h_blur);
                }
            });

            // ---- keyboard hook ----
            hook::TAKEOVER.store(cfg.hook_enabled, std::sync::atomic::Ordering::SeqCst);
            if !no_hook && cfg.hook_enabled {
                let (tx, rx) = mpsc::channel::<()>();
                hook::start(tx);
                let h = handle.clone();
                std::thread::spawn(move || {
                    for _ in rx {
                        window::toggle(&h);
                    }
                });
            }

            // ---- tray ----
            let show = MenuItem::with_id(app, "show", "Open NLX_SEARCH", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Restore Windows Search (pause takeover)", true, None::<&str>)?;
            let resume = MenuItem::with_id(app, "resume", "Enable keyboard takeover", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &pause, &resume, &quit])?;
            let mut tray = TrayIconBuilder::new().tooltip("NLX_SEARCH").menu(&menu);
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.on_menu_event(|app, event| match event.id().as_ref() {
                "show" => window::show(app),
                "pause" => set_takeover(app, false),
                "resume" => set_takeover(app, true),
                "quit" => app.exit(0),
                _ => {}
            })
            .build(app)?;

            // ---- background indexer ----
            indexer::start(handle.clone());

            // first run (or debug build): show the window / onboarding
            if !cfg.first_run_done || cfg!(debug_assertions) {
                let h = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(900));
                    window::show(&h);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running NLX_SEARCH");
}