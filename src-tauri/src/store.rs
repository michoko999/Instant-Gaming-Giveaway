//! Stockage local en JSON, dans le dossier de données de l'application.
//! Chaque fichier est écrit de façon atomique (fichier temporaire puis renommage).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use base64::Engine;
use chrono::{DateTime, Datelike, Local, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::model::*;

const SETTINGS_FILE: &str = "settings.json";
const GIVEAWAYS_FILE: &str = "giveaways.json";
const HISTORY_FILE: &str = "history.json";
const META_FILE: &str = "meta.json";
const ACCOUNTS_FILE: &str = "accounts.json";
/// Dossier des photos de profil, dans le dossier de données.
pub const AVATARS_DIR: &str = "avatars";
const MAX_HISTORY: usize = 20_000;

pub struct Store {
    dir: PathBuf,
    pub settings: Settings,
    pub giveaways: Vec<Giveaway>,
    pub history: Vec<HistoryEntry>,
    pub meta: Meta,
    pub accounts: Accounts,
    /// Photos de profil déjà converties en `data:` (identifiant du compte → image).
    avatar_cache: HashMap<String, Option<String>>,
}

impl Store {
    pub fn load(dir: &Path) -> Store {
        let mut settings: Settings = read_json(&dir.join(SETTINGS_FILE));
        settings.sanitize();
        let accounts_path = dir.join(ACCOUNTS_FILE);
        let first_run_with_accounts = !accounts_path.exists();
        let mut accounts: Accounts = read_json(&accounts_path);
        if first_run_with_accounts {
            // Version à un seul compte : il devient le premier compte, avec son profil d'origine.
            if let Some(main) = accounts.get_mut(MAIN_ACCOUNT) {
                main.connected = settings.account_connected;
            }
        }
        accounts.sanitize();
        let mut store = Store {
            dir: dir.to_path_buf(),
            settings,
            giveaways: read_json(&dir.join(GIVEAWAYS_FILE)),
            history: read_json(&dir.join(HISTORY_FILE)),
            meta: read_json(&dir.join(META_FILE)),
            accounts,
            avatar_cache: HashMap::new(),
        };
        store.migrate_marks();
        if store.repair_glued_codes() {
            let _ = store.save_giveaways();
            let _ = store.save_history();
        }
        if first_run_with_accounts {
            let _ = store.save_accounts();
        }
        store
    }

    /// Reprend les anciens champs « dernière participation » dans les marques du premier compte.
    fn migrate_marks(&mut self) {
        for g in &mut self.giveaways {
            if g.last_participation.is_some() || g.last_outcome.is_some() {
                g.marks.entry(MAIN_ACCOUNT.to_string()).or_insert(Mark {
                    participated_at: g.last_participation,
                    outcome: g.last_outcome,
                });
                g.last_participation = None;
                g.last_outcome = None;
            }
        }
    }

    /// Répare les codes abîmés par deux liens collés dans une liste (« retroraiderhttps ») :
    /// ils sont fusionnés avec le bon code, ou renommés s'il n'existe pas encore.
    /// Renvoie `true` si quelque chose a changé.
    fn repair_glued_codes(&mut self) -> bool {
        let broken: Vec<(String, String)> = self
            .giveaways
            .iter()
            .filter_map(|g| {
                crate::sources::glued_prefix(&g.code)
                    .map(|clean| (g.code.clone(), clean.to_string()))
            })
            .collect();
        if broken.is_empty() {
            return false;
        }
        for (bad, clean) in &broken {
            let Some(pos) = self.giveaways.iter().position(|g| &g.code == bad) else {
                continue;
            };
            let target = self
                .giveaways
                .iter()
                .position(|g| g.code.eq_ignore_ascii_case(clean));
            match target {
                Some(t) => {
                    let old = self.giveaways.remove(pos);
                    let t = if t > pos { t - 1 } else { t };
                    let good = &mut self.giveaways[t];
                    for (account, mark) in old.marks {
                        let keep = good
                            .marks
                            .get(&account)
                            .is_some_and(|m| m.participated_at >= mark.participated_at);
                        if !keep {
                            good.marks.insert(account, mark);
                        }
                    }
                    for s in old.sources {
                        if !good.sources.contains(&s) {
                            good.sources.push(s);
                        }
                    }
                    good.added_at = good.added_at.min(old.added_at);
                }
                None => self.giveaways[pos].code = clean.clone(),
            }
            let fixed = self
                .giveaways
                .iter()
                .find(|g| g.code.eq_ignore_ascii_case(clean))
                .map(|g| g.code.clone())
                .unwrap_or_else(|| clean.clone());
            for h in self.history.iter_mut().filter(|h| &h.code == bad) {
                h.code = fixed.clone();
            }
        }
        true
    }

    #[cfg(test)]
    pub fn in_memory() -> Store {
        Store {
            dir: std::env::temp_dir().join(format!("giveaway-ig-test-{}", std::process::id())),
            settings: Settings::default(),
            giveaways: Vec::new(),
            history: Vec::new(),
            meta: Meta::default(),
            accounts: Accounts::default(),
            avatar_cache: HashMap::new(),
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

    pub fn save_accounts(&self) -> io::Result<()> {
        write_json(&self.dir.join(ACCOUNTS_FILE), &self.accounts)
    }

    pub fn active_account(&self) -> String {
        self.accounts.active().id.clone()
    }

    /// Oublie la photo de profil en mémoire (elle vient de changer).
    pub fn forget_avatar(&mut self, id: &str) {
        self.avatar_cache.remove(id);
    }

    /// Photo de profil du compte, en `data:` pour l'interface.
    fn avatar_data(&mut self, account: &Account) -> Option<String> {
        if let Some(cached) = self.avatar_cache.get(&account.id) {
            return cached.clone();
        }
        let data = account.avatar_file.as_ref().and_then(|file| {
            let bytes = fs::read(self.dir.join(AVATARS_DIR).join(file)).ok()?;
            Some(format!(
                "data:{};base64,{}",
                image_mime(file),
                base64::engine::general_purpose::STANDARD.encode(bytes)
            ))
        });
        self.avatar_cache.insert(account.id.clone(), data.clone());
        data
    }

    pub fn account_views(&mut self, now: DateTime<Utc>) -> Vec<AccountView> {
        let active = self.accounts.active().id.clone();
        let list = self.accounts.list.clone();
        list.iter()
            .map(|a| AccountView {
                id: a.id.clone(),
                name: a.name.clone(),
                avatar: self.avatar_data(a),
                profile_url: a.profile.as_ref().map(|p| {
                    format!(
                        "https://www.instant-gaming.com/{}/user/{}",
                        self.settings.language, p
                    )
                }),
                connected: a.connected,
                active: a.id == active,
                done_this_month: self.done_count(&a.id, now),
                queue: self.session_queue_for(&a.id, now).len(),
            })
            .collect()
    }

    /// Concours différents faits ce mois-ci par un compte.
    fn done_count(&self, account: &str, now: DateTime<Utc>) -> usize {
        self.history
            .iter()
            .filter(|h| {
                h.account_id() == account && h.outcome.done_for_month() && same_month(h.at, now)
            })
            .map(|h| h.code.to_ascii_uppercase())
            .collect::<HashSet<_>>()
            .len()
    }

    /// Supprime tout ce qui concerne un compte (marques et historique).
    pub fn forget_account_data(&mut self, account: &str) {
        self.history.retain(|h| h.account_id() != account);
        for g in &mut self.giveaways {
            g.marks.remove(account);
        }
        self.avatar_cache.remove(account);
    }

    /// Concours vus par le compte actif.
    pub fn giveaway_views(&self) -> Vec<GiveawayView> {
        let active = self.accounts.active().id.clone();
        self.giveaways
            .iter()
            .map(|g| {
                let mark = g.marks.get(&active);
                GiveawayView {
                    code: g.code.clone(),
                    status: g.status,
                    checked_at: g.checked_at,
                    last_participation: mark.and_then(|m| m.participated_at),
                    last_outcome: mark.and_then(|m| m.outcome),
                    sources: g.sources.clone(),
                    added_at: g.added_at,
                    ignored: g.ignored,
                }
            })
            .collect()
    }

    /// Historique du compte actif, du plus récent au plus ancien.
    pub fn active_history(&self, limit: usize) -> Vec<HistoryEntry> {
        let active = self.accounts.active().id.clone();
        self.history
            .iter()
            .rev()
            .filter(|h| h.account_id() == active)
            .take(limit)
            .cloned()
            .collect()
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
                        marks: Default::default(),
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

    /// Le compte a déjà participé à ce concours pendant le mois civil en cours.
    fn done_this_month(g: &Giveaway, account: &str, now: DateTime<Utc>) -> bool {
        match g.marks.get(account) {
            Some(Mark {
                participated_at: Some(at),
                outcome: Some(outcome),
            }) => outcome.done_for_month() && same_month(*at, now),
            _ => false,
        }
    }

    /// Codes à traiter lors d'une session du compte actif, dans l'ordre de la liste.
    pub fn session_queue(&self, now: DateTime<Utc>) -> Vec<String> {
        let account = self.accounts.active().id.clone();
        self.session_queue_for(&account, now)
    }

    /// Codes à traiter lors d'une session d'un compte donné.
    pub fn session_queue_for(&self, account: &str, now: DateTime<Utc>) -> Vec<String> {
        self.giveaways
            .iter()
            .filter(|g| g.status == Status::Active && !g.ignored)
            .filter(|g| {
                !(self.settings.skip_done_this_month && Self::done_this_month(g, account, now))
            })
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
        if let Some(seen) = self.meta.giveaways_seen_at {
            c.fresh = self
                .giveaways
                .iter()
                .filter(|g| g.status == Status::Active && !g.ignored && g.added_at > seen)
                .count();
        }
        c
    }

    /// Repère les comptes de l'application parmi les gagnants d'un concours.
    /// Renvoie les gains qui n'étaient pas encore connus.
    pub fn record_wins(
        &mut self,
        code: &str,
        winners: &[crate::checker::Winner],
        now: DateTime<Utc>,
    ) -> Vec<Win> {
        let mut found = Vec::new();
        for account in &self.accounts.list {
            for w in winners.iter().filter(|w| winner_is(account, w)) {
                let known = self.meta.wins.iter().any(|x| {
                    x.account == account.id
                        && x.code.eq_ignore_ascii_case(code)
                        && x.product_url == w.product_url
                });
                if !known {
                    found.push(Win {
                        account: account.id.clone(),
                        code: code.to_string(),
                        product: w.product_url.as_deref().and_then(product_name),
                        product_url: w.product_url.clone(),
                        detected_at: now,
                        seen: false,
                    });
                }
            }
        }
        self.meta.wins.extend(found.iter().cloned());
        found
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        account: &str,
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
            account: Some(account.to_string()),
        });
        if self.history.len() > MAX_HISTORY {
            let excess = self.history.len() - MAX_HISTORY;
            self.history.drain(0..excess);
        }
        if let Some(g) = self.find_mut(code) {
            if outcome == Outcome::Ended {
                g.status = Status::Ended;
                g.checked_at = Some(now);
                return;
            }
            let mark = g.marks.entry(account.to_string()).or_default();
            match outcome {
                Outcome::Participated | Outcome::Already | Outcome::Opened => {
                    mark.participated_at = Some(now);
                    mark.outcome = Some(outcome);
                    if outcome != Outcome::Opened {
                        g.status = Status::Active;
                    }
                }
                Outcome::Captcha | Outcome::Error | Outcome::Login => {
                    // Ne pas effacer une participation déjà réussie ce mois-ci.
                    let already_done = mark.outcome.map(|o| o.done_for_month()).unwrap_or(false)
                        && mark
                            .participated_at
                            .map(|at| same_month(at, now))
                            .unwrap_or(false);
                    if !already_done {
                        mark.outcome = Some(outcome);
                    }
                }
                Outcome::Ended | Outcome::Skipped => {}
            }
        }
    }

    /// Statistiques du compte actif.
    pub fn stats(&self, now: DateTime<Utc>) -> Stats {
        let mut stats = Stats::default();
        let mut done: HashSet<String> = HashSet::new();
        let months = last_months(now, 12);
        let mut monthly = vec![0usize; months.len()];
        let active = self.accounts.active().id.clone();
        for h in self.history.iter().filter(|h| h.account_id() == active) {
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

/// Le gagnant est-il ce compte ? Par l'identifiant de profil s'il est connu, sinon par le pseudo.
fn winner_is(account: &Account, winner: &crate::checker::Winner) -> bool {
    match (&account.profile, &winner.profile) {
        (Some(mine), Some(theirs)) => mine.eq_ignore_ascii_case(theirs),
        (Some(_), None) | (None, _) => account
            .name
            .as_deref()
            .map(|name| !name.is_empty() && name.eq_ignore_ascii_case(&winner.nickname))
            .unwrap_or(false),
    }
}

/// Nom lisible d'un jeu à partir de l'adresse de sa page :
/// `…/fr/23617-acheter-phantom-blade-zero-digital-deluxe-edition-pc-steam/` → « Phantom Blade Zero Digital Deluxe Edition ».
pub fn product_name(url: &str) -> Option<String> {
    const VERBS: [&str; 8] = [
        "acheter",
        "buy",
        "comprar",
        "kaufen",
        "comprare",
        "acquistare",
        "kup",
        "kupic",
    ];
    const PLATFORMS: [&str; 16] = [
        "pc",
        "mac",
        "steam",
        "epic",
        "gog",
        "origin",
        "ea",
        "ubisoft",
        "connect",
        "rockstar",
        "xbox",
        "playstation",
        "ps4",
        "ps5",
        "switch",
        "eshop",
    ];
    let slug = url.trim_end_matches('/').rsplit('/').next()?;
    let mut words: Vec<&str> = slug.split('-').filter(|w| !w.is_empty()).collect();
    if words
        .first()
        .map(|w| w.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
    {
        words.remove(0);
    }
    if words.first().map(|w| VERBS.contains(w)).unwrap_or(false) {
        words.remove(0);
    }
    while words.len() > 1 && words.last().map(|w| PLATFORMS.contains(w)).unwrap_or(false) {
        words.pop();
    }
    if words.is_empty() {
        return None;
    }
    let name = words
        .iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(name)
}

fn image_mime(file: &str) -> &'static str {
    match file
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/jpeg",
    }
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
        store.record(
            MAIN_ACCOUNT,
            "D",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
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
            MAIN_ACCOUNT,
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
        store.record(
            MAIN_ACCOUNT,
            "A",
            Outcome::Opened,
            SessionMode::Manual,
            None,
            now,
        );
        assert_eq!(store.session_queue(now), vec!["A".to_string()]);
    }

    #[test]
    fn error_does_not_erase_a_success_this_month() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.merge_codes(&[("A".into(), "s".into())], now);
        store.set_status("A", Status::Active, now);
        store.record(
            MAIN_ACCOUNT,
            "A",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
        store.record(
            MAIN_ACCOUNT,
            "A",
            Outcome::Error,
            SessionMode::Auto,
            None,
            now,
        );
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
        store.record(
            MAIN_ACCOUNT,
            "A",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
        store.record(
            MAIN_ACCOUNT,
            "B",
            Outcome::Already,
            SessionMode::Auto,
            None,
            now,
        );
        store.record(
            MAIN_ACCOUNT,
            "C",
            Outcome::Opened,
            SessionMode::Manual,
            None,
            now,
        );
        store.record(
            MAIN_ACCOUNT,
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
    fn each_account_has_its_own_month() {
        let mut store = Store::in_memory();
        let now = Utc::now();
        store.accounts.list.push(Account::new("second", now));
        store.merge_codes(&[("A".into(), "s".into()), ("B".into(), "s".into())], now);
        store.set_status("A", Status::Active, now);
        store.set_status("B", Status::Active, now);
        store.record(
            MAIN_ACCOUNT,
            "A",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
        assert_eq!(store.session_queue(now), vec!["B".to_string()]);
        assert_eq!(store.stats(now).done_this_month, 1);

        store.accounts.active = "second".into();
        assert_eq!(
            store.session_queue(now),
            vec!["A".to_string(), "B".to_string()]
        );
        assert_eq!(store.stats(now).done_this_month, 0);
        assert!(store.active_history(10).is_empty());
        assert!(store
            .giveaway_views()
            .iter()
            .all(|g| g.last_participation.is_none()));
        store.record(
            "second",
            "B",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );

        let views = store.account_views(now);
        assert_eq!(
            views.iter().map(|v| v.done_this_month).collect::<Vec<_>>(),
            vec![1, 1]
        );
        assert!(views[1].active && !views[0].active);

        store.forget_account_data("second");
        assert_eq!(
            store.session_queue(now),
            vec!["A".to_string(), "B".to_string()]
        );
        store.accounts.active = MAIN_ACCOUNT.into();
        assert_eq!(store.session_queue(now), vec!["B".to_string()]);
    }

    #[test]
    fn single_account_data_moves_to_the_first_account() {
        let dir = std::env::temp_dir().join(format!("giveaway-ig-migrate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(SETTINGS_FILE), r#"{"accountConnected":true}"#).unwrap();
        fs::write(
            dir.join(GIVEAWAYS_FILE),
            r#"[{"code":"A","status":"active","lastParticipation":"2026-11-02T10:00:00Z","lastOutcome":"participated","addedAt":"2026-11-01T10:00:00Z"}]"#,
        )
        .unwrap();
        fs::write(
            dir.join(HISTORY_FILE),
            r#"[{"code":"A","at":"2026-11-02T10:00:00Z","outcome":"participated","mode":"auto"}]"#,
        )
        .unwrap();
        let store = Store::load(&dir);
        assert!(store.accounts.active().connected);
        assert_eq!(store.accounts.active().id, MAIN_ACCOUNT);
        assert_eq!(
            store.giveaways[0].marks[MAIN_ACCOUNT].outcome,
            Some(Outcome::Participated)
        );
        assert_eq!(store.active_history(5).len(), 1);
        assert!(dir.join(ACCOUNTS_FILE).exists());
        // Les anciens champs ne sont plus écrits.
        store.save_giveaways().unwrap();
        let saved = fs::read_to_string(dir.join(GIVEAWAYS_FILE)).unwrap();
        assert!(!saved.contains("lastParticipation") && saved.contains("marks"));
        let settings = fs::read_to_string(dir.join(SETTINGS_FILE)).unwrap();
        assert!(settings.contains("accountConnected"));
        store.save_settings().unwrap();
        let settings = fs::read_to_string(dir.join(SETTINGS_FILE)).unwrap();
        assert!(!settings.contains("accountConnected"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn finds_our_accounts_among_winners() {
        use crate::checker::Winner;
        let mut store = Store::in_memory();
        let now = Utc::now();
        {
            let main = store.accounts.get_mut(MAIN_ACCOUNT).unwrap();
            main.profile = Some("gamer-33ecea".into());
            main.name = Some("Michoko".into());
        }
        let mut second = Account::new("second", now);
        second.name = Some("Faerune".into());
        store.accounts.list.push(second);
        store.accounts.list.push(Account::new("unknown", now));
        let winners = vec![
            Winner {
                nickname: "Someone".into(),
                profile: Some("GAMER-33ECEA".into()),
                product_url: Some("https://www.instant-gaming.com/fr/23617-acheter-phantom-blade-zero-digital-deluxe-edition-pc-steam/".into()),
            },
            Winner { nickname: "faerune".into(), profile: Some("gamer-91bf31".into()), product_url: None },
            Winner { nickname: "Michoko".into(), profile: Some("someone-else".into()), product_url: None },
        ];
        let wins = store.record_wins("SUPREMELEADER", &winners, now);
        assert_eq!(wins.len(), 2);
        assert_eq!(wins[0].account, MAIN_ACCOUNT);
        assert_eq!(
            wins[0].product.as_deref(),
            Some("Phantom Blade Zero Digital Deluxe Edition")
        );
        assert_eq!(wins[1].account, "second");
        // Déjà connus : rien de nouveau.
        assert!(store.record_wins("supremeleader", &winners, now).is_empty());
        assert_eq!(store.meta.wins.len(), 2);
    }

    #[test]
    fn readable_game_names() {
        assert_eq!(
            product_name(
                "https://www.instant-gaming.com/en/15628-buy-the-blood-of-dawnwalker-pc-steam/"
            )
            .as_deref(),
            Some("The Blood Of Dawnwalker")
        );
        assert_eq!(
            product_name(
                "https://www.instant-gaming.com/de/9-kaufen-elden-ring-xbox-series-x-s-xbox-one/"
            )
            .as_deref(),
            Some("Elden Ring Xbox Series X S Xbox One")
        );
        assert_eq!(
            product_name("https://www.instant-gaming.com/fr/1-acheter-pc/").as_deref(),
            Some("Pc")
        );
        assert_eq!(
            product_name("https://www.instant-gaming.com/fr/"),
            Some("Fr".into())
        );
    }

    #[test]
    fn new_giveaways_since_last_visit() {
        let mut store = Store::in_memory();
        let earlier = Utc::now() - chrono::Duration::days(2);
        let now = Utc::now();
        store.merge_codes(&[("OLD".into(), "s".into())], earlier);
        store.merge_codes(&[("NEW".into(), "s".into())], now);
        store.set_status("OLD", Status::Active, now);
        store.set_status("NEW", Status::Active, now);
        assert_eq!(store.counts(now).fresh, 0);
        store.meta.giveaways_seen_at = Some(earlier + chrono::Duration::hours(1));
        assert_eq!(store.counts(now).fresh, 1);
    }

    #[test]
    fn repairs_codes_glued_to_the_next_link() {
        let mut store = Store::in_memory();
        let earlier = Utc::now() - chrono::Duration::days(40);
        let now = Utc::now();
        store.merge_codes(
            &[
                ("retroraiderhttps".into(), "a".into()),
                ("HIGHSTYLEDhttps".into(), "a".into()),
                ("HIGHSTYLED".into(), "b".into()),
            ],
            earlier,
        );
        store.record(
            MAIN_ACCOUNT,
            "HIGHSTYLEDhttps",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
        store.record(
            MAIN_ACCOUNT,
            "retroraiderhttps",
            Outcome::Participated,
            SessionMode::Auto,
            None,
            now,
        );
        assert!(store.repair_glued_codes());
        let mut codes: Vec<&str> = store.giveaways.iter().map(|g| g.code.as_str()).collect();
        codes.sort_unstable();
        assert_eq!(codes, vec!["HIGHSTYLED", "retroraider"]);
        let merged = store
            .giveaways
            .iter()
            .find(|g| g.code == "HIGHSTYLED")
            .unwrap();
        assert_eq!(merged.sources, vec!["b".to_string(), "a".to_string()]);
        assert!(merged.marks[MAIN_ACCOUNT].participated_at.is_some());
        assert!(store.history.iter().all(|h| !h.code.ends_with("https")));
        assert!(!store.repair_glued_codes());
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
