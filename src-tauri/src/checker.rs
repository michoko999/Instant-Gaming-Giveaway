//! Vérification de l'état des concours en lisant leur page publique.

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
pub const CONCURRENCY: usize = 4;

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
}

pub async fn check_one(client: &Client, lang: &str, code: &str) -> PageCheck {
    let url = giveaway_url(lang, code);
    for attempt in 0..2 {
        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                if status == 429 && attempt == 0 {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
                return match response.text().await {
                    Ok(body) => PageCheck {
                        status: classify(status, &body),
                        winners: parse_winners(&body),
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

/// Vérifie une liste de codes ; `on_result` est appelé après chaque page.
pub async fn check_all<F>(client: &Client, lang: &str, codes: Vec<String>, mut on_result: F)
where
    F: FnMut(&str, PageCheck),
{
    let mut results = stream::iter(codes.into_iter().map(|code| async move {
        let page = check_one(client, lang, &code).await;
        (code, page)
    }))
    .buffer_unordered(CONCURRENCY);
    while let Some((code, page)) = results.next().await {
        on_result(&code, page);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
