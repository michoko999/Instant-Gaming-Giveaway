//! Types de données partagés entre le cœur de l'application et l'interface.
//! Tous les champs sont sérialisés en camelCase pour le TypeScript.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Langues prises en charge (interface et pages Instant Gaming).
pub const LANGUAGES: [&str; 7] = ["fr", "en", "es", "de", "pt", "it", "pl"];

pub const DEFAULT_ACCENT: &str = "#FFD27A";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Unknown,
    Active,
    Ended,
}

/// Résultat d'un passage sur un concours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Participation enregistrée par le site.
    Participated,
    /// Le compte participait déjà.
    Already,
    /// Page ouverte en mode manuel, sans confirmation de participation.
    Opened,
    /// Le site demande une vérification humaine.
    Captcha,
    /// Le compte n'est pas connecté.
    Login,
    /// Le concours est terminé.
    Ended,
    /// Passé par l'utilisateur.
    Skipped,
    /// Erreur technique (bouton introuvable, délai dépassé...).
    Error,
}

impl Outcome {
    /// Compte comme une participation pour les statistiques.
    pub fn counts_as_participation(self) -> bool {
        matches!(self, Outcome::Participated | Outcome::Opened)
    }

    /// Le concours est « fait » pour aujourd'hui.
    pub fn done_for_today(self) -> bool {
        matches!(self, Outcome::Participated | Outcome::Already | Outcome::Opened)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    /// JSON de la forme { "alive": [...], "dead": [...] }.
    AliveJson,
    /// Liste d'URL ou de codes, une par ligne (CSV ou texte).
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub kind: SourceKind,
    pub enabled: bool,
    #[serde(default)]
    pub builtin: bool,
}

pub fn builtin_sources() -> Vec<Source> {
    vec![
        Source {
            id: "enzomtpyt".into(),
            name: "enzomtpYT/InstantGamingGiveawayList".into(),
            url: "https://raw.githubusercontent.com/enzomtpYT/InstantGamingGiveawayList/refs/heads/main/json.json".into(),
            kind: SourceKind::AliveJson,
            enabled: true,
            builtin: true,
        },
        Source {
            id: "michoko999".into(),
            name: "michoko999/Instant-Gaming-Giveaway".into(),
            url: "https://raw.githubusercontent.com/michoko999/Instant-Gaming-Giveaway/refs/heads/main/List-Uncheck.csv".into(),
            kind: SourceKind::Csv,
            enabled: true,
            builtin: true,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub language: String,
    pub mode: SessionMode,
    pub seconds_per_giveaway: f64,
    pub skip_done_today: bool,
    pub accent: String,
    pub sky_animation: bool,
    pub sync_on_start: bool,
    pub check_updates: bool,
    pub onboarding_done: bool,
    pub account_connected: bool,
    pub sources: Vec<Source>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "fr".into(),
            mode: SessionMode::Auto,
            seconds_per_giveaway: 4.0,
            skip_done_today: true,
            accent: DEFAULT_ACCENT.into(),
            sky_animation: true,
            sync_on_start: true,
            check_updates: true,
            onboarding_done: false,
            account_connected: false,
            sources: builtin_sources(),
        }
    }
}

impl Settings {
    /// Corrige les valeurs incohérentes (fichier modifié à la main, ancienne version...).
    pub fn sanitize(&mut self) {
        if !LANGUAGES.contains(&self.language.as_str()) {
            self.language = "en".into();
        }
        if !self.seconds_per_giveaway.is_finite() {
            self.seconds_per_giveaway = 4.0;
        }
        self.seconds_per_giveaway = self.seconds_per_giveaway.clamp(2.0, 30.0);
        if !is_hex_color(&self.accent) {
            self.accent = DEFAULT_ACCENT.into();
        }
        // Les sources intégrées sont toujours présentes (mais peuvent être désactivées).
        for builtin in builtin_sources() {
            match self.sources.iter_mut().find(|s| s.id == builtin.id) {
                Some(existing) => {
                    let enabled = existing.enabled;
                    *existing = Source { enabled, ..builtin };
                }
                None => self.sources.insert(0, builtin),
            }
        }
        self.sources.retain(|s| s.builtin || s.url.starts_with("https://"));
        let mut seen = std::collections::HashSet::new();
        self.sources.retain(|s| seen.insert(s.id.clone()));
    }
}

pub fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Giveaway {
    pub code: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub checked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_participation: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_outcome: Option<Outcome>,
    #[serde(default)]
    pub sources: Vec<String>,
    pub added_at: DateTime<Utc>,
    #[serde(default)]
    pub ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub code: String,
    pub at: DateTime<Utc>,
    pub outcome: Outcome,
    pub mode: SessionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SessionSummary {
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub mode: SessionMode,
    pub total: usize,
    pub participated: usize,
    pub already: usize,
    pub opened: usize,
    pub ended: usize,
    pub errors: usize,
    pub skipped: usize,
    /// Concours qui demandent une action manuelle (vérification humaine).
    pub pending: Vec<String>,
    pub stopped: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Meta {
    pub last_sync: Option<DateTime<Utc>>,
    pub last_check: Option<DateTime<Utc>>,
    pub last_session: Option<SessionSummary>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub total: usize,
    pub active: usize,
    pub ended: usize,
    pub unknown: usize,
    pub ignored: usize,
    /// Actifs restants pour une session (hors ignorés et déjà faits aujourd'hui si l'option est active).
    pub queue: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_participations: usize,
    pub today: usize,
    pub streak_days: u32,
    pub best_streak: u32,
}
