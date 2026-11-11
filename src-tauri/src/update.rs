//! Mises à jour : vérifie la dernière version publiée sur GitHub et, sous Windows, l'installe
//! en un clic.
//!
//! L'installation reste volontairement simple (rien de caché, rien en arrière-plan) :
//! 1. téléchargement de `GiveawayIG.exe` et de `SHA256SUMS.txt` depuis la release GitHub ;
//! 2. vérification de la taille et de l'empreinte SHA-256 publiée ;
//! 3. l'exécutable en cours est renommé en `….old` (Windows le permet), le nouveau prend sa
//!    place, puis il est lancé et l'ancien processus se ferme ;
//! 4. au démarrage suivant, le fichier `….old` est supprimé.
//!
//! L'empreinte vient de la même release que l'exécutable : elle garantit que le fichier est
//! complet et identique à celui publié, comme un téléchargement manuel depuis la page.

use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const LATEST_RELEASE: &str =
    "https://api.github.com/repos/michoko999/Instant-Gaming-Giveaway/releases/latest";
/// Seuls les fichiers des releases de ce dépôt sont acceptés.
const DOWNLOAD_PREFIX: &str =
    "https://github.com/michoko999/instant-gaming-giveaway/releases/download/";
const EXE_ASSET: &str = "GiveawayIG.exe";
const APPIMAGE_ASSET: &str = "GiveawayIG-x86_64.AppImage";

/// Ce que la mise à jour remplace sur cette machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    WindowsExe,
    LinuxAppImage,
}

impl Kind {
    fn asset(self) -> &'static str {
        match self {
            Kind::WindowsExe => EXE_ASSET,
            Kind::LinuxAppImage => APPIMAGE_ASSET,
        }
    }

    /// Début obligatoire du fichier : exécutable Windows (MZ) ou Linux (ELF).
    fn magic(self) -> &'static [u8] {
        match self {
            Kind::WindowsExe => b"MZ",
            Kind::LinuxAppImage => b"\x7fELF",
        }
    }
}

/// Type de mise à jour possible ici et fichier à remplacer : l'exécutable sous Windows,
/// le fichier `.AppImage` sous Linux (indiqué par la variable `APPIMAGE`).
pub fn local_target() -> Option<(Kind, PathBuf)> {
    if cfg!(windows) {
        return std::env::current_exe()
            .ok()
            .map(|path| (Kind::WindowsExe, path));
    }
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return std::env::var_os("APPIMAGE")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_file())
            .map(|path| (Kind::LinuxAppImage, path));
    }
    None
}
const SUMS_ASSET: &str = "SHA256SUMS.txt";
const MAX_EXE_BYTES: u64 = 300 * 1024 * 1024;
const MAX_SUMS_BYTES: usize = 64 * 1024;
/// Le téléchargement de l'exécutable peut être long sur une petite connexion.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: Option<String>,
    pub available: bool,
    pub url: Option<String>,
    /// La mise à jour peut être installée depuis l'application (Windows, fichiers présents).
    pub can_install: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

pub fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let cleaned = raw.trim().trim_start_matches(['v', 'V']);
    let core = cleaned.split(['-', '+', ' ']).next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u64>());
    let major = parts.next()?.ok()?;
    let minor = parts.next().unwrap_or(Ok(0)).ok()?;
    let patch = parts.next().unwrap_or(Ok(0)).ok()?;
    Some((major, minor, patch))
}

pub async fn latest(client: &Client) -> Result<Release, String> {
    let response = client
        .get(LATEST_RELEASE)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| "GitHub injoignable".to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    let body = response.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&body).map_err(|e| e.to_string())
}

pub async fn check(client: &Client) -> Result<UpdateInfo, String> {
    Ok(info(&latest(client).await?))
}

pub fn info(release: &Release) -> UpdateInfo {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let newer = match (parse_version(&release.tag_name), parse_version(&current)) {
        (Some(latest), Some(mine)) => latest > mine,
        _ => false,
    };
    let available = newer && !release.draft && !release.prerelease;
    UpdateInfo {
        current,
        latest: Some(release.tag_name.clone()),
        available,
        url: Some(release.html_url.clone()),
        can_install: available
            && local_target()
                .map(|(kind, _)| files(release, kind).is_some())
                .unwrap_or(false),
    }
}

/// Le fichier à installer et le fichier d'empreintes de la release, s'ils viennent bien de ce dépôt.
fn files(release: &Release, kind: Kind) -> Option<(&Asset, &Asset)> {
    let find = |name: &str| {
        release.assets.iter().find(|a| {
            a.name == name
                && a.browser_download_url
                    .to_ascii_lowercase()
                    .starts_with(DOWNLOAD_PREFIX)
        })
    };
    Some((find(kind.asset())?, find(SUMS_ASSET)?))
}

/// Empreinte annoncée pour `name` dans un fichier au format `sha256sum` (« empreinte  nom »).
pub fn expected_hash(sums: &str, name: &str) -> Option<String> {
    sums.trim_start_matches('\u{feff}')
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');
            let valid = hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit());
            (valid && file.eq_ignore_ascii_case(name)).then(|| hash.to_ascii_lowercase())
        })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Vérifie le fichier téléchargé : taille annoncée, empreinte, et en-tête d'exécutable.
pub fn verify(
    bytes: &[u8],
    kind: Kind,
    expected_size: u64,
    expected_hash: &str,
) -> Result<(), String> {
    if expected_size > 0 && bytes.len() as u64 != expected_size {
        return Err("update-corrupt".into());
    }
    if !bytes.starts_with(kind.magic()) || sha256_hex(bytes) != expected_hash {
        return Err("update-corrupt".into());
    }
    Ok(())
}

/// Télécharge et vérifie le nouvel exécutable. `progress(reçu, total)` suit le téléchargement.
pub async fn download(
    client: &Client,
    release: &Release,
    kind: Kind,
    mut progress: impl FnMut(u64, u64),
) -> Result<Vec<u8>, String> {
    let (exe, sums) = files(release, kind).ok_or("update-none")?;
    if exe.size > MAX_EXE_BYTES {
        return Err("update-corrupt".into());
    }

    let sums_text = client
        .get(&sums.browser_download_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "update-download".to_string())?
        .text()
        .await
        .map_err(|_| "update-download".to_string())?;
    if sums_text.len() > MAX_SUMS_BYTES {
        return Err("update-no-hash".into());
    }
    let expected = expected_hash(&sums_text, kind.asset()).ok_or("update-no-hash")?;

    let mut response = client
        .get(&exe.browser_download_url)
        .timeout(DOWNLOAD_TIMEOUT)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "update-download".to_string())?;
    let total = response.content_length().unwrap_or(exe.size);
    if total > MAX_EXE_BYTES {
        return Err("update-corrupt".into());
    }
    let mut bytes = Vec::with_capacity(total as usize);
    progress(0, total);
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "update-download".to_string())?
    {
        bytes.extend_from_slice(&chunk);
        if bytes.len() as u64 > MAX_EXE_BYTES {
            return Err("update-corrupt".into());
        }
        progress(bytes.len() as u64, total);
    }
    verify(&bytes, kind, exe.size, &expected)?;
    Ok(bytes)
}

fn sibling(exe: &Path, suffix: &str) -> PathBuf {
    let name = exe
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "GiveawayIG".to_string());
    exe.with_file_name(format!("{name}{suffix}"))
}

/// Remplace l'exécutable `exe` (en cours d'utilisation) par `bytes`. En cas d'échec,
/// l'ancien exécutable est remis en place.
pub fn replace(exe: &Path, bytes: &[u8]) -> Result<(), String> {
    let staged = sibling(exe, ".new");
    let old = sibling(exe, ".old");
    std::fs::write(&staged, bytes).map_err(|_| "update-write".to_string())?;
    #[cfg(unix)]
    {
        // L'AppImage doit rester exécutable.
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755));
    }
    // Un reste d'une mise à jour précédente.
    let _ = std::fs::remove_file(&old);
    if std::fs::rename(exe, &old).is_err() {
        let _ = std::fs::remove_file(&staged);
        return Err("update-write".into());
    }
    if std::fs::rename(&staged, exe).is_err() {
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(&staged);
        return Err("update-write".into());
    }
    Ok(())
}

/// Au démarrage : supprime l'ancien exécutable laissé par une mise à jour.
pub fn cleanup() {
    let Some(exe) = local_target()
        .map(|(_, path)| path)
        .or_else(|| std::env::current_exe().ok())
    else {
        return;
    };
    for suffix in [".old", ".new"] {
        let path = sibling(&exe, suffix);
        if path.exists() {
            // L'ancien processus peut mettre un instant à se fermer.
            std::thread::spawn(move || {
                for _ in 0..20 {
                    if std::fs::remove_file(&path).is_ok() || !path.exists() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("V1.3"), Some((1, 3, 0)));
        assert_eq!(parse_version("v2.0.1"), Some((2, 0, 1)));
        assert_eq!(parse_version("2.1.0-beta.1"), Some((2, 1, 0)));
        assert_eq!(parse_version("exe"), None);
        assert!(parse_version("v2.0.0") > parse_version("V1.3"));
    }

    fn release(tag: &str, prefix: &str) -> Release {
        let asset = |name: &str| Asset {
            name: name.into(),
            browser_download_url: format!("{prefix}{tag}/{name}"),
            size: 10,
        };
        Release {
            tag_name: tag.into(),
            html_url: "https://github.com/x".into(),
            draft: false,
            prerelease: false,
            assets: vec![asset(EXE_ASSET), asset(SUMS_ASSET)],
        }
    }

    #[test]
    fn only_newer_releases_from_this_repo_are_installable() {
        let repo = "https://github.com/michoko999/Instant-Gaming-Giveaway/releases/download/";
        let newer = info(&release("v99.0.0", repo));
        assert!(newer.available);
        assert_eq!(newer.can_install, cfg!(windows));
        assert!(!info(&release("v1.0.0", repo)).available);
        let elsewhere = release(
            "v99.0.0",
            "https://github.com/someone/else/releases/download/",
        );
        assert!(files(&elsewhere, Kind::WindowsExe).is_none());
        // Les deux fichiers de la release : chaque système prend le sien.
        let mut both = release("v99.0.0", repo);
        both.assets.push(Asset {
            name: APPIMAGE_ASSET.into(),
            browser_download_url: format!("{repo}v99.0.0/{APPIMAGE_ASSET}"),
            size: 10,
        });
        assert_eq!(files(&both, Kind::WindowsExe).unwrap().0.name, EXE_ASSET);
        assert_eq!(
            files(&both, Kind::LinuxAppImage).unwrap().0.name,
            APPIMAGE_ASSET
        );
        // Une release sans AppImage (la 2.0.x) ne propose rien sous Linux.
        assert!(files(&release("v99.0.0", repo), Kind::LinuxAppImage).is_none());
        assert!(!info(&elsewhere).can_install);
        let mut old_python = release("V1.3", repo);
        old_python.assets.clear();
        assert!(!info(&old_python).can_install);
    }

    #[test]
    fn reads_published_hashes() {
        let hash = "a".repeat(64);
        let sums = format!(
            "\u{feff}{hash}  GiveawayIG.exe\r\n{}  other.zip\n",
            "b".repeat(64)
        );
        assert_eq!(expected_hash(&sums, EXE_ASSET), Some(hash.clone()));
        assert_eq!(
            expected_hash(&format!("{}  *giveawayig.exe", "C".repeat(64)), EXE_ASSET),
            Some("c".repeat(64))
        );
        assert_eq!(expected_hash("abc  GiveawayIG.exe", EXE_ASSET), None);
        assert_eq!(expected_hash(&sums, "missing.exe"), None);
    }

    #[test]
    fn verifies_downloaded_file() {
        let exe = b"MZ fake windows program".to_vec();
        let hash = sha256_hex(&exe);
        let w = Kind::WindowsExe;
        assert!(verify(&exe, w, exe.len() as u64, &hash).is_ok());
        assert!(verify(&exe, w, 0, &hash).is_ok());
        assert!(verify(&exe, w, 3, &hash).is_err());
        assert!(verify(&exe, w, exe.len() as u64, &"0".repeat(64)).is_err());
        let not_exe = b"<html>error</html>".to_vec();
        assert!(verify(&not_exe, w, 0, &sha256_hex(&not_exe)).is_err());
        // AppImage : un exécutable Linux (ELF), pas un exe Windows.
        let elf = b"\x7fELF fake appimage".to_vec();
        assert!(verify(&elf, Kind::LinuxAppImage, 0, &sha256_hex(&elf)).is_ok());
        assert!(verify(&exe, Kind::LinuxAppImage, 0, &hash).is_err());
        assert!(verify(&elf, w, 0, &sha256_hex(&elf)).is_err());
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn replaces_the_executable_and_keeps_the_old_one_aside() {
        let dir = std::env::temp_dir().join(format!("giveaway-ig-update-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("GiveawayIG.exe");
        std::fs::write(&exe, b"MZ old").unwrap();
        std::fs::write(sibling(&exe, ".old"), b"MZ older").unwrap();
        replace(&exe, b"MZ new").unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"MZ new");
        assert_eq!(std::fs::read(sibling(&exe, ".old")).unwrap(), b"MZ old");
        assert!(!sibling(&exe, ".new").exists());
        // L'exécutable n'existe pas : rien n'est cassé, rien ne traîne.
        let missing = dir.join("absent.exe");
        assert_eq!(replace(&missing, b"MZ"), Err("update-write".to_string()));
        assert!(!sibling(&missing, ".new").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
