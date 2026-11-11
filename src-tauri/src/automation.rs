//! Fenêtre Instant Gaming intégrée et script de participation.
//!
//! La fenêtre affiche le vrai site, avec la session de l'utilisateur (cookies gardés dans le
//! dossier de données). Elle n'a accès à aucune commande de l'application : le script injecté
//! signale son résultat en demandant une navigation vers un hôte réservé
//! (`giveaway-ig.invalid`), que l'application intercepte et annule.

use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use url::Url;

use crate::checker;
use crate::session::Control;
use crate::state::AppState;

pub const IG_LABEL: &str = "ig";
pub const REPORT_HOST: &str = "giveaway-ig.invalid";

/// Arguments WebView2, identiques pour toutes les fenêtres (elles partagent le même profil).
/// En plus des réglages par défaut de wry, ils évitent que la page d'une fenêtre cachée
/// soit ralentie par le navigateur.
pub const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
--disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows";

/// Ce que doit faire la fenêtre Instant Gaming quand on y ouvre une page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Afficher et mettre au premier plan.
    Focus,
    /// Afficher sans voler le focus.
    Show,
    /// Garder cachée (session automatique en arrière-plan).
    Hidden,
    /// Ne rien changer.
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptMode {
    /// Clique sur « Participer » et observe le résultat.
    Auto,
    /// Ne clique pas : signale seulement quand l'utilisateur a participé (mode manuel).
    Watch,
    /// Vérifie seulement si le compte est connecté.
    Probe,
}

impl ScriptMode {
    fn as_str(self) -> &'static str {
        match self {
            ScriptMode::Auto => "auto",
            ScriptMode::Watch => "watch",
            ScriptMode::Probe => "probe",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub kind: String,
    pub code: String,
    pub token: String,
    pub message: Option<String>,
}

const SCRIPT_TEMPLATE: &str = r#"(() => {
  const TOKEN = __TOKEN__;
  if (window.__giveawayIg === TOKEN) return;
  window.__giveawayIg = TOKEN;
  const MODE = __MODE__;
  const CODE = __CODE__;
  const ENDED = __ENDED__;
  let done = false;
  const report = (kind, message) => {
    if (done) return;
    done = true;
    const u = new URL('https://__HOST__/report');
    u.searchParams.set('k', kind);
    u.searchParams.set('c', CODE);
    u.searchParams.set('t', TOKEN);
    if (message) u.searchParams.set('m', String(message).slice(0, 200));
    window.location.assign(u.toString());
  };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const text = () => (document.body ? document.body.innerText : '');
  const ended = () => ENDED.some((e) => text().includes(e));
  const root = () => document.getElementById('giveaway-app');
  const vm = () => { const el = root(); return el && el.__vue__ ? el.__vue__ : null; };
  const loggedOut = () => typeof window.signup_url === 'string' && window.signup_url.length > 0;
  const participating = () => {
    const v = vm();
    if (v && v.isParticipating) return true;
    return !!(window.giveaway && window.giveaway.isParticipating);
  };
  const captchaVisible = () => Array.from(document.querySelectorAll('iframe')).some((f) =>
    /recaptcha\/api2\/bframe|recaptcha\/enterprise\/bframe|hcaptcha|challenges\.cloudflare/.test(f.src || '') &&
    f.offsetWidth > 50 && f.offsetHeight > 50);
  const findButton = () =>
    document.querySelector('#giveaway-app .participate-actions button.validate') ||
    Array.from(document.querySelectorAll('#giveaway-app button')).find((b) =>
      /particip|teilnehm|partecip|weź udział|wez udzial/i.test(b.innerText || ''));

  (async () => {
    if (!/\/giveaway\//i.test(location.pathname)) {
      if (MODE === 'probe') report(loggedOut() ? 'login' : 'connected');
      else report('ended', 'redirect');
      return;
    }
    // Délais mesurés en temps réel (et non en nombre de tours) pour rester justes même si
    // le navigateur ralentit les minuteries de la page.
    const loadDeadline = Date.now() + 15000;
    while (Date.now() < loadDeadline) {
      if (ended() || (root() && (window.giveaway || findButton()))) break;
      await sleep(250);
    }
    if (MODE === 'probe') { report(loggedOut() ? 'login' : 'connected'); return; }
    if (ended()) { report('ended'); return; }
    if (loggedOut()) { report('login'); return; }
    if (participating()) { report('already'); return; }
    if (MODE === 'watch') {
      for (;;) { await sleep(400); if (participating()) { report('participated'); return; } }
    }
    const button = findButton();
    if (!button) { report('nobutton'); return; }
    let warning = null;
    try {
      if (window.igPopin && typeof window.igPopin.showWarning === 'function') {
        const original = window.igPopin.showWarning.bind(window.igPopin);
        window.igPopin.showWarning = (options) => {
          warning = (options && options.message) || 'warning';
          return original(options);
        };
      }
    } catch (e) { /* page différente de celle attendue */ }
    // Laisse la page finir de se charger avant de cliquer.
    await sleep(1500);
    button.scrollIntoView({ block: 'center' });
    button.click();
    const clickDeadline = Date.now() + 15000;
    while (Date.now() < clickDeadline) {
      await sleep(250);
      if (participating()) { report('participated'); return; }
      if (warning) { report('error', warning); return; }
      if (captchaVisible()) { report('captcha'); return; }
    }
    report(captchaVisible() ? 'captcha' : 'timeout');
  })();
})();"#;

fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}

pub fn build_script(mode: ScriptMode, code: &str, token: &str) -> String {
    let ended = serde_json::to_string(&checker::ended_markers()).unwrap_or_else(|_| "[]".into());
    SCRIPT_TEMPLATE
        .replace("__TOKEN__", &js_string(token))
        .replace("__MODE__", &js_string(mode.as_str()))
        .replace("__CODE__", &js_string(code))
        .replace("__ENDED__", &ended)
        .replace("__HOST__", REPORT_HOST)
}

pub fn parse_report(url: &Url) -> Option<Report> {
    if url.host_str() != Some(REPORT_HOST) {
        return None;
    }
    let mut report = Report {
        kind: String::new(),
        code: String::new(),
        token: String::new(),
        message: None,
    };
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "k" => report.kind = value.into_owned(),
            "c" => report.code = value.into_owned(),
            "t" => report.token = value.into_owned(),
            "m" => report.message = Some(value.chars().take(200).collect()),
            _ => {}
        }
    }
    if report.kind.is_empty() {
        None
    } else {
        Some(report)
    }
}

/// Jeton aléatoire pour reconnaître les rapports de la page en cours.
pub fn new_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    );
    format!("{:016x}", hasher.finish())
}

pub fn ig_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(IG_LABEL)
}

pub fn reveal(window: &WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Ouvre (ou réutilise) la fenêtre Instant Gaming sur `url`.
pub fn open_ig(
    app: &AppHandle,
    url: &str,
    visibility: Visibility,
) -> Result<WebviewWindow, String> {
    let target = Url::parse(url).map_err(|e| e.to_string())?;
    if let Some(window) = ig_window(app) {
        window.navigate(target).map_err(|e| e.to_string())?;
        match visibility {
            Visibility::Focus => reveal(&window),
            Visibility::Show => {
                let _ = window.show();
            }
            Visibility::Hidden => {
                let _ = window.hide();
            }
            Visibility::Keep => {}
        }
        return Ok(window);
    }

    let state = app.state::<AppState>();
    let nav_handle = app.clone();
    let load_handle = app.clone();
    let window = WebviewWindowBuilder::new(app, IG_LABEL, WebviewUrl::External(target))
        .title("Instant Gaming · Giveaway IG")
        .inner_size(1100.0, 820.0)
        .min_inner_size(700.0, 500.0)
        .data_directory(state.paths.webview.clone())
        .additional_browser_args(BROWSER_ARGS)
        .visible(visibility != Visibility::Hidden)
        .focused(visibility == Visibility::Focus)
        .on_navigation(move |url| {
            if url.host_str() == Some(REPORT_HOST) {
                if let Some(report) = parse_report(url) {
                    nav_handle.state::<AppState>().send_report(report);
                }
                return false;
            }
            true
        })
        .on_page_load(move |webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let script = load_handle
                    .state::<AppState>()
                    .ig_script
                    .lock()
                    .ok()
                    .and_then(|s| s.clone());
                if let Some(script) = script {
                    let _ = webview.eval(&script);
                }
            }
        })
        .build()
        .map_err(|e| e.to_string())?;

    let close_handle = app.clone();
    window.on_window_event(move |event| match event {
        // Pendant une session automatique, fermer la fenêtre la cache seulement : la session continue.
        WindowEvent::CloseRequested { api, .. } => {
            let state = close_handle.state::<AppState>();
            if state
                .ig_hide_on_close
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                api.prevent_close();
                if let Some(window) = ig_window(&close_handle) {
                    let _ = window.hide();
                }
            }
        }
        WindowEvent::Destroyed => {
            close_handle
                .state::<AppState>()
                .send_control(Control::IgClosed);
        }
        _ => {}
    });
    Ok(window)
}

pub fn close_ig(app: &AppHandle) {
    if let Some(window) = ig_window(app) {
        let _ = window.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_contains_parameters() {
        let script = build_script(ScriptMode::Auto, "TOMBIE", "abc123");
        assert!(script.contains("\"TOMBIE\""));
        assert!(script.contains("\"auto\""));
        assert!(script.contains("\"abc123\""));
        assert!(script.contains("Ce Giveaway est terminé"));
        for placeholder in ["__TOKEN__", "__MODE__", "__CODE__", "__ENDED__", "__HOST__"] {
            assert!(!script.contains(placeholder));
        }
    }

    #[test]
    fn parses_reports() {
        let url = Url::parse("https://giveaway-ig.invalid/report?k=error&c=PIVI&t=tok&m=Oops%20!")
            .unwrap();
        let report = parse_report(&url).unwrap();
        assert_eq!(report.kind, "error");
        assert_eq!(report.code, "PIVI");
        assert_eq!(report.token, "tok");
        assert_eq!(report.message.as_deref(), Some("Oops !"));
        assert!(
            parse_report(&Url::parse("https://www.instant-gaming.com/report?k=x").unwrap())
                .is_none()
        );
    }

    #[test]
    fn tokens_differ() {
        assert_ne!(new_token(), new_token());
    }
}
