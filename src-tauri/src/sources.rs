//! Récupération et lecture des listes de concours (JSON communautaire, CSV, imports).

use futures::future::join_all;
use reqwest::Client;
use serde::Serialize;

use crate::model::{Source, SourceKind};

/// Mots d'en-tête de colonnes à ignorer quand un fichier contient des codes nus.
const HEADER_WORDS: [&str; 9] = [
    "url",
    "urls",
    "code",
    "codes",
    "giveaway",
    "giveaways",
    "link",
    "lien",
    "liens",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceResult {
    pub id: String,
    pub name: String,
    pub ok: bool,
    pub count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct FetchOutcome {
    /// (code, identifiant de la source)
    pub codes: Vec<(String, String)>,
    /// Codes signalés comme terminés par une source JSON.
    pub dead: Vec<String>,
    pub results: Vec<SourceResult>,
}

/// Extrait le code d'un concours à partir d'une URL Instant Gaming ou d'un code nu.
/// Renvoie `None` pour une URL d'un autre site, une ligne vide ou un en-tête.
pub fn extract_code(raw: &str) -> Option<String> {
    let s = raw.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    let candidate = if let Some(idx) = lower.find("/giveaway/") {
        if s.contains("://") {
            let url = url::Url::parse(s).ok()?;
            let host = url.host_str()?.to_ascii_lowercase();
            if host != "instant-gaming.com" && !host.ends_with(".instant-gaming.com") {
                return None;
            }
        }
        &s[idx + "/giveaway/".len()..]
    } else if s.contains("://") || s.contains('/') || s.contains(' ') {
        return None;
    } else {
        if HEADER_WORDS.contains(&lower.as_str()) {
            return None;
        }
        s
    };
    let code: String = candidate
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if code.is_empty() || code.len() > 64 {
        None
    } else {
        Some(code)
    }
}

/// Lit une liste (CSV ou texte) : premier champ de chaque ligne.
pub fn parse_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let first = line.split([',', ';', '\t']).next().unwrap_or("");
            extract_code(first)
        })
        .collect()
}

/// Lit le JSON communautaire { "alive": [...], "dead": [...] }.
pub fn parse_alive_json(text: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON invalide : {e}"))?;
    let read = |key: &str| -> Vec<String> {
        value
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| i.as_str())
                    .filter_map(extract_code)
                    .collect()
            })
            .unwrap_or_default()
    };
    if value.get("alive").is_none() {
        return Err("champ « alive » absent".into());
    }
    Ok((read("alive"), read("dead")))
}

pub async fn fetch_text(client: &Client, url: &str) -> Result<String, String> {
    let response = client.get(url).send().await.map_err(|e| short_error(&e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    response.text().await.map_err(|e| short_error(&e))
}

fn short_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "délai dépassé".into()
    } else if e.is_connect() {
        "connexion impossible".into()
    } else {
        "erreur réseau".into()
    }
}

/// Récupère toutes les sources actives en parallèle.
pub async fn fetch_all(client: &Client, sources: &[Source]) -> FetchOutcome {
    let enabled: Vec<&Source> = sources.iter().filter(|s| s.enabled).collect();
    let texts = join_all(enabled.iter().map(|s| fetch_text(client, &s.url))).await;
    let mut outcome = FetchOutcome::default();
    for (source, text) in enabled.into_iter().zip(texts) {
        let parsed = text.and_then(|t| match source.kind {
            SourceKind::AliveJson => parse_alive_json(&t),
            SourceKind::Csv => Ok((parse_list(&t), Vec::new())),
        });
        match parsed {
            Ok((alive, dead)) => {
                outcome.results.push(SourceResult {
                    id: source.id.clone(),
                    name: source.name.clone(),
                    ok: true,
                    count: alive.len(),
                    error: None,
                });
                outcome
                    .codes
                    .extend(alive.into_iter().map(|c| (c, source.id.clone())));
                outcome.dead.extend(dead);
            }
            Err(error) => outcome.results.push(SourceResult {
                id: source.id.clone(),
                name: source.name.clone(),
                ok: false,
                count: 0,
                error: Some(error),
            }),
        }
    }
    // Un code présent dans une liste « vivante » ne doit pas être marqué mort par une autre.
    let alive: std::collections::HashSet<String> = outcome
        .codes
        .iter()
        .map(|(c, _)| c.to_ascii_uppercase())
        .collect();
    outcome
        .dead
        .retain(|c| !alive.contains(&c.to_ascii_uppercase()));
    outcome
}

/// Devine le type d'une source ajoutée par l'utilisateur à partir de son URL.
pub fn guess_kind(url: &str) -> SourceKind {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    if path.ends_with(".json") {
        SourceKind::AliveJson
    } else {
        SourceKind::Csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_codes_from_urls_and_plain_codes() {
        assert_eq!(
            extract_code("https://www.instant-gaming.com/fr/giveaway/TOMBIE"),
            Some("TOMBIE".into())
        );
        assert_eq!(
            extract_code("https://www.instant-gaming.com/fr/giveaway/TOMBIE#"),
            Some("TOMBIE".into())
        );
        assert_eq!(
            extract_code("  \"https://instant-gaming.com/en/giveaway/muusoo?x=1\" "),
            Some("muusoo".into())
        );
        assert_eq!(extract_code("MFGAMING"), Some("MFGAMING".into()));
        assert_eq!(extract_code("Seals311"), Some("Seals311".into()));
        assert_eq!(
            extract_code("https://evil.example.com/giveaway/TOMBIE"),
            None
        );
        assert_eq!(extract_code("https://www.instant-gaming.com/fr/"), None);
        assert_eq!(extract_code("url"), None);
        assert_eq!(extract_code(""), None);
        assert_eq!(extract_code("deux mots"), None);
    }

    #[test]
    fn parses_csv_lines() {
        let text = "url\nhttps://www.instant-gaming.com/fr/giveaway/ICONOBLAST\n\nPIVI;note\nhttps://example.com/x\n";
        assert_eq!(
            parse_list(text),
            vec!["ICONOBLAST".to_string(), "PIVI".to_string()]
        );
    }

    #[test]
    fn parses_alive_json() {
        let (alive, dead) =
            parse_alive_json(r#"{"alive":["BOBLENNON","bugland"],"dead":["OLD"]}"#).unwrap();
        assert_eq!(alive, vec!["BOBLENNON".to_string(), "bugland".to_string()]);
        assert_eq!(dead, vec!["OLD".to_string()]);
        assert!(parse_alive_json(r#"{"foo":1}"#).is_err());
        assert!(parse_alive_json("pas du json").is_err());
    }

    #[test]
    fn guesses_source_kind() {
        assert_eq!(guess_kind("https://x.org/list.json"), SourceKind::AliveJson);
        assert_eq!(guess_kind("https://x.org/list.csv?raw=1"), SourceKind::Csv);
    }
}
