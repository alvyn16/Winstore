use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub configured: bool,
    pub version: Option<String>,
    pub download_url: Option<String>,
    pub notes: Option<String>,
}
#[tauri::command]
pub fn get_update_configuration() -> UpdateInfo {
    UpdateInfo {
        current_version: env!("CARGO_PKG_VERSION").into(),
        configured: option_env!("WINSTORE_UPDATE_FEED_URL").is_some_and(|url| !url.is_empty()),
        version: None,
        download_url: None,
        notes: None,
    }
}
fn parse_feed(value: serde_json::Value, current: &str) -> Result<UpdateInfo, String> {
    let mut info = get_update_configuration();
    info.current_version = current.into();
    info.configured = true;
    let version = value["version"]
        .as_str()
        .ok_or("The update feed has no version.")?
        .trim_start_matches('v');
    let available =
        semver::Version::parse(version).map_err(|_| "The update feed has an invalid version.")?;
    let current = semver::Version::parse(current).map_err(|_| "Invalid current app version.")?;
    if available <= current {
        return Ok(info);
    }
    let target = format!("windows-{}", std::env::consts::ARCH);
    let url = value["platforms"][&target]["url"]
        .as_str()
        .or_else(|| value["download_url"].as_str())
        .ok_or("No installer is provided for this PC's architecture.")?;
    info.download_url =
        Some(crate::package_details::https_url(url).ok_or("The installer URL must use HTTPS.")?);
    info.version = Some(version.into());
    info.notes = value["notes"]
        .as_str()
        .map(|notes| notes.chars().take(32_000).collect());
    Ok(info)
}
#[tauri::command]
pub async fn check_winstore_update() -> Result<UpdateInfo, String> {
    let endpoint = option_env!("WINSTORE_UPDATE_FEED_URL")
        .filter(|url| !url.is_empty())
        .ok_or("This build has no update feed configured.")?;
    tauri::async_runtime::spawn_blocking(move || {
        parse_feed(
            crate::package_details::read_json(endpoint)?,
            env!("CARGO_PKG_VERSION"),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn manual_updates_require_newer_versions_and_https_downloads() {
        assert!(parse_feed(
            json!({"version":"99.0.0","download_url":"http://example.com/app.exe"}),
            "0.1.0"
        )
        .is_err());
        assert!(parse_feed(json!({"version":"0.0.1"}), "0.1.0")
            .unwrap()
            .version
            .is_none());
        let update=parse_feed(json!({"version":"0.2.0","download_url":"https://example.com/app.exe","notes":"Changes"}),"0.1.0").unwrap();
        assert_eq!(update.version.as_deref(), Some("0.2.0"));
        assert!(update.download_url.is_some());
    }
}
