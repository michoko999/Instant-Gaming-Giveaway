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
    ("es", "Este sorteo ya ha terminado", "Gana el juego que quieras"),
    ("de", "Dieses Giveaway ist vorüber", "Gewinne ein Spiel deiner Wahl"),
    ("pt", "O giveaway terminou", "Ganha um jogo à tua escolha"),
    ("it", "Questo giveaway è finito", "Vinci un gioco a tua scelta"),
    ("pl", "To giveaway zakończone", "Wygraj wybraną przez siebie grę"),
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
    if MARKERS.iter().any(|(_, _, active)| body.contains(active)) || body.contains("participate-actions") {
        return Status::Active;
    }
    Status::Unknown
}

pub async fn check_one(client: &Client, lang: &str, code: &str) -> Status {
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
                    Ok(body) => classify(status, &body),
                    Err(_) => Status::Unknown,
                };
            }
            Err(e) if e.is_timeout() && attempt == 0 => continue,
            Err(_) => return Status::Unknown,
        }
    }
    Status::Unknown
}

/// Vérifie une liste de codes ; `on_result` est appelé après chaque page.
pub async fn check_all<F>(client: &Client, lang: &str, codes: Vec<String>, mut on_result: F)
where
    F: FnMut(&str, Status),
{
    let mut results = stream::iter(codes.into_iter().map(|code| async move {
        let status = check_one(client, lang, &code).await;
        (code, status)
    }))
    .buffer_unordered(CONCURRENCY);
    while let Some((code, status)) = results.next().await {
        on_result(&code, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_pages() {
        assert_eq!(classify(404, ""), Status::Ended);
        assert_eq!(classify(500, "Gagne le jeu de ton choix"), Status::Unknown);
        assert_eq!(classify(200, "<h1>Ce Giveaway est terminé</h1>"), Status::Ended);
        assert_eq!(classify(200, "<div>Gagne le jeu de ton choix</div>"), Status::Active);
        assert_eq!(classify(200, "<div class=\"participate-actions\"></div>"), Status::Active);
        assert_eq!(classify(200, "Win the game of your choice"), Status::Active);
        assert_eq!(classify(200, "<html>accueil</html>"), Status::Unknown);
    }

    #[test]
    fn builds_urls() {
        assert_eq!(giveaway_url("pl", "PIVI"), "https://www.instant-gaming.com/pl/giveaway/PIVI");
    }
}
