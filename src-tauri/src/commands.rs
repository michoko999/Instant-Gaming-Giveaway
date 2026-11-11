//! Commandes appelées par l'interface.

use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::{Local, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tokio::sync::mpsc::unbounded_channel;

use crate::automation::{self, ScriptMode, Visibility};
use crate::checker::{self, giveaway_url};
use crate::model::*;
use crate::session::{self, Control};
use crate::sources::{self, SourceResult};
use crate::state::AppState;
use crate::store::csv_field;
use crate::update::{self, UpdateInfo};

type CmdResult<T> = Result<T, String>;

const MAX_IMPORT_BYTES: u64 = 5 * 1024 * 1024;
const FALLBACK_GIVEAWAY: &str = "INSTANTGAMING";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    settings: Settings,
    counts: Counts,
    stats: Stats,
    meta: Meta,
    version: String,
    data_dir: String,
    portable: bool,
    session_running: bool,
    busy: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    added: usize,
    marked_ended: usize,
    total: usize,
    sources: Vec<SourceResult>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CheckProgress {
    done: usize,
    total: usize,
    active: usize,
    ended: usize,
    unknown: usize,
    code: String,
    status: Status,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    total: usize,
    active: usize,
    ended: usize,
    unknown: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    found: usize,
    added: usize,
}

/// Réserve l'indicateur « occupé » le temps d'une synchronisation ou d'une vérification.
struct BusyGuard<'a>(&'a AppState);

impl<'a> BusyGuard<'a> {
    fn acquire(state: &'a AppState) -> CmdResult<BusyGuard<'a>> {
        state
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| BusyGuard(state))
            .map_err(|_| "busy".to_string())
    }
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::SeqCst);
    }
}

fn lock_err<T>(_: T) -> String {
    "internal-lock".into()
}

#[tauri::command]
pub fn get_overview(state: State<'_, AppState>) -> CmdResult<Overview> {
    let store = state.store.lock().map_err(lock_err)?;
    let now = Utc::now();
    Ok(Overview {
        settings: store.settings.clone(),
        counts: store.counts(now),
        stats: store.stats(now),
        meta: store.meta.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state.paths.data.display().to_string(),
        portable: state.paths.portable,
        session_running: state.session_running(),
        busy: state.busy.load(Ordering::SeqCst),
    })
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<Settings> {
    let mut store = state.store.lock().map_err(lock_err)?;
    let mut next = settings;
    // Les sources se gèrent avec add_source / remove_source ; on ne garde ici que l'état activé.
    let previous = store.settings.sources.clone();
    next.sources = previous
        .into_iter()
        .map(|mut s| {
            if let Some(updated) = next.sources.iter().find(|n| n.id == s.id) {
                s.enabled = updated.enabled;
            }
            s
        })
        .collect();
    next.sanitize();
    store.settings = next;
    store.save_settings().map_err(|e| e.to_string())?;
    Ok(store.settings.clone())
}

#[tauri::command]
pub fn add_source(state: State<'_, AppState>, name: String, url: String) -> CmdResult<Settings> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| "invalid-url".to_string())?;
    if parsed.scheme() != "https" {
        return Err("https-only".into());
    }
    let mut store = state.store.lock().map_err(lock_err)?;
    if store
        .settings
        .sources
        .iter()
        .any(|s| s.url == parsed.as_str())
    {
        return Err("duplicate-source".into());
    }
    let label = name.trim();
    store.settings.sources.push(Source {
        id: format!("custom-{}", automation::new_token()),
        name: if label.is_empty() {
            parsed.host_str().unwrap_or("source").to_string()
        } else {
            label.chars().take(60).collect()
        },
        url: parsed.to_string(),
        kind: sources::guess_kind(parsed.as_str()),
        enabled: true,
        builtin: false,
    });
    store.save_settings().map_err(|e| e.to_string())?;
    Ok(store.settings.clone())
}

#[tauri::command]
pub fn remove_source(state: State<'_, AppState>, id: String) -> CmdResult<Settings> {
    let mut store = state.store.lock().map_err(lock_err)?;
    store.settings.sources.retain(|s| s.builtin || s.id != id);
    store.save_settings().map_err(|e| e.to_string())?;
    Ok(store.settings.clone())
}

#[tauri::command]
pub async fn sync_lists(app: AppHandle) -> CmdResult<SyncReport> {
    let state = app.state::<AppState>();
    let _busy = BusyGuard::acquire(&state)?;
    let sources_list = state
        .store
        .lock()
        .map_err(lock_err)?
        .settings
        .sources
        .clone();
    let fetched = sources::fetch_all(&state.client, &sources_list).await;
    let mut store = state.store.lock().map_err(lock_err)?;
    let now = Utc::now();
    let added = store.merge_codes(&fetched.codes, now);
    let marked_ended = store.mark_ended(&fetched.dead, now);
    if fetched.results.iter().any(|r| r.ok) {
        store.meta.last_sync = Some(now);
    }
    store.save_giveaways().map_err(|e| e.to_string())?;
    store.save_meta().map_err(|e| e.to_string())?;
    Ok(SyncReport {
        added,
        marked_ended,
        total: store.giveaways.len(),
        sources: fetched.results,
    })
}

/// `scope` : "all" (actifs et inconnus) ou "unknown" (seulement ceux jamais vérifiés).
#[tauri::command]
pub async fn check_giveaways(app: AppHandle, scope: String) -> CmdResult<CheckReport> {
    let state = app.state::<AppState>();
    let _busy = BusyGuard::acquire(&state)?;
    let (lang, codes) = {
        let store = state.store.lock().map_err(lock_err)?;
        let codes: Vec<String> = store
            .giveaways
            .iter()
            .filter(|g| !g.ignored)
            .filter(|g| match scope.as_str() {
                "unknown" => g.status == Status::Unknown,
                _ => g.status != Status::Ended,
            })
            .map(|g| g.code.clone())
            .collect();
        (store.settings.language.clone(), codes)
    };
    let total = codes.len();
    let mut progress = CheckProgress {
        done: 0,
        total,
        active: 0,
        ended: 0,
        unknown: 0,
        code: String::new(),
        status: Status::Unknown,
    };
    checker::check_all(&state.client, &lang, codes, |code, status| {
        progress.done += 1;
        match status {
            Status::Active => progress.active += 1,
            Status::Ended => progress.ended += 1,
            Status::Unknown => progress.unknown += 1,
        }
        progress.code = code.to_string();
        progress.status = status;
        if let Ok(mut store) = state.store.lock() {
            // Une page injoignable (réseau, site lent) ne doit pas faire oublier un état déjà connu.
            let known = store
                .find_mut(code)
                .map(|g| g.status != Status::Unknown)
                .unwrap_or(false);
            if status != Status::Unknown || !known {
                store.set_status(code, status, Utc::now());
            }
        }
        let _ = app.emit("check-progress", progress.clone());
    })
    .await;
    let mut store = state.store.lock().map_err(lock_err)?;
    store.meta.last_check = Some(Utc::now());
    store.save_giveaways().map_err(|e| e.to_string())?;
    store.save_meta().map_err(|e| e.to_string())?;
    Ok(CheckReport {
        total,
        active: progress.active,
        ended: progress.ended,
        unknown: progress.unknown,
    })
}

#[tauri::command]
pub fn list_giveaways(state: State<'_, AppState>) -> CmdResult<Vec<Giveaway>> {
    Ok(state.store.lock().map_err(lock_err)?.giveaways.clone())
}

#[tauri::command]
pub fn set_ignored(state: State<'_, AppState>, code: String, ignored: bool) -> CmdResult<()> {
    let mut store = state.store.lock().map_err(lock_err)?;
    if let Some(g) = store.find_mut(&code) {
        g.ignored = ignored;
    }
    store.save_giveaways().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_list(app: AppHandle) -> CmdResult<Option<ImportReport>> {
    let picker = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .add_filter("CSV / TXT / JSON", &["csv", "txt", "json"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(file) = picked else { return Ok(None) };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if size > MAX_IMPORT_BYTES {
        return Err("file-too-large".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|_| "file-unreadable".to_string())?;
    let codes = match sources::parse_alive_json(&text) {
        Ok((alive, _)) => alive,
        Err(_) => sources::parse_list(&text),
    };
    let state = app.state::<AppState>();
    let mut store = state.store.lock().map_err(lock_err)?;
    let pairs: Vec<(String, String)> = codes
        .iter()
        .map(|c| (c.clone(), "import".to_string()))
        .collect();
    let added = store.merge_codes(&pairs, Utc::now());
    store.save_giveaways().map_err(|e| e.to_string())?;
    Ok(Some(ImportReport {
        found: codes.len(),
        added,
    }))
}

#[tauri::command]
pub fn get_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> CmdResult<Vec<HistoryEntry>> {
    let store = state.store.lock().map_err(lock_err)?;
    let limit = limit.unwrap_or(500);
    Ok(store.history.iter().rev().take(limit).cloned().collect())
}

#[tauri::command]
pub async fn export_history(app: AppHandle) -> CmdResult<Option<String>> {
    let picker = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!(
                "giveaway-ig-historique-{}.csv",
                Local::now().format("%Y-%m-%d")
            ))
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(file) = picked else { return Ok(None) };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let csv = {
        let store = state.store.lock().map_err(lock_err)?;
        let mut out = String::from("date,code,outcome,mode,message\n");
        for h in &store.history {
            let mode = match h.mode {
                SessionMode::Auto => "auto",
                SessionMode::Manual => "manual",
            };
            let outcome = serde_json::to_value(h.outcome)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            out.push_str(&format!(
                "{},{},{},{},{}\n",
                h.at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S"),
                csv_field(&h.code),
                outcome,
                mode,
                csv_field(h.message.as_deref().unwrap_or(""))
            ));
        }
        out
    };
    std::fs::write(&path, csv).map_err(|e| e.to_string())?;
    Ok(Some(path.display().to_string()))
}

// Les commandes qui créent ou ferment des fenêtres sont `async` : sous Windows, une commande
// synchrone s'exécute sur le fil principal et bloque WebView2 (fenêtre blanche, application figée).
#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    mode: Option<SessionMode>,
    codes: Option<Vec<String>>,
) -> CmdResult<usize> {
    let state = app.state::<AppState>();
    let (mode, queue, limited) = {
        let store = state.store.lock().map_err(lock_err)?;
        let mode = mode.unwrap_or(store.settings.mode);
        let mut queue: Vec<String> = match codes {
            Some(list) => list
                .into_iter()
                .filter_map(|c| sources::extract_code(&c))
                .collect(),
            None => store.session_queue(Utc::now()),
        };
        // Limite par session : uniquement en mode automatique.
        let limit = store.settings.session_limit as usize;
        let mut limited = 0;
        if mode == SessionMode::Auto && limit > 0 && queue.len() > limit {
            queue.truncate(limit);
            limited = limit;
        }
        (mode, queue, limited)
    };
    session::start(&app, mode, queue, limited)
}

#[tauri::command]
pub fn session_control(state: State<'_, AppState>, action: String) -> CmdResult<()> {
    let control = Control::parse(&action).ok_or_else(|| "unknown-action".to_string())?;
    if state.send_control(control) {
        Ok(())
    } else {
        Err("no-session".into())
    }
}

#[tauri::command]
pub async fn open_login(app: AppHandle) -> CmdResult<()> {
    let state = app.state::<AppState>();
    if state.session_running() {
        return Err("session-running".into());
    }
    let lang = state
        .store
        .lock()
        .map_err(lock_err)?
        .settings
        .language
        .clone();
    state.set_script(None);
    automation::open_ig(
        &app,
        &format!("https://www.instant-gaming.com/{lang}/"),
        Visibility::Focus,
    )?;
    Ok(())
}

/// Ouvre une page de concours dans la fenêtre Instant Gaming et vérifie si le compte est connecté.
#[tauri::command]
pub async fn verify_login(app: AppHandle) -> CmdResult<bool> {
    let state = app.state::<AppState>();
    if state.session_running() {
        return Err("session-running".into());
    }
    let (lang, code) = {
        let store = state.store.lock().map_err(lock_err)?;
        let code = store
            .giveaways
            .iter()
            .find(|g| g.status == Status::Active)
            .map(|g| g.code.clone())
            .unwrap_or_else(|| FALLBACK_GIVEAWAY.to_string());
        (store.settings.language.clone(), code)
    };
    let token = automation::new_token();
    let (tx, mut rx) = unbounded_channel();
    if let Ok(mut reports) = state.reports.lock() {
        *reports = Some(tx);
    }
    state.set_script(Some(automation::build_script(
        ScriptMode::Probe,
        &code,
        &token,
    )));
    let opened = automation::open_ig(&app, &giveaway_url(&lang, &code), Visibility::Keep);
    let result = match opened {
        Err(e) => Err(e),
        Ok(_) => {
            // Attend la réponse de la page, en abandonnant si l'utilisateur ferme la fenêtre.
            let mut outcome: CmdResult<bool> = Err("no-response".into());
            for _ in 0..60 {
                match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                    Ok(Some(report)) if report.token == token => {
                        outcome = Ok(report.kind == "connected");
                        break;
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(_) => {
                        if automation::ig_window(&app).is_none() {
                            outcome = Err("ig-closed".into());
                            break;
                        }
                    }
                }
            }
            outcome
        }
    };
    state.set_script(None);
    if let Ok(mut reports) = state.reports.lock() {
        *reports = None;
    }
    if let Ok(connected) = result {
        let mut store = state.store.lock().map_err(lock_err)?;
        store.settings.account_connected = connected;
        let _ = store.save_settings();
        if connected {
            automation::close_ig(&app);
        } else if let Some(window) = automation::ig_window(&app) {
            automation::reveal(&window);
        }
    }
    result
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> CmdResult<()> {
    let state = app.state::<AppState>();
    if state.session_running() {
        return Err("session-running".into());
    }
    automation::close_ig(&app);
    let cleared = app
        .get_webview_window("main")
        .map(|w| w.clear_all_browsing_data().is_ok())
        .unwrap_or(false);
    if !cleared {
        state.paths.request_webview_reset();
    }
    let mut store = state.store.lock().map_err(lock_err)?;
    store.settings.account_connected = false;
    store.save_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_giveaway(app: AppHandle, code: String) -> CmdResult<()> {
    let state = app.state::<AppState>();
    if state.session_running() {
        return Err("session-running".into());
    }
    let code = sources::extract_code(&code).ok_or_else(|| "invalid-code".to_string())?;
    let lang = state
        .store
        .lock()
        .map_err(lock_err)?
        .settings
        .language
        .clone();
    state.set_script(None);
    automation::open_ig(&app, &giveaway_url(&lang, &code), Visibility::Focus)?;
    Ok(())
}

#[tauri::command]
pub async fn show_ig_window(app: AppHandle) -> bool {
    match automation::ig_window(&app) {
        Some(window) => {
            automation::reveal(&window);
            true
        }
        None => false,
    }
}

/// Ouvre un lien dans le navigateur par défaut (sites connus uniquement).
#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> CmdResult<()> {
    let parsed = url::Url::parse(&url).map_err(|_| "invalid-url".to_string())?;
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let allowed = [
        "github.com",
        "www.instant-gaming.com",
        "instant-gaming.com",
        "signpath.org",
        "www.microsoft.com",
    ];
    if parsed.scheme() != "https" || !allowed.contains(&host.as_str()) {
        return Err("url-not-allowed".into());
    }
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_data_dir(app: AppHandle) -> CmdResult<()> {
    let path = app.state::<AppState>().paths.data.display().to_string();
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> CmdResult<UpdateInfo> {
    let state = app.state::<AppState>();
    update::check(&state.client).await
}
