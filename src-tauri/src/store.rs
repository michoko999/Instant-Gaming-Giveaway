//! Stockage local en JSON, dans le dossier de données de l'application.
//! Chaque fichier est écrit de façon atomique (fichier temporaire puis renommage).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Local, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::*;

const SETTINGS_FILE: &str = "settings.json";
const GIVEAWAYS_FILE: &str = "giveaways.json";
const HISTORY_FILE: &str = "history.json";
const META_FILE: &str = "meta.json";
const MAX_HISTORY: usize = 20_000;

pub struct Store {
    dir: PathBuf,
    pub settings: Settings,
    pub giveaways: Vec<Giveaway>,
    pub history: Vec<HistoryEntry>,
    pub meta: Meta,
}

impl Store {
    pub fn load(dir: &Path) -> Store {
        let mut settings: Settings = read_json(&dir.join(SETTINGS_FILE));
        settings.sanitize();
        Store {
            dir: dir.to_path_buf(),
            settings,
            giveaways: read_json(&dir.join(GIVEAWAYS_FILE)),
            history: read_json(&dir.join(HISTORY_FILE)),
            meta: read_json(&dir.join(META_FILE)),
        }
    }

    #[cfg(test)]
    pub fn in_memory() -> Store {
        Store {
            dir: std::env::temp_dir().join(format!("giveaway-ig-test-{}", std::process::id())),
            settings: Settings::default(),
            giveaways: Vec::new(),
            history: Vec::new(),
            meta: Meta::default(),
        }
    }

    pub fn save_settings(&self) -> io::Result<()> {
        write_json(&self.dir.join(SETTINGS_FILE), &self.settings)
    }

    pub fn save_giveaways(&self) -> io::Result<()> {
        write_json(&self.dir.join(GIVEAWAYS_FILE), &self.giveaways)
    }

    pub fn save_history(&self) -> io::Result<()> {
        write_json(&self.dir.join(HISTORY_FILE), &self.history)
    }

    pub fn save_meta(&self) -> io::Result<()> {
        write_json(&self.dir.join(META_FILE), &self.meta)
    }

    pub fn find_mut(&mut self, code: &str) -> Option<&mut Giveaway> {
        self.giveaways
            .iter_mut()
            .find(|g| g.code.eq_ignore_ascii_case(code))
    }

    /// Ajoute les codes inconnus. Renvoie le nombre de nouveaux concours.
    pub fn merge_codes(&mut self, codes: &[(String, String)], now: DateTime<Utc>) -> usize {
        let mut index: HashMap<String, usize> = self
            .giveaways
            .iter()
            .enumerate()
            .map(|(i, g)| (g.code.to_ascii_uppercase(), i))
            .collect();
        let mut added = 0;
        for (code, source) in codes {
            let key = code.to_ascii_uppercase();
            match index.get(&key) {
                Some(&i) => {
                    let g = &mut self.giveaways[i];
                    if !g.sources.contains(source) {
                        g.sources.push(source.clone());
                    }
                }
                None => {
                    self.giveaways.push(Giveaway {
                        code: code.clone(),
                        status: Status::Unknown,
                        checked_at: None,
                        last_participation: None,
                        last_outcome: None,
                        sources: vec![source.clone()],
                        added_at: now,
                        ignored: false,
                    });
                    index.insert(key, self.giveaways.len() - 1);
                    added += 1;
                }
            }
        }
        added
    }

    /// Marque comme terminés les concours signalés morts par une source, seulement s'ils n'ont
    /// jamais été vérifiés : ce que l'application a constaté elle-même passe avant la liste
    /// (un concours revenu à la vie reste actif).
    pub fn mark_ended(&mut self, codes: &[String], now: DateTime<Utc>) -> usize {
        let dead: BTreeSet<String> = codes.iter().map(|c| c.to_ascii_uppercase()).collect();
        let mut changed = 0;
        for g in &mut self.giveaways {
            if g.status == Status::Unknown && dead.contains(&g.code.to_ascii_uppercase()) {
                g.status = Status::Ended;
                g.checked_at = Some(now);
                changed += 1;
            }
        }
        changed
    }

    pub fn set_status(&mut self, code: &str, status: Status, now: DateTime<Utc>) {
        if let Some(g) = self.find_mut(code) {
            g.status = status;
            g.checked_at = Some(now);
        }
    }

    /// Le concours a déjà reçu une participation pendant le mois civil en cours.
    fn done_this_month(g: &Giveaway, now: DateTime<Utc>) -> bool {
        match (g.last_participation, g.last_outcome) {
            (Some(at), Some(outcome)) => outcome.done_for_month() && same_month(at, now),
            _ => false,
        }
    }

    /// Codes à traiter lors d'une session, dans l'ordre de la liste.
    pub fn session_queue(&self, now: DateTime<Utc>) -> Vec<String> {
        self.giveaways
            .iter()
            .filter(|g| g.status == Status::Active && !g.ignored)
            .filter(|g| !(self.settings.skip_done_this_month && Self::done_this_month(g, now)))
            .map(|g| g.code.clone())
            .collect()
    }

    pub fn counts(&self, now: DateTime<Utc>) -> Counts {
        let mut c = Counts {
            total: self.giveaways.len(),
            ..Counts::default()
        };
        for g in &self.giveaways {
            match g.status {
                Status::Active => c.active += 1,
                Status::Ended => c.ended += 1,
                Status::Unknown => c.unknown += 1,
            }
            if g.ignored {
                c.ignored += 1;
            }
        }
        c.queue = self.session_queue(now).len();
        c
    }

    pub fn record(
        &mut self,
        code: &str,
        outcome: Outcome,
        mode: SessionMode,
        message: Option<String>,
        now: DateTime<Utc>,
    ) {
        self.history.push(HistoryEntry {
            code: code.to_string(),
            at: now,
            outcome,
            mode,
            message,
        });
        if self.history.len() > MAX_HISTORY {
            let excess = self.history.len() - MAX_HISTORY;
            self.history.drain(0..excess);
        }
        if let Some(g) = self.find_mut(code) {
            match outcome {
                Outcome::Ended => {
                    g.status = Status::Ended;
                    g.checked_at = Some(now);
                }
                Outcome::Participated | Outcome::Already | Outcome::Opened => {
                    g.last_participation = Some(now);
                    g.last_outcome = Some(outcome);
                    if outcome != Outcome::Opened {
                        g.status = Status::Active;
                    }
                }
                Outcome::Captcha | Outcome::Error | Outcome::Login => {
                    // Ne pas effacer une participation déjà réussie ce mois-ci.
                    let already_done = g.last_outcome.map(|o| o.done_for_month()).unwrap_or(false)
                        && g.last_participation
                            .map(|at| same_month(at, now))
                            .unwrap_or(false);
                    if !already_done {
                        g.last_outcome = Some(outcome);
                    }
                }
                Outcome::Skipped => {}
            }
        }
    }

    pub fn stats(&self, now: DateTime<Utc>) -> Stats {
        let mut stats = Stats::default();
        let mut done: HashSet<String> = HashSet::new();
        let months = last_months(now, 12);
        let mut monthly = vec![0usize; months.len()];
        for h in &self.history {
            let this_month = same_month(h.at, now);
            if h.outcome.counts_as_participation() {
                stats.total_participations += 1;
                if this_month {
                    stats.this_month += 1;
                }
                let local = h.at.with_timezone(&Local);
                if let Some(i) = months
                    .iter()
                    .position(|&m| m == (local.year(), local.month()))
                {
                    monthly[i] += 1;
                }
            }
            if this_month && h.outcome.done_for_month() {
                done.insert(h.code.to_ascii_uppercase());
            }
        }
        stats.done_this_month = done.len();
        stats.monthly = months
            .iter()
            .zip(monthly)
            .map(|(&(year, month), count)| MonthCount {
                month: format!("{year:04}-{month:02}"),
                count,
            })
            .collect();
        stats
    }
}

/// Les `n` derniers mois civils (heure locale), du plus ancien au mois en cours.
fn last_months(now: DateTime<Utc>, n: u32) -> Vec<(i32, u32)> {
    let local = now.with_timezone(&Local);
    let current = local.year() * 12 + local.month0() as i32;
    (0..n as i32)
        .rev()
        .map(|back| {
            let index = current - back;
            (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
        })
        .collect()
}

/// Deux dates tombent-elles dans le même mois civil (heure locale) ?
pub fn same_month(a: DateTime<Utc>, b: DateTime<Utc>) -> bool {
    let (a, b) = (a.with_timezone(&Local), b.with_timezone(&Local));
    a.year() == b.year() && a.month() == b.month()
}

fn read_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(_) => {
                // Fichier abîmé : on le garde de côté plutôt que de l'écraser.
                let backup = path.with_extension("json.bak");
                let _ = fs::rename(path, backup);
                T::default()
            }
        },
        Err(_) => T::default(),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)
}

/// Échappe un champ CSV (guillemets si nécessaire).
pub fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r', ';']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn merge_deduplicates_case_insensitively() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        let added = store.merge_codes(
            &[
                ("TOMBIE".into(), "a".into()),
                ("tombie".into(), "b".into()),
                ("PIVI".into(), "a".into()),
            ],
            now,
        );
        assert_eq!(added, 2);
        assert_eq!(store.giveaways.len(), 2);
        assert_eq!(
            store.giveaways[0].sources,
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn dead_list_never_overrides_a_checked_status() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(
            &[("REVIVED".into(), "a".into()), ("NEW".into(), "a".into())],
            now,
        );
        store.set_status("REVIVED", Status::Active, now);
        let changed = store.mark_ended(&["revived".into(), "new".into()], now);
        assert_eq!(changed, 1);
        assert_eq!(store.giveaways[0].status, Status::Active);
        assert_eq!(store.giveaways[1].status, Status::Ended);
    }

    #[test]
    fn queue_skips_ignored_ended_and_done_this_month() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(
            &[
                ("A".into(), "s".into()),
                ("B".into(), "s".into()),
                ("C".into(), "s".into()),
                ("D".into(), "s".into()),
            ],
            now,
        );
        for code in ["A", "B", "C", "D"] {
            store.set_status(code, Status::Active, now);
        }
        store.set_status("B", Status::Ended, now);
        store.find_mut("C").unwrap().ignored = true;
        store.record("D", Outcome::Participated, SessionMode::Auto, None, now);
        assert_eq!(store.session_queue(now), vec!["A".to_string()]);
        store.settings.skip_done_this_month = false;
        assert_eq!(
            store.session_queue(now),
            vec!["A".to_string(), "D".to_string()]
        );
    }

    #[test]
    fn participation_from_last_month_is_queued_again() {
        let mut store = Store::in_memory();
        let now = Local
            .with_ymd_and_hms(2026, 11, 11, 20, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let last_month = Local
            .with_ymd_and_hms(2026, 10, 28, 20, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        store.merge_codes(&[("A".into(), "s".into())], last_month);
        store.set_status("A", Status::Active, last_month);
        store.record(
            "A",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            last_month,
        );
        assert_eq!(store.session_queue(now), vec!["A".to_string()]);
        assert!(store.session_queue(last_month).is_empty());
    }

    #[test]
    fn opened_page_is_not_counted_as_done() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(&[("A".into(), "s".into())], now);
        store.set_status("A", Status::Active, now);
        store.record("A", Outcome::Opened, SessionMode::Manual, None, now);
        assert_eq!(store.session_queue(now), vec!["A".to_string()]);
    }

    #[test]
    fn error_does_not_erase_a_success_this_month() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(&[("A".into(), "s".into())], now);
        store.set_status("A", Status::Active, now);
        store.record("A", Outcome::Participated, SessionMode::Auto, None, now);
        store.record("A", Outcome::Error, SessionMode::Auto, None, now);
        assert!(store.session_queue(now).is_empty());
    }

    #[test]
    fn monthly_stats() {
        let mut store = Store::in_memory();
        let now = Local
            .with_ymd_and_hms(2026, 11, 11, 20, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let last_month = Local
            .with_ymd_and_hms(2026, 10, 2, 20, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        store.record("A", Outcome::Participated, SessionMode::Auto, None, now);
        store.record("B", Outcome::Already, SessionMode::Auto, None, now);
        store.record("C", Outcome::Opened, SessionMode::Manual, None, now);
        store.record(
            "D",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            last_month,
        );
        let stats = store.stats(now);
        assert_eq!(stats.total_participations, 3);
        assert_eq!(stats.this_month, 2);
        assert_eq!(stats.done_this_month, 2);
        assert_eq!(stats.monthly.len(), 12);
        assert_eq!(
            stats.monthly[11],
            MonthCount {
                month: "2026-11".into(),
                count: 2
            }
        );
        assert_eq!(
            stats.monthly[10],
            MonthCount {
                month: "2026-10".into(),
                count: 1
            }
        );
        assert_eq!(stats.monthly[0].month, "2025-12");
    }

    #[test]
    fn last_months_cross_the_year() {
        let january = Local
            .with_ymd_and_hms(2027, 1, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let months = last_months(january, 3);
        assert_eq!(months, vec![(2026, 11), (2026, 12), (2027, 1)]);
    }

    #[test]
    fn csv_escaping() {
        assert_eq!(csv_field("abc"), "abc");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
