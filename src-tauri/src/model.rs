//! Types de données partagés entre le cœur de l'application et l'interface.
//! Tous les champs sont sérialisés en camelCase pour le TypeScript.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Langues prises en charge (interface et pages Instant Gaming).
pub const LANGUAGES: [&str; 7] = ["fr", "en", "es", "de", "pt", "it", "pl"];

pub const DEFAULT_ACCENT: &str = "#FFD27A";

/// Mode prudent : temps minimum entre deux concours.
pub const SAFE_MIN_DELAY: f64 = 8.0;

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

    /// Le concours est « fait » pour le mois : Instant Gaming n'accepte qu'une participation
    /// par concours et par mois. Une page seulement ouverte (mode manuel) ne compte pas.
    pub fn done_for_month(self) -> bool {
        matches!(self, Outcome::Participated | Outcome::Already)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    /// JSON de la forme { "alive": [...], "dead": [...] }.
    AliveJson,
    /// Liste d'URL ou de codes, une par ligne (CSV ou texte).
    Csv,
    /// Page web : tous les liens vers des concours Instant Gaming qu'elle contient.
    Page,
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
        Source {
            id: "jv-actu".into(),
            name: "jv-actu.com".into(),
            url: "https://jv-actu.com/gagner-un-jeu-gratuit-par-mois/".into(),
            kind: SourceKind::Page,
            enabled: true,
            builtin: true,
        },
        Source {
            id: "ledemondujeu".into(),
            name: "ledemondujeu.com".into(),
            url: "https://www.ledemondujeu.com/jeux-concours-instant-gaming.com.html".into(),
            kind: SourceKind::Page,
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
    #[serde(alias = "skipDoneToday")]
    pub skip_done_this_month: bool,
    /// Mode prudent : rythme minimum et arrêt automatique aux premiers signaux du site.
    pub safe_mode: bool,
    /// Nombre maximal de concours par session automatique (0 = illimité).
    pub session_limit: u32,
    /// Afficher la fenêtre Instant Gaming pendant les sessions automatiques (sinon elle reste cachée).
    pub show_ig_window: bool,
    pub accent: String,
    pub sky_animation: bool,
    pub sync_on_start: bool,
    pub check_updates: bool,
    pub onboarding_done: bool,
    /// Ancien réglage (un seul compte) : lu pour la migration vers `accounts.json`, plus écrit.
    #[serde(skip_serializing)]
    pub account_connected: bool,
    pub sources: Vec<Source>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "fr".into(),
            mode: SessionMode::Auto,
            seconds_per_giveaway: SAFE_MIN_DELAY,
            skip_done_this_month: true,
            safe_mode: true,
            session_limit: 40,
            show_ig_window: false,
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
            self.seconds_per_giveaway = SAFE_MIN_DELAY;
        }
        self.seconds_per_giveaway = self.seconds_per_giveaway.clamp(2.0, 30.0);
        if self.safe_mode {
            self.seconds_per_giveaway = self.seconds_per_giveaway.max(SAFE_MIN_DELAY);
        }
        if self.session_limit > 0 {
            self.session_limit = self.session_limit.clamp(5, 500);
        }
        if !is_hex_color(&self.accent) {
            self.accent = DEFAULT_ACCENT.into();
        }
        // Les sources intégrées sont toujours présentes, en tête et dans leur ordre
        // (mais peuvent être désactivées) ; viennent ensuite les sources ajoutées.
        let mut rest = std::mem::take(&mut self.sources);
        let builtins = builtin_sources();
        for builtin in &builtins {
            let enabled = rest
                .iter()
                .find(|s| s.id == builtin.id)
                .map_or(builtin.enabled, |s| s.enabled);
            self.sources.push(Source {
                enabled,
                ..builtin.clone()
            });
        }
        rest.retain(|s| {
            !s.builtin
                && s.url.starts_with("https://")
                && !builtins.iter().any(|b| b.id == s.id || b.url == s.url)
        });
        let mut seen = std::collections::HashSet::new();
        rest.retain(|s| seen.insert(s.id.clone()));
        self.sources.extend(rest);
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
    /// Anciens champs (avant le multi-compte) : repris dans `marks` au chargement.
    #[serde(default, skip_serializing)]
    pub last_participation: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing)]
    pub last_outcome: Option<Outcome>,
    /// Dernière participation de chaque compte (identifiant du compte → marque).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub marks: BTreeMap<String, Mark>,
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
    /// Compte concerné (absent dans l'historique d'avant le multi-compte : premier compte).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

impl HistoryEntry {
    pub fn account_id(&self) -> &str {
        self.account.as_deref().unwrap_or(MAIN_ACCOUNT)
    }
}

/// Ce qu'un compte a fait sur un concours.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Mark {
    /// Dernière participation (réussie, déjà inscrit ou page ouverte en mode manuel).
    pub participated_at: Option<DateTime<Utc>>,
    /// Dernier résultat.
    pub outcome: Option<Outcome>,
}

/// Concours tel que l'interface le voit, pour le compte actif.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiveawayView {
    pub code: String,
    pub status: Status,
    pub checked_at: Option<DateTime<Utc>>,
    pub last_participation: Option<DateTime<Utc>>,
    pub last_outcome: Option<Outcome>,
    pub sources: Vec<String>,
    pub added_at: DateTime<Utc>,
    pub ignored: bool,
}

/// Identifiant du premier compte : son profil de navigation est le dossier `webview` d'origine.
pub const MAIN_ACCOUNT: &str = "main";
pub const MAX_ACCOUNTS: usize = 8;

/// Un compte Instant Gaming : son propre profil de navigation, son suivi et son historique.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    /// Pseudo affiché sur Instant Gaming.
    #[serde(default)]
    pub name: Option<String>,
    /// Identifiant du profil public (`/user/<profil>`).
    #[serde(default)]
    pub profile: Option<String>,
    /// Photo de profil enregistrée dans le dossier `avatars`.
    #[serde(default)]
    pub avatar_file: Option<String>,
    #[serde(default)]
    pub connected: bool,
    /// Dernière lecture du pseudo et de la photo.
    #[serde(default)]
    pub identity_at: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
}

impl Account {
    pub fn new(id: &str, now: DateTime<Utc>) -> Account {
        Account {
            id: id.to_string(),
            name: None,
            profile: None,
            avatar_file: None,
            connected: false,
            identity_at: None,
            added_at: now,
        }
    }

    pub fn forget_identity(&mut self) {
        self.connected = false;
        self.name = None;
        self.profile = None;
        self.avatar_file = None;
        self.identity_at = None;
    }
}

/// Un identifiant de compte sert aussi de nom de dossier : lettres, chiffres et tirets.
pub fn valid_account_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Accounts {
    pub active: String,
    pub list: Vec<Account>,
}

impl Default for Accounts {
    fn default() -> Self {
        Accounts {
            active: MAIN_ACCOUNT.into(),
            list: vec![Account::new(MAIN_ACCOUNT, Utc::now())],
        }
    }
}

impl Accounts {
    /// Toujours au moins un compte, identifiants valides et uniques, compte actif existant.
    pub fn sanitize(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.list
            .retain(|a| valid_account_id(&a.id) && seen.insert(a.id.clone()));
        self.list.truncate(MAX_ACCOUNTS);
        if self.list.is_empty() {
            self.list.push(Account::new(MAIN_ACCOUNT, Utc::now()));
        }
        if !self.list.iter().any(|a| a.id == self.active) {
            self.active = self.list[0].id.clone();
        }
    }

    pub fn active(&self) -> &Account {
        self.list
            .iter()
            .find(|a| a.id == self.active)
            .unwrap_or(&self.list[0])
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.list.iter_mut().find(|a| a.id == id)
    }
}

/// Compte tel que l'interface le voit.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub name: Option<String>,
    /// Photo de profil en `data:` (l'interface n'affiche aucune image externe).
    pub avatar: Option<String>,
    pub profile_url: Option<String>,
    pub connected: bool,
    pub active: bool,
    /// Concours différents déjà faits ce mois-ci par ce compte.
    pub done_this_month: usize,
    /// Concours qu'une session de ce compte traiterait maintenant.
    pub queue: usize,
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
    /// Raison d'un arrêt automatique : "warning", "captcha" ou "errors".
    pub stop_reason: Option<String>,
    /// Limite de concours appliquée à cette session (0 = aucune).
    pub limited: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Meta {
    pub last_sync: Option<DateTime<Utc>>,
    pub last_check: Option<DateTime<Utc>>,
    pub last_session: Option<SessionSummary>,
    /// Gains repérés dans les listes de gagnants.
    pub wins: Vec<Win>,
    /// Mois (« AAAA-MM ») dont le bandeau « nouveau mois » a été fermé.
    pub seen_month: Option<String>,
    /// Dernière visite de l'écran Concours : les concours ajoutés après sont « nouveaux ».
    pub giveaways_seen_at: Option<DateTime<Utc>>,
    /// Version de l'application qui a écrit ces données (sauvegarde avant mise à jour).
    pub app_version: Option<String>,
}

/// Un compte de l'application apparaît parmi les gagnants d'un concours.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Win {
    pub account: String,
    pub code: String,
    /// Jeu gagné (nom tiré de l'adresse de sa page) et adresse de la page.
    pub product: Option<String>,
    pub product_url: Option<String>,
    pub detected_at: DateTime<Utc>,
    /// L'utilisateur a déjà vu l'annonce.
    pub seen: bool,
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
    /// Concours actifs ajoutés depuis la dernière visite de l'écran Concours.
    pub fresh: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_participations: usize,
    /// Participations enregistrées pendant le mois en cours.
    pub this_month: usize,
    /// Concours différents déjà faits ce mois-ci.
    pub done_this_month: usize,
    /// Participations des 12 derniers mois (heure locale), du plus ancien au mois en cours.
    pub monthly: Vec<MonthCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthCount {
    /// « AAAA-MM »
    pub month: String,
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn careful_mode_enforces_minimum_delay() {
        let mut settings = Settings {
            seconds_per_giveaway: 3.0,
            safe_mode: true,
            ..Settings::default()
        };
        settings.sanitize();
        assert_eq!(settings.seconds_per_giveaway, SAFE_MIN_DELAY);

        let mut settings = Settings {
            seconds_per_giveaway: 3.0,
            safe_mode: false,
            ..Settings::default()
        };
        settings.sanitize();
        assert_eq!(settings.seconds_per_giveaway, 3.0);

        let mut settings = Settings {
            seconds_per_giveaway: f64::NAN,
            ..Settings::default()
        };
        settings.sanitize();
        assert_eq!(settings.seconds_per_giveaway, SAFE_MIN_DELAY);
    }

    #[test]
    fn builtin_sources_stay_first_and_keep_their_switch() {
        let custom = Source {
            id: "custom-1".into(),
            name: "Ma liste".into(),
            url: "https://example.org/list.csv".into(),
            kind: SourceKind::Csv,
            enabled: true,
            builtin: false,
        };
        let mut old_builtin = builtin_sources()[1].clone();
        old_builtin.enabled = false;
        let duplicate = Source {
            id: "custom-2".into(),
            builtin: false,
            ..builtin_sources()[2].clone()
        };
        let mut settings = Settings {
            sources: vec![custom.clone(), old_builtin, duplicate],
            ..Settings::default()
        };
        settings.sanitize();
        let ids: Vec<&str> = settings.sources.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "enzomtpyt",
                "michoko999",
                "jv-actu",
                "ledemondujeu",
                "custom-1"
            ]
        );
        assert!(!settings.sources[1].enabled);
        assert!(settings.sources[2].enabled);
    }

    #[test]
    fn ig_window_hidden_by_default() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.show_ig_window);
        assert!(settings.safe_mode);
    }
}
