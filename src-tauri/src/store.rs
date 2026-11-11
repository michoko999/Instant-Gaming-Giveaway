//! Stockage local en JSON, dans le dossier de données de l'application.
//! Chaque fichier est écrit de façon atomique (fichier temporaire puis renommage).

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDate, Utc};
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
        self.giveaways.iter_mut().find(|g| g.code.eq_ignore_ascii_case(code))
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

    /// Marque comme terminés les concours signalés morts par une source.
    pub fn mark_ended(&mut self, codes: &[String], now: DateTime<Utc>) -> usize {
        let dead: BTreeSet<String> = codes.iter().map(|c| c.to_ascii_uppercase()).collect();
        let mut changed = 0;
        for g in &mut self.giveaways {
            if g.status != Status::Ended && dead.contains(&g.code.to_ascii_uppercase()) {
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

    fn done_today(g: &Giveaway, today: NaiveDate) -> bool {
        match (g.last_participation, g.last_outcome) {
            (Some(at), Some(outcome)) => {
                outcome.done_for_today() && at.with_timezone(&Local).date_naive() == today
            }
            _ => false,
        }
    }

    /// Codes à traiter lors d'une session, dans l'ordre de la liste.
    pub fn session_queue(&self, now: DateTime<Utc>) -> Vec<String> {
        let today = now.with_timezone(&Local).date_naive();
        self.giveaways
            .iter()
            .filter(|g| g.status == Status::Active && !g.ignored)
            .filter(|g| !(self.settings.skip_done_today && Self::done_today(g, today)))
            .map(|g| g.code.clone())
            .collect()
    }

    pub fn counts(&self, now: DateTime<Utc>) -> Counts {
        let mut c = Counts { total: self.giveaways.len(), ..Counts::default() };
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
        self.history.push(HistoryEntry { code: code.to_string(), at: now, outcome, mode, message });
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
                    g.last_outcome = Some(outcome);
                }
                Outcome::Skipped => {}
            }
        }
    }

    pub fn stats(&self, now: DateTime<Utc>) -> Stats {
        let today = now.with_timezone(&Local).date_naive();
        let mut days: BTreeSet<NaiveDate> = BTreeSet::new();
        let mut stats = Stats::default();
        for h in &self.history {
            if h.outcome.counts_as_participation() {
                stats.total_participations += 1;
                let day = h.at.with_timezone(&Local).date_naive();
                if day == today {
                    stats.today += 1;
                }
                days.insert(day);
            }
        }
        stats.streak_days = current_streak(&days, today);
        stats.best_streak = best_streak(&days);
        stats
    }
}

/// Série en cours : jours consécutifs avec au moins une participation,
/// en partant d'aujourd'hui (ou d'hier si rien n'a encore été fait aujourd'hui).
pub fn current_streak(days: &BTreeSet<NaiveDate>, today: NaiveDate) -> u32 {
    let mut day = if days.contains(&today) {
        today
    } else {
        match today.pred_opt() {
            Some(y) if days.contains(&y) => y,
            _ => return 0,
        }
    };
    let mut streak = 0;
    while days.contains(&day) {
        streak += 1;
        match day.pred_opt() {
            Some(prev) => day = prev,
            None => break,
        }
    }
    streak
}

pub fn best_streak(days: &BTreeSet<NaiveDate>) -> u32 {
    let mut best = 0;
    let mut run = 0;
    let mut previous: Option<NaiveDate> = None;
    for &day in days {
        run = match previous {
            Some(p) if p.succ_opt() == Some(day) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        previous = Some(day);
    }
    best
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
    use chrono::{Duration, TimeZone};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn merge_deduplicates_case_insensitively() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        let added = store.merge_codes(
            &[("TOMBIE".into(), "a".into()), ("tombie".into(), "b".into()), ("PIVI".into(), "a".into())],
            now,
        );
        assert_eq!(added, 2);
        assert_eq!(store.giveaways.len(), 2);
        assert_eq!(store.giveaways[0].sources, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn queue_skips_ignored_ended_and_done_today() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(
            &[("A".into(), "s".into()), ("B".into(), "s".into()), ("C".into(), "s".into()), ("D".into(), "s".into())],
            now,
        );
        for code in ["A", "B", "C", "D"] {
            store.set_status(code, Status::Active, now);
        }
        store.set_status("B", Status::Ended, now);
        store.find_mut("C").unwrap().ignored = true;
        store.record("D", Outcome::Participated, SessionMode::Auto, None, now);
        assert_eq!(store.session_queue(now), vec!["A".to_string()]);
        store.settings.skip_done_today = false;
        assert_eq!(store.session_queue(now), vec!["A".to_string(), "D".to_string()]);
    }

    #[test]
    fn streaks() {
        let days: BTreeSet<NaiveDate> =
            [day(2026, 11, 1), day(2026, 11, 2), day(2026, 11, 3), day(2026, 11, 7), day(2026, 11, 8)]
                .into_iter()
                .collect();
        assert_eq!(current_streak(&days, day(2026, 11, 8)), 2);
        assert_eq!(current_streak(&days, day(2026, 11, 9)), 2);
        assert_eq!(current_streak(&days, day(2026, 11, 10)), 0);
        assert_eq!(best_streak(&days), 3);
    }

    #[test]
    fn stats_count_participations() {
        let mut store = Store::in_memory();
        let now = Local.with_ymd_and_hms(2026, 11, 11, 20, 0, 0).unwrap().with_timezone(&Utc);
        store.record("A", Outcome::Participated, SessionMode::Auto, None, now);
        store.record("B", Outcome::Already, SessionMode::Auto, None, now);
        store.record("C", Outcome::Opened, SessionMode::Manual, None, now - Duration::days(1));
        let stats = store.stats(now);
        assert_eq!(stats.total_participations, 2);
        assert_eq!(stats.today, 1);
        assert_eq!(stats.streak_days, 2);
    }

    #[test]
    fn csv_escaping() {
        assert_eq!(csv_field("abc"), "abc");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
