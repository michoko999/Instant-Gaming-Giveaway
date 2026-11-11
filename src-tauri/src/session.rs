//! Déroulement d'une session de participation (automatique ou manuelle).

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, UserAttentionType};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::automation::{self, Report, ScriptMode};
use crate::checker::giveaway_url;
use crate::model::{Outcome, SessionMode, SessionSummary};
use crate::state::AppState;

/// Délai maximal d'attente d'une réponse de la page en mode automatique.
const AUTO_TIMEOUT: Duration = Duration::from_secs(45);
/// Pause après une participation détectée en mode manuel, avant de passer au suivant.
const MANUAL_AFTER_SUCCESS: Duration = Duration::from_millis(1200);
/// Mode prudent : temps minimum entre deux concours en mode automatique.
const SAFE_MIN_DELAY: f64 = 8.0;
/// Mode prudent : arrêt après ce nombre de vérifications humaines dans la session.
const SAFE_MAX_CAPTCHAS: usize = 2;
/// Arrêt après ce nombre d'erreurs techniques d'affilée (site injoignable, page inattendue...).
const MAX_CONSECUTIVE_ERRORS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Pause,
    Resume,
    Stop,
    Skip,
    Next,
    IgClosed,
}

impl Control {
    pub fn parse(action: &str) -> Option<Control> {
        match action {
            "pause" => Some(Control::Pause),
            "resume" => Some(Control::Resume),
            "stop" => Some(Control::Stop),
            "skip" => Some(Control::Skip),
            "next" => Some(Control::Next),
            _ => None,
        }
    }

    fn stops(self) -> bool {
        matches!(self, Control::Stop | Control::IgClosed)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// "running", "paused", "login" ou "finished".
    pub state: &'static str,
    pub mode: SessionMode,
    pub index: usize,
    pub total: usize,
    pub current: Option<String>,
    pub participated: usize,
    pub already: usize,
    pub opened: usize,
    pub ended: usize,
    pub errors: usize,
    pub skipped: usize,
    pub pending: usize,
    pub remaining_secs: u64,
    /// Secondes restantes avant le concours suivant (mode manuel).
    pub countdown: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub at: DateTime<Utc>,
    pub code: String,
    pub outcome: Outcome,
    pub message: Option<String>,
}

enum Wait {
    Report(Report),
    Control(Control),
    Timeout,
}

/// Lance une session en arrière-plan. Renvoie le nombre de concours à traiter.
/// `limited` vaut la limite appliquée à la file (0 si aucune).
pub fn start(app: &AppHandle, mode: SessionMode, codes: Vec<String>, limited: usize) -> Result<usize, String> {
    if codes.is_empty() {
        return Err("no-giveaways".into());
    }
    let state = app.state::<AppState>();
    let (ctl_tx, ctl_rx) = unbounded_channel();
    let (rep_tx, rep_rx) = unbounded_channel();
    {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if session.is_some() {
            return Err("session-running".into());
        }
        *session = Some(ctl_tx);
    }
    if let Ok(mut reports) = state.reports.lock() {
        *reports = Some(rep_tx);
    }
    let total = codes.len();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        run(handle, mode, codes, limited, ctl_rx, rep_rx).await;
    });
    Ok(total)
}

async fn wait_event(
    ctl: &mut UnboundedReceiver<Control>,
    rep: &mut UnboundedReceiver<Report>,
    token: &str,
    timeout: Duration,
) -> Wait {
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            control = ctl.recv() => return Wait::Control(control.unwrap_or(Control::Stop)),
            Some(report) = rep.recv() => {
                if report.token == token {
                    return Wait::Report(report);
                }
            }
            _ = &mut deadline => return Wait::Timeout,
        }
    }
}

/// Attend « Reprendre » (true) ou « Arrêter » (false).
async fn wait_resume(ctl: &mut UnboundedReceiver<Control>) -> bool {
    loop {
        match ctl.recv().await {
            Some(Control::Resume) => return true,
            Some(c) if c.stops() => return false,
            Some(_) => continue,
            None => return false,
        }
    }
}

fn emit_progress(app: &AppHandle, progress: &Progress) {
    let _ = app.emit("session-progress", progress);
}

fn estimate(mode: SessionMode, remaining: usize, delay: f64) -> u64 {
    let per_item = match mode {
        SessionMode::Auto => delay + 3.0,
        SessionMode::Manual => delay,
    };
    (remaining as f64 * per_item).round() as u64
}

/// Traduit le rapport de la page. `None` : le compte n'est pas connecté.
/// Le booléen indique un avertissement affiché par le site lui-même.
fn map_auto_report(report: &Report) -> Option<(Outcome, Option<String>, bool)> {
    Some(match report.kind.as_str() {
        "participated" => (Outcome::Participated, None, false),
        "already" => (Outcome::Already, None, false),
        "ended" => (Outcome::Ended, None, false),
        "captcha" => (Outcome::Captcha, None, false),
        "login" => return None,
        "nobutton" => (Outcome::Error, Some("button-not-found".into()), false),
        "timeout" => (Outcome::Error, Some("no-response".into()), false),
        "error" => (Outcome::Error, report.message.clone(), true),
        other => (Outcome::Error, Some(other.to_string()), false),
    })
}

async fn run(
    app: AppHandle,
    mode: SessionMode,
    codes: Vec<String>,
    limited: usize,
    mut ctl: UnboundedReceiver<Control>,
    mut rep: UnboundedReceiver<Report>,
) {
    let state = app.state::<AppState>();
    let (lang, delay, safe) = match state.store.lock() {
        Ok(store) => (store.settings.language.clone(), store.settings.seconds_per_giveaway, store.settings.safe_mode),
        Err(_) => ("fr".to_string(), 4.0, true),
    };
    let careful = safe && mode == SessionMode::Auto;
    // En mode prudent, on laisse toujours au moins quelques secondes entre deux concours.
    let delay = if careful { delay.max(SAFE_MIN_DELAY) } else { delay };
    let total = codes.len();
    let mut summary =
        SessionSummary { started_at: Some(Utc::now()), mode, total, limited, ..SessionSummary::default() };
    let mut captchas = 0usize;
    let mut consecutive_errors = 0usize;
    let mut progress = Progress {
        state: "running",
        mode,
        index: 0,
        total,
        current: None,
        participated: 0,
        already: 0,
        opened: 0,
        ended: 0,
        errors: 0,
        skipped: 0,
        pending: 0,
        remaining_secs: estimate(mode, total, delay),
        countdown: None,
    };
    let mut i = 0;
    let mut first = true;

    'session: while i < total {
        let code = codes[i].clone();
        progress.state = "running";
        progress.index = i;
        progress.current = Some(code.clone());
        progress.countdown = None;
        progress.remaining_secs = estimate(mode, total - i, delay);
        emit_progress(&app, &progress);

        let token = automation::new_token();
        let script_mode = match mode {
            SessionMode::Auto => ScriptMode::Auto,
            SessionMode::Manual => ScriptMode::Watch,
        };
        state.set_script(Some(automation::build_script(script_mode, &code, &token)));
        while rep.try_recv().is_ok() {}

        let mut site_warning = false;
        let (outcome, message) = match automation::open_ig(&app, &giveaway_url(&lang, &code), first) {
            Err(error) => (Outcome::Error, Some(error)),
            Ok(_) => {
                first = false;
                match mode {
                    SessionMode::Auto => match wait_event(&mut ctl, &mut rep, &token, AUTO_TIMEOUT).await {
                        Wait::Report(report) => match map_auto_report(&report) {
                            Some((outcome, message, warning)) => {
                                site_warning = warning;
                                (outcome, message)
                            }
                            None => {
                                // Compte déconnecté : on attend que l'utilisateur se connecte.
                                progress.state = "login";
                                emit_progress(&app, &progress);
                                if let Some(window) = automation::ig_window(&app) {
                                    let _ = window.set_focus();
                                }
                                if wait_resume(&mut ctl).await {
                                    continue 'session;
                                }
                                summary.stopped = true;
                                break 'session;
                            }
                        },
                        Wait::Timeout => (Outcome::Error, Some("no-response".into())),
                        Wait::Control(control) => match control {
                            Control::Skip | Control::Next => (Outcome::Skipped, None),
                            Control::Resume => continue 'session,
                            Control::Pause => {
                                progress.state = "paused";
                                emit_progress(&app, &progress);
                                if wait_resume(&mut ctl).await {
                                    continue 'session;
                                }
                                summary.stopped = true;
                                break 'session;
                            }
                            Control::Stop | Control::IgClosed => {
                                summary.stopped = true;
                                break 'session;
                            }
                        },
                    },
                    SessionMode::Manual => {
                        let mut remaining = delay.ceil().max(1.0) as u64;
                        let mut result = (Outcome::Opened, None);
                        loop {
                            progress.countdown = Some(remaining);
                            emit_progress(&app, &progress);
                            if remaining == 0 {
                                break;
                            }
                            match wait_event(&mut ctl, &mut rep, &token, Duration::from_secs(1)).await {
                                Wait::Timeout => remaining -= 1,
                                Wait::Report(report) => match report.kind.as_str() {
                                    "participated" => {
                                        result = (Outcome::Participated, None);
                                        tokio::time::sleep(MANUAL_AFTER_SUCCESS).await;
                                        break;
                                    }
                                    "already" => {
                                        result = (Outcome::Already, None);
                                        break;
                                    }
                                    "ended" => {
                                        result = (Outcome::Ended, None);
                                        break;
                                    }
                                    _ => {}
                                },
                                Wait::Control(Control::Next) => break,
                                Wait::Control(Control::Skip) => {
                                    result = (Outcome::Skipped, None);
                                    break;
                                }
                                Wait::Control(Control::Resume) => {}
                                Wait::Control(Control::Pause) => {
                                    progress.state = "paused";
                                    emit_progress(&app, &progress);
                                    if !wait_resume(&mut ctl).await {
                                        summary.stopped = true;
                                        break 'session;
                                    }
                                    progress.state = "running";
                                }
                                Wait::Control(Control::Stop | Control::IgClosed) => {
                                    summary.stopped = true;
                                    break 'session;
                                }
                            }
                        }
                        progress.countdown = None;
                        result
                    }
                }
            }
        };

        record(&app, &code, outcome, mode, message.clone(), i);
        match outcome {
            Outcome::Participated => summary.participated += 1,
            Outcome::Already => summary.already += 1,
            Outcome::Opened => summary.opened += 1,
            Outcome::Ended => summary.ended += 1,
            Outcome::Skipped => summary.skipped += 1,
            Outcome::Captcha => summary.pending.push(code.clone()),
            Outcome::Error | Outcome::Login => summary.errors += 1,
        }
        progress.participated = summary.participated;
        progress.already = summary.already;
        progress.opened = summary.opened;
        progress.ended = summary.ended;
        progress.skipped = summary.skipped;
        progress.errors = summary.errors;
        progress.pending = summary.pending.len();
        let _ = app.emit("session-log", LogEntry { at: Utc::now(), code: code.clone(), outcome, message });
        i += 1;
        progress.index = i;

        // Garde-fous : on s'arrête dès que le site montre des signes de méfiance ou ne répond plus.
        if outcome == Outcome::Error && !site_warning {
            consecutive_errors += 1;
        } else if outcome != Outcome::Skipped {
            consecutive_errors = 0;
        }
        if outcome == Outcome::Captcha {
            captchas += 1;
        }
        let stop_reason = if careful && site_warning {
            Some("warning")
        } else if careful && captchas >= SAFE_MAX_CAPTCHAS {
            Some("captcha")
        } else if mode == SessionMode::Auto && consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
            Some("errors")
        } else {
            None
        };
        if let Some(reason) = stop_reason {
            if i < total {
                summary.stopped = true;
                summary.stop_reason = Some(reason.to_string());
                break 'session;
            }
        }

        // Rythme entre deux concours en mode automatique.
        if mode == SessionMode::Auto && i < total && outcome != Outcome::Skipped {
            progress.remaining_secs = estimate(mode, total - i, delay);
            emit_progress(&app, &progress);
            let pace = tokio::time::sleep(Duration::from_secs_f64(delay));
            tokio::pin!(pace);
            loop {
                tokio::select! {
                    _ = &mut pace => break,
                    control = ctl.recv() => match control.unwrap_or(Control::Stop) {
                        Control::Next | Control::Skip => break,
                        Control::Resume => {}
                        Control::Pause => {
                            progress.state = "paused";
                            emit_progress(&app, &progress);
                            if !wait_resume(&mut ctl).await {
                                summary.stopped = true;
                                break 'session;
                            }
                            break;
                        }
                        Control::Stop | Control::IgClosed => {
                            summary.stopped = true;
                            break 'session;
                        }
                    },
                }
            }
        }
    }

    finish(&app, summary, progress);
}

fn record(app: &AppHandle, code: &str, outcome: Outcome, mode: SessionMode, message: Option<String>, index: usize) {
    let state = app.state::<AppState>();
    let Ok(mut store) = state.store.lock() else { return };
    store.record(code, outcome, mode, message, Utc::now());
    let _ = store.save_giveaways();
    if index % 5 == 4 {
        let _ = store.save_history();
    }
}

fn finish(app: &AppHandle, mut summary: SessionSummary, mut progress: Progress) {
    let state = app.state::<AppState>();
    summary.finished_at = Some(Utc::now());
    if let Ok(mut store) = state.store.lock() {
        store.meta.last_session = Some(summary.clone());
        let _ = store.save_history();
        let _ = store.save_giveaways();
        let _ = store.save_meta();
    }
    if let Ok(mut session) = state.session.lock() {
        *session = None;
    }
    if let Ok(mut reports) = state.reports.lock() {
        *reports = None;
    }
    state.set_script(None);
    automation::close_ig(app);

    progress.state = "finished";
    progress.current = None;
    progress.countdown = None;
    progress.remaining_secs = 0;
    emit_progress(app, &progress);
    let _ = app.emit("session-finished", &summary);
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.request_user_attention(Some(UserAttentionType::Informational));
    }
}
