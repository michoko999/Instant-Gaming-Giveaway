//! État partagé de l'application et emplacement des données.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::Client;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::UnboundedSender;

use crate::automation::Report;
use crate::session::Control;
use crate::store::Store;

/// Nom du dossier créé à côté de l'exécutable en mode portable.
pub const PORTABLE_DIR: &str = "GiveawayIG-data";
/// Fichier témoin : le profil de navigation sera effacé au prochain démarrage (déconnexion).
const RESET_MARKER: &str = "reset-webview";
/// Dossiers à supprimer au prochain démarrage (profils de comptes retirés encore occupés).
const PENDING_DELETE: &str = "pending-delete.txt";
/// Profils de navigation des comptes ajoutés (le premier compte garde `webview`).
const PROFILES_DIR: &str = "profiles";

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";

/// Comment l'application a été installée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Setup {
    /// Fichier seul (exe ou AppImage) : données à côté, dans `GiveawayIG-data`.
    Portable,
    /// Installateur Windows : données dans le dossier de l'utilisateur (`%LOCALAPPDATA%`).
    Installed,
    /// Installée avec Scoop : données à côté de l'exe, conservées par Scoop entre versions.
    Scoop,
}

/// Type d'installation d'après l'emplacement de l'exécutable.
pub fn setup_of(exe: &Path) -> Setup {
    let lower = exe.to_string_lossy().replace('\\', "/").to_lowercase();
    let in_scoop_root = std::env::var_os("SCOOP")
        .map(|root| {
            let root = root.to_string_lossy().replace('\\', "/").to_lowercase();
            !root.is_empty() && lower.starts_with(&format!("{}/apps/", root.trim_end_matches('/')))
        })
        .unwrap_or(false);
    if in_scoop_root || lower.contains("/scoop/apps/") {
        return Setup::Scoop;
    }
    // L'installateur range `uninstall.exe` à côté du programme.
    if cfg!(windows)
        && exe
            .parent()
            .is_some_and(|dir| dir.join("uninstall.exe").is_file())
    {
        return Setup::Installed;
    }
    Setup::Portable
}

/// Type d'installation de l'application en cours.
pub fn current_setup() -> Setup {
    std::env::current_exe()
        .map(|exe| setup_of(&exe))
        .unwrap_or(Setup::Portable)
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data: PathBuf,
    pub webview: PathBuf,
    pub portable: bool,
    pub setup: Setup,
}

impl AppPaths {
    /// Dossier à côté de l'exécutable s'il est accessible en écriture (mode portable),
    /// sinon le dossier de données local de l'utilisateur (version installée, ou dossier
    /// de l'exécutable en lecture seule).
    pub fn resolve(app: &AppHandle) -> AppPaths {
        let setup = current_setup();
        let portable_dir = portable_base()
            .filter(|_| setup != Setup::Installed)
            .map(|dir| dir.join(PORTABLE_DIR))
            .filter(|dir| is_writable_dir(dir));
        let (data, portable) = match portable_dir {
            Some(dir) => (dir, true),
            None => {
                let fallback = app
                    .path()
                    .app_local_data_dir()
                    .unwrap_or_else(|_| std::env::temp_dir().join("GiveawayIG"));
                let _ = std::fs::create_dir_all(&fallback);
                (fallback, false)
            }
        };
        let webview = data.join("webview");
        AppPaths {
            data,
            webview,
            portable,
            setup,
        }
    }

    pub fn request_webview_reset(&self) {
        let _ = std::fs::write(self.data.join(RESET_MARKER), b"1");
    }

    /// Efface le profil de navigation si une déconnexion a été demandée, et les profils des
    /// comptes retirés qui n'avaient pas pu être supprimés.
    pub fn apply_pending_reset(&self) {
        let marker = self.data.join(RESET_MARKER);
        if marker.exists() {
            let _ = std::fs::remove_dir_all(&self.webview);
            let _ = std::fs::remove_file(marker);
        }
        let list = self.data.join(PENDING_DELETE);
        if let Ok(text) = std::fs::read_to_string(&list) {
            let profiles = self.data.join(PROFILES_DIR);
            for name in text
                .lines()
                .map(str::trim)
                .filter(|l| crate::model::valid_account_id(l))
            {
                let _ = std::fs::remove_dir_all(profiles.join(name));
            }
            let _ = std::fs::remove_file(list);
        }
    }

    /// Profil de navigation d'un compte (cookies de connexion Instant Gaming).
    pub fn profile_dir(&self, account: &str) -> PathBuf {
        if account == crate::model::MAIN_ACCOUNT {
            self.webview.clone()
        } else {
            self.data.join(PROFILES_DIR).join(account)
        }
    }

    /// Supprime le profil d'un compte ajouté ; s'il est encore utilisé, au prochain démarrage.
    pub fn delete_profile(&self, account: &str) {
        if account == crate::model::MAIN_ACCOUNT || !crate::model::valid_account_id(account) {
            return;
        }
        let dir = self.profile_dir(account);
        if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.data.join(PENDING_DELETE))
            {
                let _ = writeln!(file, "{account}");
            }
        }
    }
}

/// Dossier de l'application portable : celui de l'exécutable, ou sous Linux celui du fichier
/// `.AppImage` (l'AppImage s'exécute depuis un dossier temporaire en lecture seule et indique
/// son vrai emplacement dans la variable `APPIMAGE`).
fn portable_base() -> Option<PathBuf> {
    portable_base_from(
        std::env::var_os("APPIMAGE").map(PathBuf::from),
        std::env::current_exe().ok(),
    )
}

fn portable_base_from(appimage: Option<PathBuf>, exe: Option<PathBuf>) -> Option<PathBuf> {
    if cfg!(target_os = "linux") {
        if let Some(dir) = appimage
            .filter(|p| p.is_absolute())
            .and_then(|p| p.parent().map(Path::to_path_buf))
        {
            return Some(dir);
        }
    }
    exe.and_then(|exe| exe.parent().map(Path::to_path_buf))
}

fn is_writable_dir(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write-test");
    let ok = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(probe);
    ok
}

pub fn http_client() -> Client {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new())
}

pub struct AppState {
    pub paths: AppPaths,
    pub store: Mutex<Store>,
    pub client: Client,
    /// Synchronisation ou vérification en cours.
    pub busy: AtomicBool,
    pub session: Mutex<Option<UnboundedSender<Control>>>,
    pub reports: Mutex<Option<UnboundedSender<Report>>>,
    /// Script injecté à chaque page chargée dans la fenêtre Instant Gaming.
    pub ig_script: Mutex<Option<String>>,
    /// Fermer la fenêtre Instant Gaming la cache au lieu de la fermer (session automatique en cours).
    pub ig_hide_on_close: AtomicBool,
    /// Compte dont le profil est chargé dans la fenêtre Instant Gaming ouverte.
    pub ig_account: Mutex<Option<String>>,
    /// Vérification de connexion en cours (fenêtre Instant Gaming occupée).
    pub probing: AtomicBool,
}

impl AppState {
    pub fn new(paths: AppPaths, store: Store) -> AppState {
        AppState {
            paths,
            store: Mutex::new(store),
            client: http_client(),
            busy: AtomicBool::new(false),
            session: Mutex::new(None),
            reports: Mutex::new(None),
            ig_script: Mutex::new(None),
            ig_hide_on_close: AtomicBool::new(false),
            ig_account: Mutex::new(None),
            probing: AtomicBool::new(false),
        }
    }

    pub fn send_report(&self, report: Report) {
        if let Ok(guard) = self.reports.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.send(report);
            }
        }
    }

    pub fn send_control(&self, control: Control) -> bool {
        match self.session.lock() {
            Ok(guard) => guard
                .as_ref()
                .map(|tx| tx.send(control).is_ok())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    pub fn session_running(&self) -> bool {
        self.session.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn set_script(&self, script: Option<String>) {
        if let Ok(mut guard) = self.ig_script.lock() {
            *guard = script;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_scoop_installs() {
        assert_eq!(
            setup_of(Path::new(
                r"C:\Users\me\scoop\apps\giveaway-ig\current\GiveawayIG.exe"
            )),
            Setup::Scoop
        );
        assert_eq!(
            setup_of(Path::new("/home/me/Apps/GiveawayIG-x86_64.AppImage")),
            Setup::Portable
        );
    }

    #[test]
    fn portable_folder_sits_next_to_the_app_file() {
        let exe = Some(PathBuf::from("/tmp/.mount_GiveawXYZ/usr/bin/giveaway-ig"));
        let appimage = Some(PathBuf::from("/home/me/Apps/GiveawayIG-x86_64.AppImage"));
        let expected = if cfg!(target_os = "linux") {
            PathBuf::from("/home/me/Apps")
        } else {
            PathBuf::from("/tmp/.mount_GiveawXYZ/usr/bin")
        };
        assert_eq!(portable_base_from(appimage, exe.clone()), Some(expected));
        // Sans AppImage (exe Windows, binaire Linux) : à côté de l'exécutable.
        assert_eq!(
            portable_base_from(None, exe),
            Some(PathBuf::from("/tmp/.mount_GiveawXYZ/usr/bin"))
        );
        // Chemin relatif ignoré.
        assert_eq!(
            portable_base_from(
                Some(PathBuf::from("rel.AppImage")),
                Some(PathBuf::from("/opt/app/bin"))
            ),
            Some(PathBuf::from("/opt/app"))
        );
    }
}
