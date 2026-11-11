//! Sauvegardes des données (réglages, concours, historique, comptes).
//!
//! Au premier lancement d'une nouvelle version, avant que les données soient lues (et
//! éventuellement converties), une copie des fichiers est rangée dans `backups/`. On peut la
//! restaurer depuis les Paramètres. Les profils de navigation (connexions) et les photos ne
//! sont pas copiés : ils ne changent pas d'une version à l'autre et peuvent être volumineux.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

/// Fichiers de données sauvegardés.
pub const FILES: [&str; 5] = [
    "settings.json",
    "giveaways.json",
    "history.json",
    "meta.json",
    "accounts.json",
];
const DIR: &str = "backups";
const INFO: &str = "backup.json";
/// Nombre de sauvegardes gardées (les plus anciennes sont supprimées).
pub const KEEP: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    /// Premier lancement d'une nouvelle version.
    Update,
    /// Juste avant de restaurer une autre sauvegarde.
    Restore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// Nom du dossier de la sauvegarde.
    #[serde(default)]
    pub id: String,
    pub created_at: DateTime<Utc>,
    /// Version qui utilisait ces données (inconnue avant la 2.1.0).
    pub from_version: Option<String>,
    /// Version lancée au moment de la sauvegarde.
    pub to_version: String,
    pub reason: Reason,
}

/// Version qui a écrit les données en dernier (`appVersion` de `meta.json`), si elles existent.
/// `Some(None)` : des données existent mais d'une version qui ne notait pas son numéro.
fn stored_version(data: &Path) -> Option<Option<String>> {
    let text = fs::read_to_string(data.join("meta.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    Some(
        value
            .get("appVersion")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    )
}

/// À appeler au démarrage, avant de lire les données : sauvegarde si la version a changé.
pub fn before_upgrade(data: &Path, current: &str) -> Option<BackupInfo> {
    let previous = stored_version(data)?;
    if previous.as_deref() == Some(current) || already_saved(data) {
        return None;
    }
    create(data, previous, current, Reason::Update, Utc::now()).ok()
}

/// Les données sont identiques à une sauvegarde existante (par exemple juste après une
/// restauration) : inutile d'en refaire une copie.
fn already_saved(data: &Path) -> bool {
    let current: Vec<Option<Vec<u8>>> = FILES.iter().map(|n| fs::read(data.join(n)).ok()).collect();
    list(data).into_iter().any(|backup| {
        let dir = data.join(DIR).join(backup.id);
        FILES
            .iter()
            .zip(&current)
            .all(|(name, mine)| fs::read(dir.join(name)).ok() == *mine)
    })
}

/// Copie les fichiers de données dans un nouveau dossier de `backups/`.
pub fn create(
    data: &Path,
    from_version: Option<String>,
    current: &str,
    reason: Reason,
    now: DateTime<Utc>,
) -> io::Result<BackupInfo> {
    let root = data.join(DIR);
    let stamp = now.with_timezone(&Local).format("%Y-%m-%d_%H-%M-%S");
    let mut id = format!("{stamp}_v{current}");
    let mut n = 2;
    while root.join(&id).exists() {
        id = format!("{stamp}_v{current}-{n}");
        n += 1;
    }
    let dir = root.join(&id);
    fs::create_dir_all(&dir)?;
    for name in FILES {
        let source = data.join(name);
        if source.is_file() {
            fs::copy(&source, dir.join(name))?;
        }
    }
    let info = BackupInfo {
        id: id.clone(),
        created_at: now,
        from_version,
        to_version: current.to_string(),
        reason,
    };
    fs::write(
        dir.join(INFO),
        serde_json::to_vec_pretty(&info).unwrap_or_default(),
    )?;
    prune(data, KEEP);
    Ok(info)
}

/// Sauvegardes existantes, la plus récente d'abord.
pub fn list(data: &Path) -> Vec<BackupInfo> {
    let Ok(entries) = fs::read_dir(data.join(DIR)) else {
        return Vec::new();
    };
    let mut list: Vec<BackupInfo> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let id = e.file_name().to_string_lossy().into_owned();
            let text = fs::read_to_string(e.path().join(INFO)).ok()?;
            let mut info: BackupInfo = serde_json::from_str(&text).ok()?;
            info.id = id;
            Some(info)
        })
        .collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
    list
}

/// Ne garde que les `keep` sauvegardes les plus récentes.
fn prune(data: &Path, keep: usize) {
    for old in list(data).into_iter().skip(keep) {
        let _ = fs::remove_dir_all(data.join(DIR).join(old.id));
    }
}

/// Dossier d'une sauvegarde existante (refuse tout nom qui sortirait de `backups/`).
fn backup_dir(data: &Path, id: &str) -> Option<PathBuf> {
    let safe = !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !id.contains("..");
    let dir = data.join(DIR).join(id);
    (safe && dir.join(INFO).is_file()).then_some(dir)
}

/// Remet les fichiers d'une sauvegarde à la place des données actuelles (sauvegardées avant).
/// Un fichier absent de la sauvegarde (créé par une version plus récente) est retiré, pour
/// retrouver exactement l'état de l'époque. Il faut redémarrer l'application juste après.
pub fn restore(data: &Path, id: &str, current: &str) -> Result<(), String> {
    let dir = backup_dir(data, id).ok_or("backup-not-found")?;
    let version = stored_version(data).flatten();
    create(data, version, current, Reason::Restore, Utc::now())
        .map_err(|_| "backup-failed".to_string())?;
    for name in FILES {
        let saved = dir.join(name);
        let target = data.join(name);
        if saved.is_file() {
            // Copie puis renommage : le fichier n'est jamais à moitié écrit.
            let staged = data.join(format!("{name}.restore"));
            fs::copy(&saved, &staged).map_err(|_| "backup-failed".to_string())?;
            fs::rename(&staged, &target).map_err(|_| "backup-failed".to_string())?;
        } else if target.is_file() {
            fs::remove_file(&target).map_err(|_| "backup-failed".to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("giveaway-ig-backup-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn backs_up_once_per_new_version() {
        let data = temp("upgrade");
        // Première installation : rien à sauvegarder.
        assert!(before_upgrade(&data, "2.1.0").is_none());
        // Données d'une version qui ne notait pas son numéro.
        fs::write(data.join("meta.json"), br#"{"lastSync":null}"#).unwrap();
        fs::write(data.join("history.json"), b"[1]").unwrap();
        let first = before_upgrade(&data, "2.1.0").unwrap();
        assert_eq!(first.from_version, None);
        assert_eq!(first.reason, Reason::Update);
        let saved = data.join(DIR).join(&first.id);
        assert_eq!(fs::read(saved.join("history.json")).unwrap(), b"[1]");
        assert!(!saved.join("accounts.json").exists());
        // Même version : pas de nouvelle copie.
        fs::write(data.join("meta.json"), br#"{"appVersion":"2.1.0"}"#).unwrap();
        assert!(before_upgrade(&data, "2.1.0").is_none());
        let next = before_upgrade(&data, "2.2.0").unwrap();
        assert_eq!(next.from_version.as_deref(), Some("2.1.0"));
        assert_eq!(list(&data).len(), 2);
        let _ = fs::remove_dir_all(&data);
    }

    #[test]
    fn keeps_the_latest_backups_only() {
        let data = temp("prune");
        fs::write(data.join("meta.json"), b"{}").unwrap();
        let start = Utc::now() - chrono::Duration::days(30);
        for day in 0..(KEEP as i64 + 3) {
            create(
                &data,
                None,
                "2.1.0",
                Reason::Update,
                start + chrono::Duration::days(day),
            )
            .unwrap();
        }
        let kept = list(&data);
        assert_eq!(kept.len(), KEEP);
        assert!(kept[0].created_at > kept[KEEP - 1].created_at);
        let _ = fs::remove_dir_all(&data);
    }

    #[test]
    fn restores_a_backup_and_keeps_the_current_state() {
        let data = temp("restore");
        fs::write(data.join("meta.json"), br#"{"appVersion":"2.0.1"}"#).unwrap();
        fs::write(data.join("history.json"), b"old").unwrap();
        let old = create(
            &data,
            Some("2.0.1".into()),
            "2.0.1",
            Reason::Update,
            Utc::now(),
        )
        .unwrap();
        fs::write(data.join("meta.json"), br#"{"appVersion":"2.1.0"}"#).unwrap();
        fs::write(data.join("history.json"), b"new").unwrap();
        fs::write(data.join("accounts.json"), b"{}").unwrap();

        assert!(restore(&data, "../outside", "2.1.0").is_err());
        assert!(restore(&data, "missing", "2.1.0").is_err());
        restore(&data, &old.id, "2.1.0").unwrap();
        assert_eq!(fs::read(data.join("history.json")).unwrap(), b"old");
        assert!(!data.join("accounts.json").exists());
        // Au redémarrage, ces données sont déjà sauvegardées : pas de copie en double.
        assert!(before_upgrade(&data, "2.1.0").is_none());
        // L'état d'avant la restauration a été mis de côté.
        let before = list(&data)
            .into_iter()
            .find(|b| b.reason == Reason::Restore)
            .unwrap();
        let dir = data.join(DIR).join(before.id);
        assert_eq!(fs::read(dir.join("history.json")).unwrap(), b"new");
        assert_eq!(before.from_version.as_deref(), Some("2.1.0"));
        let _ = fs::remove_dir_all(&data);
    }
}
