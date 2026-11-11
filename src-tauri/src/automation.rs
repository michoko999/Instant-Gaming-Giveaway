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
    if (message) u.searchParams.set('m', String(message).slice(0, 600));
    window.location.assign(u.toString());
  };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const text = () => (document.body ? document.body.innerText : '');
  const ended = () => ENDED.some((e) => text().includes(e));
  const root = () => document.getElementById('giveaway-app');
  const vm = () => { const el = root(); return el && el.__vue__ ? el.__vue__ : null; };
  const loggedOut = () => {
    // `window.user.isAuthenticated` est présent sur toutes les pages du site.
    if (window.user && typeof window.user.isAuthenticated === 'boolean') return !window.user.isAuthenticated;
    return typeof window.signup_url === 'string' && window.signup_url.length > 0;
  };
  // Pseudo et photo du compte connecté, lus dans l'en-tête du site (`.login-container .user`).
  const identity = () => {
    const clean = (v) => (typeof v === 'string' ? v.replace(/\s+/g, ' ').trim() : '');
    const u = window.user || {};
    const box = document.querySelector('.login-container .user') || document.querySelector('.login-container');
    const img = box ? box.querySelector('.avatar img, img.ig-avatar, img') : null;
    const holder = box ? box.querySelector('.avatar') : null;
    const link = document.querySelector('.login-container a[href*="/user/"]');
    let avatar = img ? img.getAttribute('data-src') || img.currentSrc || img.getAttribute('src') || '' : '';
    try { avatar = avatar ? new URL(avatar, location.href).href : ''; } catch (e) { avatar = ''; }
    if (/\/themes\//.test(avatar) || /^data:/.test(avatar)) avatar = '';
    let profile = '';
    const href = link ? link.getAttribute('href') || '' : '';
    const m = href.match(/\/user\/([^/?#]+)/);
    if (m) { try { profile = decodeURIComponent(m[1]); } catch (e) { profile = m[1]; } }
    const name = [u.username, u.nickname, u.name, u.custom_rname,
      img && img.getAttribute('alt'), holder && holder.getAttribute('title'),
      link && link.getAttribute('title'), profile].map(clean).find((v) => v && v.length <= 60) || '';
    return JSON.stringify({ name, avatar, profile });
  };
  const connectedReport = () => (loggedOut() ? report('login') : report('connected', identity()));
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
      if (MODE === 'probe') connectedReport();
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
    if (MODE === 'probe') { connectedReport(); return; }
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
            "m" => report.message = Some(value.chars().take(600).collect()),
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

/// Pseudo et photo lus sur la page Instant Gaming, vérifiés avant d'être gardés.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identity {
    pub name: Option<String>,
    pub avatar_url: Option<Url>,
    pub profile: Option<String>,
}

/// Sites dont la page Instant Gaming peut ouvrir une fenêtre (pop-up) : la connexion avec
/// Google, Facebook, Apple ou Discord passe par une fenêtre ouverte avec `window.open`.
/// Les autres pop-ups (publicités, partages…) restent bloquées.
const POPUP_HOSTS: [&str; 6] = [
    "instant-gaming.com",
    "accounts.google.com",
    "facebook.com",
    "appleid.apple.com",
    "discord.com",
    "discordapp.com",
];

/// La page peut-elle ouvrir cette fenêtre ? (connexion par un compte Google, Facebook, Apple, Discord)
pub fn popup_allowed(url: &Url) -> bool {
    if url.as_str() == "about:blank" {
        // Certaines connexions ouvrent d'abord une fenêtre vide, puis la dirigent vers le site.
        return true;
    }
    url.scheme() == "https"
        && url
            .host_str()
            .map(|host| {
                POPUP_HOSTS
                    .iter()
                    .any(|h| host == *h || host.ends_with(&format!(".{h}")))
            })
            .unwrap_or(false)
}

/// Hôtes autorisés pour la photo de profil (CDN d'Instant Gaming).
pub fn avatar_host_allowed(host: &str) -> bool {
    ["gaming-cdn.com", "instant-gaming.com"]
        .iter()
        .any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

impl Identity {
    pub fn parse(message: Option<&str>) -> Identity {
        #[derive(serde::Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            name: String,
            avatar: String,
            profile: String,
        }
        let raw: Raw = message
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or_default();
        let name: String = raw
            .name
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .trim()
            .chars()
            .take(40)
            .collect();
        let avatar_url = Url::parse(&raw.avatar).ok().filter(|u| {
            u.scheme() == "https" && u.host_str().map(avatar_host_allowed).unwrap_or(false)
        });
        let profile = Some(raw.profile.trim().to_string()).filter(|p| {
            !p.is_empty()
                && p.len() <= 60
                && p != "."
                && p != ".."
                && p.chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        });
        Identity {
            name: Some(name).filter(|n| !n.is_empty()),
            avatar_url,
            profile,
        }
    }
}

/// Ouvre (ou réutilise) la fenêtre Instant Gaming sur `url`, avec le profil du compte actif.
pub fn open_ig(
    app: &AppHandle,
    url: &str,
    visibility: Visibility,
) -> Result<WebviewWindow, String> {
    let target = Url::parse(url).map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let (account, account_name) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let active = store.accounts.active();
        (active.id.clone(), active.name.clone())
    };
    if let Some(window) = ig_window(app) {
        let loaded = state.ig_account.lock().ok().and_then(|a| a.clone());
        if loaded.as_deref() != Some(account.as_str()) {
            // Fenêtre ouverte avec le profil d'un autre compte : elle doit être recréée.
            let _ = window.destroy();
            return Err("ig-closed".into());
        }
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

    let nav_handle = app.clone();
    let load_handle = app.clone();
    let title = match account_name {
        Some(name) => format!("Instant Gaming · {name} · Giveaway IG"),
        None => "Instant Gaming · Giveaway IG".to_string(),
    };
    let window = WebviewWindowBuilder::new(app, IG_LABEL, WebviewUrl::External(target))
        .title(&title)
        .inner_size(1100.0, 820.0)
        .min_inner_size(700.0, 500.0)
        .data_directory(state.paths.profile_dir(&account))
        .additional_browser_args(BROWSER_ARGS)
        .visible(visibility != Visibility::Hidden)
        .focused(visibility == Visibility::Focus)
        // Connexion avec Google, Facebook, Apple ou Discord : la fenêtre de connexion s'ouvre
        // dans une vraie fenêtre (même profil, reliée à la page pour lui renvoyer le résultat).
        .on_new_window(|url, _features| {
            if popup_allowed(&url) {
                tauri::webview::NewWindowResponse::Allow
            } else {
                tauri::webview::NewWindowResponse::Deny
            }
        })
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
    if let Ok(mut loaded) = state.ig_account.lock() {
        *loaded = Some(account);
    }

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
            let state = close_handle.state::<AppState>();
            if let Ok(mut loaded) = state.ig_account.lock() {
                *loaded = None;
            }
            state.send_control(Control::IgClosed);
        }
        _ => {}
    });
    Ok(window)
}

/// Ferme la fenêtre Instant Gaming et attend qu'elle ait disparu (changement de compte).
pub async fn close_ig_and_wait(app: &AppHandle) {
    close_ig(app);
    for _ in 0..60 {
        if ig_window(app).is_none() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
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
    fn only_sign_in_popups_are_allowed() {
        let ok = [
            "https://accounts.google.com/o/oauth2/v2/auth?client_id=x&ux_mode=popup",
            "https://www.facebook.com/v19.0/dialog/oauth?app_id=1",
            "https://m.facebook.com/login.php",
            "https://appleid.apple.com/auth/authorize?client_id=x",
            "https://discord.com/oauth2/authorize?client_id=x",
            "https://www.instant-gaming.com/fr/user/connect/discord/",
            "about:blank",
        ];
        for u in ok {
            assert!(popup_allowed(&Url::parse(u).unwrap()), "{u}");
        }
        let blocked = [
            "https://ads.example.com/promo",
            "https://twitter.com/intent/tweet?text=x",
            "https://api.whatsapp.com/send?text=x",
            "https://accounts.google.com.evil.example/",
            "https://notdiscord.com/oauth2",
            "http://accounts.google.com/o/oauth2/v2/auth",
            "javascript:alert(1)",
        ];
        for u in blocked {
            assert!(!popup_allowed(&Url::parse(u).unwrap()), "{u}");
        }
    }

    #[test]
    fn reads_identity_from_the_page() {
        let id = Identity::parse(Some(
            r#"{"name":"  Yaya\n ","avatar":"https://gaming-cdn.com/images/avatars/12281209-1618124808.jpg","profile":"gamer-09d6ee"}"#,
        ));
        assert_eq!(id.name.as_deref(), Some("Yaya"));
        assert_eq!(
            id.avatar_url.as_ref().map(|u| u.as_str()),
            Some("https://gaming-cdn.com/images/avatars/12281209-1618124808.jpg")
        );
        assert_eq!(id.profile.as_deref(), Some("gamer-09d6ee"));
        // Photo ailleurs que chez Instant Gaming, ou pas en https : ignorée.
        let other = Identity::parse(Some(
            r#"{"name":"x","avatar":"https://evil.example/a.jpg","profile":"../etc"}"#,
        ));
        assert!(other.avatar_url.is_none() && other.profile.is_none());
        let accented = Identity::parse(Some(r#"{"name":"Michöko-99","profile":"Michöko-99"}"#));
        assert_eq!(accented.profile.as_deref(), Some("Michöko-99"));
        assert!(
            Identity::parse(Some(r#"{"avatar":"http://gaming-cdn.com/a.jpg"}"#))
                .avatar_url
                .is_none()
        );
        assert_eq!(Identity::parse(Some("pas du json")), Identity::default());
        assert_eq!(Identity::parse(None), Identity::default());
        let long = Identity::parse(Some(&format!(r#"{{"name":"{}"}}"#, "a".repeat(80))));
        assert_eq!(long.name.map(|n| n.len()), Some(40));
    }

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
