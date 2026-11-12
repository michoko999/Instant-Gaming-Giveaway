//! Vérification de l'état des concours en lisant leur page publique.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::stream::{self, StreamExt};
use reqwest::Client;

use crate::model::Status;

/// Textes affichés par Instant Gaming selon la langue de la page :
/// (langue, concours terminé, concours ouvert).
const MARKERS: [(&str, &str, &str); 7] = [
    ("fr", "Ce Giveaway est terminé", "Gagne le jeu de ton choix"),
    ("en", "This giveaway is over", "Win the game of your choice"),
    (
        "es",
        "Este sorteo ya ha terminado",
        "Gana el juego que quieras",
    ),
    (
        "de",
        "Dieses Giveaway ist vorüber",
        "Gewinne ein Spiel deiner Wahl",
    ),
    ("pt", "O giveaway terminou", "Ganha um jogo à tua escolha"),
    (
        "it",
        "Questo giveaway è finito",
        "Vinci un gioco a tua scelta",
    ),
    (
        "pl",
        "To giveaway zakończone",
        "Wygraj wybraną przez siebie grę",
    ),
];

/// Nombre de pages vérifiées en même temps (volontairement modeste pour ne pas surcharger le site).
pub const CONCURRENCY: usize = 2;
/// Écart minimal entre deux pages demandées : pas de rafale de requêtes vers le site.
pub const SPACING: Duration = Duration::from_millis(600);
/// Attente maximale acceptée quand le site demande de patienter (`Retry-After`).
const MAX_RETRY_AFTER_SECS: u64 = 30;

pub fn giveaway_url(lang: &str, code: &str) -> String {
    format!("https://www.instant-gaming.com/{lang}/giveaway/{code}")
}

/// Tous les textes « concours terminé », pour le script de la fenêtre Instant Gaming.
pub fn ended_markers() -> Vec<&'static str> {
    MARKERS.iter().map(|(_, ended, _)| *ended).collect()
}

/// Détermine l'état d'un concours à partir de la réponse HTTP.
pub fn classify(http_status: u16, body: &str) -> Status {
    if http_status == 404 || http_status == 410 {
        return Status::Ended;
    }
    if !(200..300).contains(&http_status) {
        return Status::Unknown;
    }
    if MARKERS.iter().any(|(_, ended, _)| body.contains(ended)) {
        return Status::Ended;
    }
    if MARKERS.iter().any(|(_, _, active)| body.contains(active))
        || body.contains("participate-actions")
    {
        return Status::Active;
    }
    Status::Unknown
}

/// Un gagnant affiché sur la page d'un concours (« Derniers gagnants »).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Winner {
    pub nickname: String,
    /// Identifiant du profil public (`/user/<profil>`).
    pub profile: Option<String>,
    /// Page du jeu gagné.
    pub product_url: Option<String>,
}

/// Lit la liste des derniers gagnants, écrite en JSON dans la page
/// (`window.giveaway = { …, winners: [ … ] }`). Liste vide si elle est absente ou illisible.
pub fn parse_winners(body: &str) -> Vec<Winner> {
    #[derive(serde::Deserialize)]
    struct RawProduct {
        #[serde(default)]
        url: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct RawWinner {
        #[serde(default)]
        nickname: String,
        #[serde(default)]
        profile_url: Option<String>,
        #[serde(default)]
        product: Option<RawProduct>,
    }
    let Some(start) = body.find("window.giveaway") else {
        return Vec::new();
    };
    let rest = &body[start..];
    let Some(key) = rest.find("winners") else {
        return Vec::new();
    };
    let after = rest[key + "winners".len()..].trim_start_matches(['"', '\'']);
    let Some(after) = after.trim_start().strip_prefix(':') else {
        return Vec::new();
    };
    let Some(json) = balanced_array(after.trim_start()) else {
        return Vec::new();
    };
    let raw: Vec<RawWinner> = serde_json::from_str(json).unwrap_or_default();
    raw.into_iter()
        .map(|w| Winner {
            nickname: w.nickname.trim().to_string(),
            profile: w.profile_url.as_deref().and_then(profile_from_url),
            product_url: w
                .product
                .and_then(|p| p.url)
                .filter(|u| u.starts_with("https://")),
        })
        .collect()
}

/// Identifiant de profil dans une adresse `…/user/<profil>`.
pub fn profile_from_url(url: &str) -> Option<String> {
    let rest = url.split("/user/").nth(1)?;
    let profile = rest.split(['/', '?', '#']).next()?.trim();
    (!profile.is_empty()).then(|| profile.to_string())
}

/// Le tableau JSON qui commence au début de `text` (crochets équilibrés, chaînes respectées).
fn balanced_array(text: &str) -> Option<&str> {
    if !text.starts_with('[') {
        return None;
    }
    let (mut depth, mut in_string, mut escaped) = (0usize, false, false);
    for (i, c) in text.char_indices() {
        if i > 200_000 {
            return None;
        }
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '[' | '{' => depth += 1,
            ']' | '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&text[..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Résultat de la vérification d'une page.
#[derive(Debug, Clone, Default)]
pub struct PageCheck {
    pub status: Status,
    pub winners: Vec<Winner>,
    /// Le site demande de ralentir (trop de requêtes, page de vérification anti-robot) :
    /// la vérification s'arrête là.
    pub blocked: bool,
}

/// Page « trop de requêtes » ou défi anti-robot (Cloudflare) à la place de la page demandée.
pub fn is_blocked(http_status: u16, body: &str) -> bool {
    http_status == 429
        || (matches!(http_status, 403 | 503)
            && [
                "cf-chl",
                "cf_chl_opt",
                "Just a moment...",
                "Attention Required!",
            ]
            .iter()
            .any(|marker| body.contains(marker)))
}

/// Délai demandé par l'en-tête `Retry-After` (en secondes), s'il est raisonnable.
fn retry_after(response: &reqwest::Response) -> Option<u64> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|secs| *secs <= MAX_RETRY_AFTER_SECS)
}

pub async fn check_one(client: &Client, lang: &str, code: &str) -> PageCheck {
    let url = giveaway_url(lang, code);
    for attempt in 0..2 {
        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                if status == 429 {
                    // Le site demande d'attendre : une seule nouvelle tentative, puis on arrête.
                    match retry_after(&response) {
                        Some(secs) if attempt == 0 => {
                            tokio::time::sleep(Duration::from_secs(secs.max(2))).await;
                            continue;
                        }
                        _ => {
                            return PageCheck {
                                blocked: true,
                                ..PageCheck::default()
                            }
                        }
                    }
                }
                return match response.text().await {
                    Ok(body) if is_blocked(status, &body) => PageCheck {
                        blocked: true,
                        ..PageCheck::default()
                    },
                    Ok(body) => PageCheck {
                        status: classify(status, &body),
                        winners: parse_winners(&body),
                        blocked: false,
                    },
                    Err(_) => PageCheck::default(),
                };
            }
            Err(e) if e.is_timeout() && attempt == 0 => continue,
            Err(_) => return PageCheck::default(),
        }
    }
    PageCheck::default()
}

/// Vérifie une liste de codes, à un rythme modéré ; `on_result` est appelé après chaque page.
/// S'arrête dès que le site demande de ralentir, et renvoie alors `true`.
pub async fn check_all<F>(client: &Client, lang: &str, codes: Vec<String>, mut on_result: F) -> bool
where
    F: FnMut(&str, PageCheck),
{
    let blocked = AtomicBool::new(false);
    let mut ticker = tokio::time::interval(SPACING);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let pace = tokio::sync::Mutex::new(ticker);
    let (blocked_ref, pace_ref) = (&blocked, &pace);
    let mut results = stream::iter(codes.into_iter().map(|code| async move {
        pace_ref.lock().await.tick().await;
        if blocked_ref.load(Ordering::SeqCst) {
            return (code, None);
        }
        let page = check_one(client, lang, &code).await;
        if page.blocked {
            blocked_ref.store(true, Ordering::SeqCst);
        }
        (code, Some(page))
    }))
    .buffer_unordered(CONCURRENCY);
    while let Some((code, page)) = results.next().await {
        match page {
            Some(page) if !page.blocked => on_result(&code, page),
            _ => {}
        }
        if blocked.load(Ordering::SeqCst) {
            break;
        }
    }
    blocked.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_a_request_to_slow_down() {
        assert!(is_blocked(429, ""));
        assert!(is_blocked(
            403,
            "<title>Just a moment...</title><script>window._cf_chl_opt={}</script>"
        ));
        assert!(is_blocked(503, "<div id=\"cf-chl-widget\"></div>"));
        // Une page normale du site charge aussi un script Cloudflare : ce n'est pas un blocage.
        assert!(!is_blocked(
            200,
            "<script src=\"/cdn-cgi/challenge-platform/scripts/jsd/main.js\"></script>"
        ));
        assert!(!is_blocked(404, "Not found"));
        assert!(!is_blocked(403, "Forbidden"));
    }

    #[test]
    fn classifies_pages() {
        assert_eq!(classify(404, ""), Status::Ended);
        assert_eq!(classify(500, "Gagne le jeu de ton choix"), Status::Unknown);
        assert_eq!(
            classify(200, "<h1>Ce Giveaway est terminé</h1>"),
            Status::Ended
        );
        assert_eq!(
            classify(200, "<div>Gagne le jeu de ton choix</div>"),
            Status::Active
        );
        assert_eq!(
            classify(200, "<div class=\"participate-actions\"></div>"),
            Status::Active
        );
        assert_eq!(classify(200, "Win the game of your choice"), Status::Active);
        assert_eq!(classify(200, "<html>accueil</html>"), Status::Unknown);
    }

    #[test]
    fn reads_the_winners_of_a_page() {
        // Extrait réel d'une page de concours (raccourci).
        let body = r#"<script>window.giveaway = { isParticipating: false, participationId: 0,
            partnerPush: {"twitter":{"networkName":"twitter"}},
            winners: [{"nickname":"M8_SunSworD_","profile_url":"https:\/\/www.instant-gaming.com\/fr\/user\/gamer-33ecea","avatar_url":"https:\/\/gaming-cdn.com\/images\/avatars\/9507840-1793487886.jpg","level":"32.00","last_participation_date":"Il y a 10 jours","product":{"product_id":23617,"updated_at":1793261315,"url":"https:\/\/www.instant-gaming.com\/fr\/23617-acheter-phantom-blade-zero-digital-deluxe-edition-pc-steam\/"}},{"nickname":"Faerune [x]","profile_url":"https:\/\/www.instant-gaming.com\/fr\/user\/gamer-91bf31","product":null}],
            url: "x" };</script>"#;
        let winners = parse_winners(body);
        assert_eq!(winners.len(), 2);
        assert_eq!(winners[0].nickname, "M8_SunSworD_");
        assert_eq!(winners[0].profile.as_deref(), Some("gamer-33ecea"));
        assert_eq!(
            winners[0].product_url.as_deref(),
            Some("https://www.instant-gaming.com/fr/23617-acheter-phantom-blade-zero-digital-deluxe-edition-pc-steam/")
        );
        assert_eq!(winners[1].nickname, "Faerune [x]");
        assert!(winners[1].product_url.is_none());
        assert!(parse_winners("<html>pas de concours</html>").is_empty());
        assert!(parse_winners("window.giveaway = { winners: [] }").is_empty());
        assert!(parse_winners("window.giveaway = { winners: [{\"nickname\": ").is_empty());
        assert_eq!(
            profile_from_url("https://www.instant-gaming.com/de/user/Mich%C3%B6ko?x=1"),
            Some("Mich%C3%B6ko".into())
        );
    }

    #[test]
    fn builds_urls() {
        assert_eq!(
            giveaway_url("pl", "PIVI"),
            "https://www.instant-gaming.com/pl/giveaway/PIVI"
        );
    }
}
