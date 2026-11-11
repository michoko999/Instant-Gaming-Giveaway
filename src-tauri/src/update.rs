//! Vérifie si une version plus récente est publiée sur GitHub.
//! L'application ne télécharge rien elle-même : elle affiche seulement un lien vers la page.

use reqwest::Client;
use serde::{Deserialize, Serialize};

const LATEST_RELEASE: &str = "https://api.github.com/repos/michoko999/Instant-Gaming-Giveaway/releases/latest";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: Option<String>,
    pub available: bool,
    pub url: Option<String>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

pub fn parse_version(raw: &str) -> Option<(u64, u64, u64)> {
    let cleaned = raw.trim().trim_start_matches(['v', 'V']);
    let core = cleaned.split(['-', '+', ' ']).next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u64>());
    let major = parts.next()?.ok()?;
    let minor = parts.next().unwrap_or(Ok(0)).ok()?;
    let patch = parts.next().unwrap_or(Ok(0)).ok()?;
    Some((major, minor, patch))
}

pub async fn check(client: &Client) -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let response = client
        .get(LATEST_RELEASE)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| "GitHub injoignable".to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    let body = response.text().await.map_err(|e| e.to_string())?;
    let release: Release = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let newer = match (parse_version(&release.tag_name), parse_version(&current)) {
        (Some(latest), Some(mine)) => latest > mine,
        _ => false,
    };
    let available = newer && !release.draft && !release.prerelease;
    Ok(UpdateInfo { current, latest: Some(release.tag_name), available, url: Some(release.html_url) })
}

#[cfg(test)]
mod tests {
    use super::parse_version;

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("V1.3"), Some((1, 3, 0)));
        assert_eq!(parse_version("v2.0.1"), Some((2, 0, 1)));
        assert_eq!(parse_version("2.1.0-beta.1"), Some((2, 1, 0)));
        assert_eq!(parse_version("exe"), None);
        assert!(parse_version("v2.0.0") > parse_version("V1.3"));
    }
}
