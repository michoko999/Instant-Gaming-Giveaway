// Pas de fenêtre de console en version publiée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod automation;
mod backup;
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
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            update::cleanup();
            let paths = AppPaths::resolve(app.handle());
            update::cleanup_downloads(&paths.data);
            paths.apply_pending_reset();
            // Nouvelle version : copie des données avant de les lire (et peut-être de les convertir).
            let version = env!("CARGO_PKG_VERSION");
            backup::before_upgrade(&paths.data, version);
            let mut store = Store::load(&paths.data);
            if store.meta.app_version.as_deref() != Some(version) {
                store.meta.app_version = Some(version.to_string());
                let _ = store.save_meta();
            }
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
            commands::list_backups,
            commands::restore_backup,
            commands::problem_report,
            commands::save_settings,
            commands::add_source,
            commands::remove_source,
            commands::sync_lists,
            commands::check_giveaways,
            commands::list_giveaways,
            commands::set_ignored,
            commands::import_list,
            commands::add_links,
            commands::get_history,
            commands::export_history,
            commands::start_session,
            commands::session_control,
            commands::open_login,
            commands::verify_login,
            commands::logout,
            commands::add_account,
            commands::switch_account,
            commands::remove_account,
            commands::ack_wins,
            commands::dismiss_month,
            commands::mark_giveaways_seen,
            commands::notify,
            commands::open_giveaway,
            commands::show_ig_window,
            commands::open_external,
            commands::open_data_dir,
            commands::check_update,
            commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Giveaway IG");
}
