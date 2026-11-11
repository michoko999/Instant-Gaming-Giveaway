// Pas de fenêtre de console en version publiée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod automation;
mod checker;
mod commands;
mod model;
mod session;
mod sources;
mod state;
mod store;
mod update;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::state::{AppPaths, AppState};
use crate::store::Store;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let paths = AppPaths::resolve(app.handle());
            paths.apply_pending_reset();
            let store = Store::load(&paths.data);
            let webview_dir = paths.webview.clone();
            app.manage(AppState::new(paths, store));

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Giveaway IG")
                .inner_size(1280.0, 860.0)
                .min_inner_size(1000.0, 700.0)
                .center()
                .theme(Some(tauri::Theme::Dark))
                .data_directory(webview_dir)
                .additional_browser_args(automation::BROWSER_ARGS)
                .build()?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Fermer la fenêtre principale ferme aussi la fenêtre Instant Gaming et quitte l'application.
            if window.label() == "main" {
                if let WindowEvent::Destroyed = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_overview,
            commands::save_settings,
            commands::add_source,
            commands::remove_source,
            commands::sync_lists,
            commands::check_giveaways,
            commands::list_giveaways,
            commands::set_ignored,
            commands::import_list,
            commands::get_history,
            commands::export_history,
            commands::start_session,
            commands::session_control,
            commands::open_login,
            commands::verify_login,
            commands::logout,
            commands::open_giveaway,
            commands::show_ig_window,
            commands::open_external,
            commands::open_data_dir,
            commands::check_update,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Giveaway IG");
}
