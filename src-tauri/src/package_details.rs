use serde::{Deserialize, Serialize};
use std::{io::Read, time::Duration};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PackageDetails {
    pub version: Option<String>,
    pub architectures: Vec<String>,
    pub size_bytes: Option<u64>,
    pub release_notes: Option<String>,
    pub release_notes_url: Option<String>,
    pub origin: String,
    pub warning: String,
}
pub fn https_url(value: &str) -> Option<String> {
    reqwest::Url::parse(value)
        .ok()
        .filter(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        })
        .map(|url| url.to_string())
}
pub fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("Winstore/0.1")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())
}
pub fn read_json(url: &str) -> Result<serde_json::Value, String> {
    let url = https_url(url).ok_or("A valid HTTPS URL is required.")?;
    let response = client()?
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    response
        .take(1_000_001)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1_000_000 {
        return Err("The metadata response is too large.".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn size(url: Option<&str>) -> Option<u64> {
    let url = https_url(url?)?;
    client()
        .ok()?
        .head(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}
fn field(text: &str, label: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix(label)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}
fn github_releases(homepage: &str) -> Option<String> {
    let url = reqwest::Url::parse(homepage).ok()?;
    if url.host_str() != Some("github.com") {
        return None;
    }
    let parts: Vec<_> = url
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return None;
    }
    Some(format!(
        "https://github.com/{}/{}/releases",
        parts[0], parts[1]
    ))
}
fn scoop_details(package: &str) -> Result<PackageDetails, String> {
    let (bucket, name) = package
        .split_once('/')
        .ok_or("Scoop metadata needs a bucket-qualified package ID.")?;
    let root = crate::scoop::root().ok_or("Scoop location was not found.")?;
    let bytes=std::fs::read(root.join("buckets").join(bucket).join("bucket").join(format!("{name}.json"))).map_err(|_|format!("The {bucket} bucket is not configured or has no {name} manifest. Select another source or add the bucket in Scoop."))?;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let mut details = PackageDetails {
        version: manifest["version"].as_str().map(str::to_owned),
        origin: format!("Configured Scoop bucket: {bucket}"),
        ..Default::default()
    };
    let host = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86" => "32bit",
        _ => "64bit",
    };
    let architecture = manifest
        .get("architecture")
        .and_then(|value| value.as_object());
    if let Some(architectures) = architecture {
        details.architectures = architectures
            .keys()
            .map(|key| {
                match key.as_str() {
                    "64bit" => "x64",
                    "32bit" => "x86",
                    other => other,
                }
                .to_owned()
            })
            .collect();
    }
    let urls = architecture
        .and_then(|arch| arch.get(host))
        .and_then(|value| value.get("url"))
        .or_else(|| manifest.get("url"));
    let url = urls.and_then(|value| {
        value.as_str().or_else(|| {
            value
                .as_array()
                .filter(|values| values.len() == 1)
                .and_then(|values| values[0].as_str())
        })
    });
    details.size_bytes = size(url);
    details.release_notes_url = manifest["notes_url"]
        .as_str()
        .and_then(https_url)
        .or_else(|| manifest["homepage"].as_str().and_then(github_releases));
    Ok(details)
}
fn details_sync(source: &str, package: &str) -> Result<PackageDetails, String> {
    if !crate::valid_package_id(package) {
        return Err("Invalid package ID.".into());
    }
    match source {
        "publisher" => {
            let app = crate::get_featured_apps()
                .into_iter()
                .find(|app| app.id == package)
                .ok_or("Unknown publisher app.")?;
            let download = app
                .publisher_download
                .ok_or("No publisher release is available.")?;
            let mut details = PackageDetails {
                version: Some(download.version.clone()),
                architectures: vec!["x64".into()],
                size_bytes: Some(download.size_bytes),
                release_notes_url: Some(download.release_url.clone()),
                origin: "Publisher release asset".into(),
                ..Default::default()
            };
            if let Some(path) = download.release_url.strip_prefix("https://github.com/") {
                if let Some((repo, tag)) = path.split_once("/releases/tag/") {
                    match read_json(&format!("https://api.github.com/repos/{repo}/releases/tags/{tag}")) {
                        Ok(release)=>details.release_notes=release["body"].as_str().filter(|text|!text.trim().is_empty()).map(|text|text.chars().take(32_000).collect()),
                        Err(_)=>details.warning="Release notes could not be loaded. The publisher release page is still available.".into(),
                    }
                }
            }
            Ok(details)
        }
        "scoop" => scoop_details(package),
        "winget" => {
            let arch = match std::env::consts::ARCH {
                "aarch64" => "arm64",
                "x86" => "x86",
                _ => "x64",
            };
            let text = crate::process_runner::capture(
                crate::command("winget.exe").args([
                    "show",
                    "--id",
                    package,
                    "--exact",
                    "--architecture",
                    arch,
                    "--accept-source-agreements",
                    "--disable-interactivity",
                ]),
                Duration::from_secs(20),
            )?;
            let mut details = parse_winget(&text);
            details.architectures = vec![format!("{arch} (selected for this PC)")];
            details.size_bytes = size(field(&text, "Installer Url:").as_deref());
            Ok(details)
        }
        "choco" => {
            let text = crate::process_runner::capture(
                crate::command("choco.exe").args(["info", package, "--limit-output", "--no-color"]),
                Duration::from_secs(20),
            )?;
            let version = text
                .lines()
                .filter_map(|line| line.trim().split_once('|'))
                .find(|(id, _)| id.eq_ignore_ascii_case(package))
                .map(|(_, version)| version.to_owned());
            Ok(PackageDetails {
                version,
                origin: "Chocolatey package repository".into(),
                release_notes_url: Some(format!(
                    "https://community.chocolatey.org/packages/{package}"
                )),
                ..Default::default()
            })
        }
        _ => Err("Unsupported package source.".into()),
    }
}
fn parse_winget(text: &str) -> PackageDetails {
    PackageDetails {
        version: field(text, "Version:"),
        release_notes_url: field(text, "Release Notes Url:").and_then(|value| https_url(&value)),
        origin: "Live Winget manifest for this PC".into(),
        ..Default::default()
    }
}
#[tauri::command]
pub async fn get_package_details(
    source: String,
    package_id: String,
) -> Result<PackageDetails, String> {
    tauri::async_runtime::spawn_blocking(move || details_sync(&source, &package_id))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_is_source_specific_and_never_invents_missing_values() {
        let details=parse_winget("Version: 3.2\nRelease Notes Url: https://example.com/releases/3.2\nInstaller Type: exe");
        assert_eq!(details.version.as_deref(), Some("3.2"));
        assert!(details.size_bytes.is_none());
        assert!(details.architectures.is_empty());
        assert!(https_url("http://example.com").is_none());
        assert!(https_url("https://user:password@example.com").is_none());
        assert!(parse_winget("Versión: 3.2").version.is_none());
    }
}
