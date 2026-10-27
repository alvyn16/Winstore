use crate::AppItem;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{
    collections::HashMap,
    io::Read,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
    time::{Duration, SystemTime},
};
use tauri::Manager;

const MAX_ICON_BYTES: usize = 1_000_000;
const CACHE_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);
static WRITES: AtomicU64 = AtomicU64::new(0);
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}
fn index() -> &'static HashMap<String, Option<String>> {
    static INDEX: OnceLock<HashMap<String, Option<String>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let slugs: Vec<String> = serde_json::from_str(include_str!("../catalog/icon-index.json"))
            .expect("bundled icon index");
        let mut index = HashMap::new();
        for slug in slugs {
            let key = normalize(&slug);
            index
                .entry(key)
                .and_modify(|value| *value = None)
                .or_insert(Some(slug));
        }
        index
    })
}
fn candidates(app: &AppItem) -> Vec<String> {
    let mut candidates = vec![];
    for source in &app.sources {
        candidates.push(normalize(&source.package_id));
        if let Some(last) = source.package_id.rsplit(['.', '/']).next() {
            if ![
                "app",
                "desktop",
                "client",
                "server",
                "cli",
                "runtime",
                "install",
                "installer",
            ]
            .contains(&last.to_ascii_lowercase().as_str())
            {
                candidates.push(normalize(last));
            }
        }
    }
    candidates.push(normalize(&app.name));
    let cleaned = app.name.split(" (").next().unwrap_or(&app.name);
    let without_version = cleaned
        .split_whitespace()
        .take_while(|word| {
            !word.as_bytes().first().is_some_and(u8::is_ascii_digit)
                && !word.starts_with("version")
                && !word.starts_with("versión")
        })
        .collect::<Vec<_>>()
        .join(" ");
    candidates.push(normalize(&without_version));
    candidates
}
fn bundled(app: &AppItem) -> Option<String> {
    if candidates(app)
        .iter()
        .any(|candidate| candidate == "localsend" || candidate == "localsendcli")
    {
        return Some("/icons/extra/localsend.png".into());
    }
    crate::get_featured_apps()
        .into_iter()
        .find(|known| {
            !known.icon.is_empty()
                && (known.id.eq_ignore_ascii_case(&app.id)
                    || known.sources.iter().any(|source| {
                        app.sources.iter().any(|other| {
                            source.source == other.source
                                && crate::inventory::same_package(
                                    &source.source,
                                    &source.package_id,
                                    &other.package_id,
                                )
                        })
                    })
                    || normalize(&known.name) == normalize(&app.name))
        })
        .map(|app| app.icon)
}
fn valid_png(bytes: &[u8]) -> bool {
    if bytes.len() < 32
        || bytes.len() > MAX_ICON_BYTES
        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.get(12..16) != Some(b"IHDR")
    {
        return false;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (1..=2048).contains(&width)
        && (1..=2048).contains(&height)
        && bytes.ends_with(b"IEND\xaeB`\x82")
}
fn data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}
fn cache_key(app: &AppItem) -> String {
    // Stable, opaque filename: app names and package IDs never become paths.
    let identity = format!(
        "{}:{}",
        app.name,
        app.sources
            .iter()
            .map(|source| format!("{}:{}", source.source, source.package_id))
            .collect::<Vec<_>>()
            .join("|")
    );
    let hash = identity.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    format!("{hash:016x}.png")
}
fn save(cache: &Path, bytes: &[u8]) {
    if let Some(parent) = cache.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let temp = cache.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::Relaxed)
    ));
    if std::fs::write(&temp, bytes).is_ok() && std::fs::rename(&temp, cache).is_err() {
        let _ = std::fs::remove_file(temp);
    }
    // Bound storage to 256 images, deleting only PNGs in this app's icon cache.
    if let Some(parent) = cache.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            let mut icons: Vec<_> = entries
                .flatten()
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "png")
                })
                .filter_map(|entry| {
                    entry.metadata().ok().map(|metadata| {
                        (
                            entry.path(),
                            metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        )
                    })
                })
                .collect();
            if icons.len() > 256 {
                icons.sort_by_key(|(_, modified)| *modified);
                let excess = icons.len() - 256;
                for (path, _) in icons.into_iter().take(excess) {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}
fn installed(app: &AppItem) -> Option<Vec<u8>> {
    if !app.is_installed {
        return None;
    }
    let mut command = crate::command("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("installed_icon.ps1"),
        ])
        .env("WINSTORE_ICON_NAME", &app.name);
    let text = crate::process_runner::capture(&mut command, Duration::from_secs(6)).ok()?;
    let bytes = STANDARD.decode(text.trim()).ok()?;
    valid_png(&bytes).then_some(bytes)
}
fn resolve(app: &AppItem, cache_dir: &Path, skip_bundled: bool) -> Option<String> {
    if !skip_bundled {
        if let Some(path) = bundled(app) {
            return Some(path);
        }
    }
    let cache = cache_dir.join(cache_key(app));
    let cached = std::fs::metadata(&cache)
        .ok()
        .filter(|metadata| metadata.len() <= MAX_ICON_BYTES as u64)
        .and_then(|_| std::fs::read(&cache).ok())
        .filter(|bytes| valid_png(bytes));
    let fresh = std::fs::metadata(&cache)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age < CACHE_AGE);
    if fresh {
        if let Some(bytes) = &cached {
            return Some(data_url(bytes));
        }
    }
    let local = installed(app);
    let bytes = local.or_else(|| {
        let slug = candidates(app)
            .into_iter()
            .find_map(|candidate| index().get(&candidate).cloned().flatten())?;
        let response = crate::package_details::client()
            .ok()?
            .get(format!(
                "https://raw.githubusercontent.com/selfhst/icons/main/png/{slug}.png"
            ))
            .send()
            .ok()?
            .error_for_status()
            .ok()?;
        let mut bytes = vec![];
        response
            .take((MAX_ICON_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .ok()?;
        valid_png(&bytes).then_some(bytes)
    });
    if let Some(bytes) = bytes {
        save(&cache, &bytes);
        return Some(data_url(&bytes));
    }
    cached.map(|bytes| data_url(&bytes))
}
#[tauri::command]
pub async fn resolve_app_icon(
    app: AppItem,
    skip_bundled: bool,
    handle: tauri::AppHandle,
) -> Result<Option<String>, String> {
    if app.name.len() > 512 || app.sources.len() > 32 {
        return Ok(None);
    }
    let directory = handle
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("icons-v1");
    tauri::async_runtime::spawn_blocking(move || resolve(&app, &directory, skip_bundled))
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_sources_reuse_bundled_icons_and_localsend_is_offline_ready() {
        let mut app = crate::get_featured_apps().remove(0);
        app.id = "installed:winget:Microsoft.VisualStudioCode".into();
        app.icon.clear();
        assert_eq!(bundled(&app).as_deref(), Some("/icons/app-0.ico"));
        app.id = "scoop:extras/localsend".into();
        app.name = "localsend".into();
        app.sources[0].package_id = "extras/localsend".into();
        assert_eq!(bundled(&app).as_deref(), Some("/icons/extra/localsend.png"));
        assert!(valid_png(include_bytes!(
            "../../public/icons/extra/localsend.png"
        )));
        app.name = "LocalSend CLI".into();
        app.sources[0].package_id = "extras/localsend-cli".into();
        assert_eq!(bundled(&app).as_deref(), Some("/icons/extra/localsend.png"));
    }
    #[test]
    fn index_matches_case_and_punctuation_but_never_guesses_unknown_apps() {
        assert_eq!(
            index()
                .get(&normalize("qBittorrent"))
                .cloned()
                .flatten()
                .as_deref(),
            Some("qbittorrent")
        );
        assert!(!index().contains_key(&normalize("nonexistent-software-123456")));
        assert!(!valid_png(b"<html>error</html>"));
        let mut app = crate::get_featured_apps().remove(0);
        app.name = "../../unsafe/name".into();
        assert!(!cache_key(&app).contains('/'));
    }
    #[test]
    fn cached_icons_work_without_a_provider_match_and_corrupt_cache_is_rejected() {
        let directory =
            std::env::temp_dir().join(format!("winstore-icon-cache-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut app = crate::get_featured_apps().remove(0);
        app.id = "unknown-cache-test".into();
        app.name = "Unknown cache test 12345".into();
        app.icon.clear();
        app.sources.clear();
        app.is_installed = false;
        let file = directory.join(cache_key(&app));
        let bytes = include_bytes!("../../public/icons/extra/localsend.png");
        std::fs::write(&file, bytes).unwrap();
        assert_eq!(resolve(&app, &directory, true), Some(data_url(bytes)));
        std::fs::write(&file, b"not an icon").unwrap();
        assert!(resolve(&app, &directory, true).is_none());
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
