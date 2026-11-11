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

/// Cherche tous les liens vers des concours Instant Gaming dans une page web, y compris
/// ceux cachés derrière une redirection (`…/visit?url=https://www.instant-gaming.com/…`,
/// éventuellement encodée) ou échappés dans du JSON. Doublons retirés, ordre conservé.
pub fn scan_page(text: &str) -> Vec<String> {
    const HOST: &str = "instant-gaming.com/";
    let decoded = text
        .replace("%2F", "/")
        .replace("%2f", "/")
        .replace("\\/", "/");
    let lower = decoded.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut seen = std::collections::HashSet::new();
    let mut codes = Vec::new();
    let mut from = 0;
    while let Some(pos) = lower[from..].find(HOST) {
        let start = from + pos;
        from = start + HOST.len();
        // « www.instant-gaming.com » ou « //instant-gaming.com », pas « autreinstant-gaming.com ».
        if start > 0 {
            let before = bytes[start - 1];
            if before.is_ascii_alphanumeric() || before == b'-' {
                continue;
            }
        }
        let mut rest = &lower[from..];
        // Préfixe de langue facultatif : « fr/ », « en/ »…
        if let Some(slash) = rest.find('/') {
            if (2..=5).contains(&slash)
                && rest[..slash]
                    .bytes()
                    .all(|b| b.is_ascii_alphabetic() || b == b'-')
            {
                rest = &rest[slash + 1..];
            }
        }
        if !rest.starts_with("giveaway/") {
            continue;
        }
        let offset = decoded.len() - rest.len() + "giveaway/".len();
        let mut code: String = decoded[offset..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        // Deux liens collés sans espace : « …/retroraiderhttps://… ».
        if decoded[offset + code.len()..].starts_with("://") {
            for scheme in ["https", "http"] {
                if code.len() > scheme.len() && code.to_ascii_lowercase().ends_with(scheme) {
                    code.truncate(code.len() - scheme.len());
                    break;
                }
            }
        }
        if !code.is_empty() && code.len() <= 64 && seen.insert(code.to_ascii_uppercase()) {
            codes.push(code);
        }
    }
    codes
}

/// Retire les doublons (sans tenir compte de la casse) en gardant l'ordre.
pub fn dedupe(codes: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    codes
        .into_iter()
        .filter(|c| seen.insert(c.to_ascii_uppercase()))
        .collect()
}

/// Lit un texte collé par l'utilisateur. S'il contient des liens Instant Gaming, on ne garde
/// qu'eux (le reste du texte est ignoré) ; sinon, chaque mot est pris comme un code.
pub fn parse_pasted(text: &str) -> Vec<String> {
    let links = scan_page(text);
    if !links.is_empty() {
        return links;
    }
    dedupe(
        text.split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .filter_map(extract_code)
            .collect(),
    )
}

/// Lit un fichier importé : JSON communautaire, liste (CSV, texte) ou page enregistrée.
pub fn parse_file(text: &str) -> Vec<String> {
    match parse_alive_json(text) {
        Ok((alive, _)) => alive,
        Err(_) => {
            let mut codes = parse_list(text);
            codes.extend(scan_page(text));
            dedupe(codes)
        }
    }
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
            SourceKind::Page => Ok((scan_page(&t), Vec::new())),
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

/// Devine le type d'une source ajoutée par l'utilisateur à partir de son URL :
/// JSON communautaire, fichier liste (CSV, texte, GitHub brut) ou page web.
pub fn guess_kind(url: &str) -> SourceKind {
    let lower = url.to_ascii_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or(&lower);
    let raw_host = [
        "://raw.githubusercontent.com/",
        "://gist.githubusercontent.com/",
        "://pastebin.com/raw/",
    ]
    .iter()
    .any(|h| path.contains(h));
    if path.ends_with(".json") {
        SourceKind::AliveJson
    } else if raw_host
        || [".csv", ".txt", ".tsv"]
            .iter()
            .any(|ext| path.ends_with(ext))
    {
        SourceKind::Csv
    } else {
        SourceKind::Page
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
        assert_eq!(
            guess_kind("https://raw.githubusercontent.com/a/b/main/list"),
            SourceKind::Csv
        );
        assert_eq!(
            guess_kind("https://jv-actu.com/gagner-un-jeu-gratuit-par-mois/"),
            SourceKind::Page
        );
        assert_eq!(
            guess_kind("https://www.ledemondujeu.com/jeux-concours-instant-gaming.com.html"),
            SourceKind::Page
        );
    }

    #[test]
    fn reads_pasted_text() {
        let message = "Ajoute-les :\nhttps://www.instant-gaming.com/pt/giveaway/retroraider\n\
            https://www.instant-gaming.com/en/giveaway/goldpeaking et https://www.instant-gaming.com/pl/giveaway/HIGHSTYLED\n\
            https://www.instant-gaming.com/en/giveaway/GOLDPEAKING merci";
        assert_eq!(
            parse_pasted(message),
            vec!["retroraider", "goldpeaking", "HIGHSTYLED"]
        );
        assert_eq!(
            parse_pasted("BOBLENNON, pivi;  TOMBIE\nbobLennon"),
            vec!["BOBLENNON", "pivi", "TOMBIE"]
        );
        assert_eq!(
            parse_pasted("https://www.instant-gaming.com/pt/giveaway/retroraiderhttps://www.instant-gaming.com/en/giveaway/goldpeaking"),
            vec!["retroraider", "goldpeaking"]
        );
        assert!(parse_pasted("   ").is_empty());
        assert!(parse_pasted("https://example.com/fr/giveaway/X").is_empty());
    }

    #[test]
    fn reads_imported_files() {
        assert_eq!(
            parse_file("url\nhttps://www.instant-gaming.com/fr/giveaway/PIVI\nTOMBIE\n"),
            vec!["PIVI", "TOMBIE"]
        );
        assert_eq!(
            parse_file("<p>Voir <a href=\"https://www.instant-gaming.com/fr/giveaway/SEGMENT\">ici</a></p>"),
            vec!["SEGMENT"]
        );
        assert_eq!(parse_file(r#"{"alive":["A1"],"dead":["B2"]}"#), vec!["A1"]);
    }

    #[test]
    fn scans_giveaway_links_in_pages() {
        let html = r#"
            <a href="https://www.instant-gaming.com/fr/giveaway/RATSUPER?igr=me5rine-lab"><img></a>
            <a href="https://www.instant-gaming.com/fr/giveaway/me5rine-lab">x</a>
            <p>https://www.instant-gaming.com/it/giveaway/VELOX et https://www.instant-gaming.com/fr/giveaway/TOMBIE#</p>
            <a href="http://ledemondujeu.digidip.net/visit?url=https://www.instant-gaming.com/fr/giveaway/supremeleader?win=MTA=&amp;utm_source=x">gagnants</a>
            <a href="https://t.example/?u=https%3A%2F%2Fwww.instant-gaming.com%2Fen%2Fgiveaway%2FENCODED">y</a>
            <script>{"u":"https:\/\/www.instant-gaming.com\/de\/giveaway\/JSONCODE"}</script>
            https://www.instant-gaming.com/fr/giveaway/ratsuper (doublon)
            https://www.instant-gaming.com/giveaway/NOLANG
            https://www.instant-gaming.com/fr/giveaways/ https://www.instant-gaming.com/fr/
            https://fakeinstant-gaming.com/fr/giveaway/EVIL https://instant-gaming.com.evil.org/fr/giveaway/EVIL2
        "#;
        assert_eq!(
            scan_page(html),
            vec![
                "RATSUPER",
                "me5rine-lab",
                "VELOX",
                "TOMBIE",
                "supremeleader",
                "ENCODED",
                "JSONCODE",
                "NOLANG"
            ]
        );
        assert!(scan_page("<html>rien ici</html>").is_empty());
    }
}
