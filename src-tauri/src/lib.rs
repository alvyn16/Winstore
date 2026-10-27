use serde::{Deserialize, Serialize};
mod app_icons;
mod app_updates;
mod inventory;
mod package_details;
mod process_runner;
mod scoop;
mod tasks;
mod window_appearance;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

fn command(program: &str) -> Command {
    let mut command = Command::new(program_path(program));
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    command
}

fn program_path(program: &str) -> std::path::PathBuf {
    // Desktop launchers can inherit PATH without the Windows app-execution aliases.
    if program == "winget.exe" {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let alias = std::path::PathBuf::from(local).join("Microsoft/WindowsApps/winget.exe");
            if alias.is_file() {
                return alias;
            }
        }
    }
    program.into()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceStatus {
    pub winget: bool,
    pub scoop: bool,
    pub choco: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceOption {
    pub source: String, // "winget", "scoop", "choco"
    pub package_id: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublisherDownload {
    pub url: String,
    pub release_url: String,
    pub version: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppScreenshot {
    pub src: String,
    pub caption: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub banner: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub publisher_download: Option<PublisherDownload>,
    #[serde(default)]
    pub screenshots: Vec<AppScreenshot>,
    pub is_verified: bool,
    pub is_installed: bool,
    pub has_update: bool,
    #[serde(default)]
    pub installations: Vec<inventory::Installation>,
    pub default_source: Option<String>,
    pub sources: Vec<SourceOption>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub output: String,
}

/// Checks which package managers are installed on the user's system
fn check_sources_sync() -> SourceStatus {
    let check_cmd = |cmd: &str| -> bool {
        if program_path(cmd).is_absolute() && program_path(cmd).is_file() {
            return true;
        }
        command("where.exe")
            .arg(cmd)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    };

    SourceStatus {
        winget: check_cmd("winget.exe"),
        scoop: check_cmd("scoop.cmd") && check_cmd("scoop.ps1"),
        choco: check_cmd("choco.exe"),
    }
}

/// Provides curated catalog for the Bazaar-style showcase
#[tauri::command]
fn get_featured_apps() -> Vec<AppItem> {
    serde_json::from_str(include_str!("../catalog/apps.json"))
        .expect("the bundled app catalog must be valid; covered by catalog integrity tests")
}

/// Searches live on the system via the selected package managers
#[cfg(test)]
fn search_apps_sync(query: String, sources: Vec<String>) -> Vec<AppItem> {
    search_apps_result(query, sources).apps
}

#[derive(Serialize)]
struct SearchResult {
    apps: Vec<AppItem>,
    warning: String,
}

fn search_apps_result(query: String, sources: Vec<String>) -> SearchResult {
    let mut warnings = Vec::new();
    let mut results: Vec<AppItem> = Vec::new();
    let query_lower = query.to_lowercase();

    // 1. Search in our curated catalog first for high fidelity metadata
    for app in get_featured_apps() {
        if app.name.to_lowercase().contains(&query_lower)
            || app.description.to_lowercase().contains(&query_lower)
            || app
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower))
        {
            let mut app = app;
            app.sources
                .retain(|option| sources.contains(&option.source));
            if !app.sources.is_empty() || app.publisher_download.is_some() {
                app.default_source = app.sources.first().map(|source| source.source.clone());
                results.push(app);
            }
        }
    }

    if sources.iter().any(|source| source == "scoop") && query.chars().count() >= 2 {
        match scoop::search(&query) {
            Ok(items) => {
                for item in items {
                    if let Some(known) = results.iter_mut().find(|app| app.id == item.id) {
                        for option in item.sources {
                            if let Some(existing) = known
                                .sources
                                .iter_mut()
                                .find(|source| source.source == option.source)
                            {
                                *existing = option;
                            } else {
                                known.sources.push(option);
                            }
                        }
                        known.default_source = Some("scoop".into());
                    } else {
                        results.push(item);
                    }
                }
            }
            Err(error) => warnings.push(error),
        }
    }

    // 2. Query winget CLI if enabled in sources
    if sources.contains(&"winget".to_string()) && query.len() >= 2 {
        if let Ok(output) = command("winget.exe")
            .args([
                "search",
                "--query",
                &query,
                "--source",
                "winget",
                "--accept-source-agreements",
                "--disable-interactivity",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for (name, id, version) in parse_winget_table(&stdout).into_iter().take(40) {
                    if let Some(existing) = results
                        .iter_mut()
                        .find(|app| app.id.eq_ignore_ascii_case(&id))
                    {
                        if let Some(option) = existing
                            .sources
                            .iter_mut()
                            .find(|option| option.source == "winget")
                        {
                            option.version = version;
                        } else {
                            existing.sources.push(SourceOption {
                                source: "winget".into(),
                                package_id: id,
                                version,
                            });
                        }
                    } else {
                        results.push(AppItem {
                            id: id.clone(),
                            name,
                            publisher: "Winget catalog".into(),
                            description: format!("Available from Windows Package Manager ({id})"),
                            category: "General".into(),
                            icon: String::new(),
                            banner: None,
                            website: None,
                            publisher_download: None,
                            screenshots: Vec::new(),
                            is_verified: false,
                            is_installed: false,
                            has_update: false,
                            installations: Vec::new(),
                            default_source: Some("winget".into()),
                            sources: vec![SourceOption {
                                source: "winget".into(),
                                package_id: id,
                                version,
                            }],
                            tags: vec!["winget".into()],
                        });
                    }
                }
            }
        }
    }

    SearchResult {
        apps: results,
        warning: warnings.join(" "),
    }
}

fn prepare_action(
    package_id: String,
    source: String,
    action: &str,
    scope: Option<String>,
) -> Result<Command, String> {
    if !valid_package_id(&package_id) || !["winget", "scoop", "choco"].contains(&source.as_str()) {
        return Err("Invalid package or source".into());
    }
    if source == "scoop" && action == "install" && !scoop::bucket_available(&package_id) {
        return Err(format!("The Scoop bucket for {package_id} is not configured. Add it in Scoop before installing this app."));
    }
    if action != "install" {
        let inventory = inventory::collect_source(&source);
        if !inventory.packages.iter().any(|(_, item)| {
            inventory::same_package(&source, &item.package_id, &package_id)
                && scope.as_deref().is_some_and(|scope| scope == item.scope)
                && (action != "update" || !item.held)
                && !["ambiguous", "overlap"].contains(&item.scope.as_str())
        }) {
            return Err(format!("This installation could not be verified for the selected source. Refresh your apps before trying again. {}", inventory.warnings.join(" ")));
        }
    }
    let (program, mut args): (&str, Vec<&str>) = match source.as_str() {
        "scoop" => (
            "scoop.cmd",
            vec![
                if action == "uninstall" {
                    "uninstall"
                } else if action == "update" {
                    "update"
                } else {
                    "install"
                },
                &package_id,
            ],
        ),
        "choco" => (
            "choco.exe",
            vec![
                if action == "update" {
                    "upgrade"
                } else {
                    action
                },
                &package_id,
                "-y",
            ],
        ),
        _ => {
            let mut args = vec![
                if action == "update" {
                    "upgrade"
                } else {
                    action
                },
                "--id",
                &package_id,
                "--exact",
                "--disable-interactivity",
            ];
            if action != "uninstall" {
                args.extend(["--accept-package-agreements", "--accept-source-agreements"]);
            }
            ("winget.exe", args)
        }
    };
    if source == "scoop" && scope.as_deref() == Some("global") {
        args.push("--global");
    }
    let mut command = command(program);
    command.args(args);
    Ok(command)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let tasks = tasks::TaskManager::load(
                app.handle().clone(),
                app.path().app_data_dir()?.join("tasks.json"),
            )
            .map_err(std::io::Error::other)?;
            app.manage(tasks);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            window_appearance::set_window_appearance,
            check_sources,
            get_featured_apps,
            get_catalog_state,
            search_apps,
            tasks::get_tasks,
            package_details::get_package_details,
            app_updates::get_update_configuration,
            app_icons::resolve_app_icon,
            app_updates::check_winstore_update,
            tasks::enqueue_task,
            tasks::cancel_task,
            tasks::retry_task,
            tasks::clear_task_history,
            tasks::dismiss_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Winstore application");
}

#[tauri::command]
async fn check_sources() -> Result<SourceStatus, String> {
    tauri::async_runtime::spawn_blocking(check_sources_sync)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn search_apps(query: String, sources: Vec<String>) -> Result<SearchResult, String> {
    tauri::async_runtime::spawn_blocking(move || search_apps_result(query, sources))
        .await
        .map_err(|error| error.to_string())
}

fn valid_package_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && !id.starts_with('-')
        && id.split('/').count() <= 2
        && id.split('/').all(|part| {
            part.as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
        })
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._+-/".contains(&c))
}

// Use header column positions, not runs of spaces in data: full-width IDs and
// versions can have only one separating space. Never infer IDs from version fields.
fn parse_winget_columns(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut previous = "";
    let mut columns = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 10 && trimmed.bytes().all(|c| c == b'-') {
            let header: Vec<_> = previous.chars().collect();
            columns = header
                .iter()
                .enumerate()
                .filter_map(|(index, c)| {
                    (!c.is_whitespace()
                        && (index == 0
                            || (index >= 2
                                && header[index - 1] == ' '
                                && header[index - 2] == ' ')))
                        .then_some(index)
                })
                .collect();
        } else if columns.len() >= 3 && !trimmed.is_empty() {
            let chars: Vec<_> = line.chars().collect();
            let mut boundaries = vec![0];
            let mut valid = true;
            for expected in columns.iter().skip(1) {
                if *expected >= chars.len() {
                    boundaries.push(chars.len());
                    continue;
                }
                // Winget's display-width calculation sometimes shifts padded rows
                // by a couple of cells. Snap only to a nearby token boundary.
                let boundary = (expected.saturating_sub(2)..=(*expected + 2).min(chars.len() - 1))
                    .filter(|index| {
                        *index > 0
                            && !chars[*index].is_whitespace()
                            && chars[*index - 1].is_whitespace()
                    })
                    .min_by_key(|index| index.abs_diff(*expected));
                if let Some(index) = boundary {
                    boundaries.push(index);
                } else if chars[*expected].is_whitespace() {
                    // Optional columns (e.g. Available) can be entirely empty.
                    boundaries.push(*expected);
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && boundaries.windows(2).all(|pair| pair[0] <= pair[1]) {
                boundaries.push(chars.len());
                let fields: Vec<String> = boundaries
                    .windows(2)
                    .map(|pair| {
                        chars[pair[0]..pair[1]]
                            .iter()
                            .collect::<String>()
                            .trim()
                            .to_owned()
                    })
                    .collect();
                if fields.len() >= 3
                    && valid_package_id(&fields[1])
                    && fields[1].bytes().any(|c| c.is_ascii_alphabetic())
                {
                    rows.push(fields);
                }
            }
        }
        previous = line;
    }
    rows
}

fn parse_winget_table(text: &str) -> Vec<(String, String, String)> {
    parse_winget_columns(text)
        .into_iter()
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect()
}

#[derive(Serialize)]
pub struct CatalogState {
    apps: Vec<AppItem>,
    warning: String,
}

#[tauri::command]
async fn get_catalog_state() -> Result<CatalogState, String> {
    tauri::async_runtime::spawn_blocking(inventory::catalog_state)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artcraft_apps_are_standalone_and_never_include_the_launcher() {
        let apps = get_featured_apps();
        let artcraft: Vec<_> = apps
            .iter()
            .filter(|app| app.publisher == "Artcraft")
            .collect();
        assert_eq!(artcraft.len(), 13);
        for name in [
            "PhotoCraft",
            "VectorCraft",
            "FilmCraft",
            "LightCraft",
            "PdfCraft",
            "EffectCraft",
            "DesignCraft",
            "FigCraft",
            "WordCraft",
            "SoundCraft",
            "DeckCraft",
            "CADCraft",
            "GridCraft",
        ] {
            let app = artcraft.iter().find(|app| app.name == name).expect(name);
            let download = app
                .publisher_download
                .as_ref()
                .expect("standalone installer");
            let prefix = format!(
                "https://github.com/storytold/{}/releases/download/",
                name.to_ascii_lowercase()
            );
            assert!(download.url.starts_with(&prefix));
            assert!(download.url.ends_with("-windows-x64.msi"));
            assert!(!download.url.to_ascii_lowercase().contains("launcher"));
            assert!(download.size_bytes > 0);
            assert!(!app.screenshots.is_empty());
            for screenshot in &app.screenshots {
                let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../public")
                    .join(screenshot.src.trim_start_matches('/'));
                assert!(path.is_file(), "{}", path.display());
            }
        }
        assert!(!apps
            .iter()
            .any(|app| app.name.eq_ignore_ascii_case("artcraft launcher")));
        let standalone_search = search_apps_sync("Artcraft".into(), vec![]);
        assert_eq!(standalone_search.len(), 13);
        assert!(standalone_search
            .iter()
            .all(|app| app.sources.is_empty() && app.default_source.is_none()));
    }

    #[test]
    fn bundled_catalog_contains_every_requested_app_without_duplicates_or_amazon() {
        let apps = get_featured_apps();
        let requested: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../docs/catalog-requested-apps.json")).unwrap();
        let mut ids = std::collections::HashSet::new();
        for app in &apps {
            assert!(
                ids.insert(app.id.to_ascii_lowercase()),
                "duplicate {}",
                app.id
            );
            assert!(!app.id.to_ascii_lowercase().starts_with("amazon."));
            assert!(!app.name.to_ascii_lowercase().contains("corretto"));
            assert!(
                !app.is_installed && !app.has_update,
                "catalog must not invent machine state"
            );
            if app.sources.is_empty() {
                assert!(
                    app.publisher_download.is_some(),
                    "{} has no download",
                    app.id
                );
                assert!(app.default_source.is_none());
            } else {
                assert!(app
                    .sources
                    .iter()
                    .any(|source| Some(source.source.as_str()) == app.default_source.as_deref()));
            }
            if app.sources.iter().any(|source| source.source == "scoop") {
                assert_eq!(app.default_source.as_deref(), Some("scoop"), "{}", app.id);
            }
            for source in &app.sources {
                assert!(
                    valid_package_id(&source.package_id),
                    "{}",
                    source.package_id
                );
            }
            if !app.icon.is_empty() {
                assert!(
                    app.icon.starts_with("/icons/"),
                    "icon must be bundled: {}",
                    app.id
                );
                let icon = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../public")
                    .join(app.icon.trim_start_matches('/'));
                assert!(
                    icon.is_file(),
                    "missing icon for {}: {}",
                    app.id,
                    icon.display()
                );
            }
        }
        for entry in requested {
            let id = entry["package_id"].as_str().unwrap();
            assert!(
                ids.contains(&id.to_ascii_lowercase()),
                "requested app missing: {id}"
            );
        }
        for id in [
            "RARLab.WinRAR",
            "7zip.7zip",
            "OBSProject.OBSStudio",
            "OpenJS.NodeJS",
            "OpenJS.NodeJS.LTS",
        ] {
            assert!(ids.contains(&id.to_ascii_lowercase()), "{id}");
        }
    }

    #[test]
    fn winget_rows_keep_names_with_spaces_and_skip_truncated_ids() {
        let output = "Nombre                 Id                         Versión\n-----------------------------------------------------------------\nVisual Studio Code     Microsoft.VisualStudioCode  1.0\nÉditeur de texte       Vendor.Editor              2.0\nTruncated              Vendor.Long…               3.0\n";
        assert_eq!(
            parse_winget_table(output),
            vec![
                (
                    "Visual Studio Code".into(),
                    "Microsoft.VisualStudioCode".into(),
                    "1.0".into()
                ),
                (
                    "Éditeur de texte".into(),
                    "Vendor.Editor".into(),
                    "2.0".into()
                ),
            ]
        );
    }

    #[test]
    fn package_ids_reject_options_and_shell_metacharacters() {
        for invalid in [
            "",
            "--all",
            "pkg & calc",
            "pkg;calc",
            "$(calc)",
            "pkg\"",
            "a\nb",
            "../file",
            "main/../../file",
            "main//file",
        ] {
            assert!(!valid_package_id(invalid), "{invalid}");
        }
        for valid in [
            "Microsoft.VisualStudioCode",
            "extras/vlc",
            "nodejs-lts",
            "7zip",
        ] {
            assert!(valid_package_id(valid), "{valid}");
        }
    }

    #[test]
    fn winget_full_columns_keep_long_ids_and_versions_separate() {
        let header = format!(
            "{:<47}{:<40}{:<28}{:<25}{}",
            "Name", "Id", "Version", "Available", "Source"
        );
        let output = format!("{header}\n{}\nHeroic 2.22.0                                  HeroicGamesLauncher.HeroicGamesLauncher 2.22.0                      2.22.4                   winget\nSpotify                                        Spotify.Spotify                         1.2.74.477.g3be53afe        1.3.6.1921.g7086c071a617 winget\n47 upgrades available.\n", "-".repeat(146));
        let rows = parse_winget_columns(&output);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][1], "HeroicGamesLauncher.HeroicGamesLauncher");
        assert_eq!(rows[0][2], "2.22.0");
        assert_eq!(rows[0][3], "2.22.4");
        assert_eq!(rows[1][3], "1.3.6.1921.g7086c071a617");
        assert_eq!(rows[1][4], "winget");
        let installed = format!(
            "{header}\n{}\n{:<47}{:<40}{:<28}{:<25}winget\n",
            "-".repeat(146),
            "Editor",
            "Vendor.Editor",
            "1.0",
            ""
        );
        let rows = parse_winget_columns(&installed);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][2], "1.0");
        assert_eq!(rows[0][3], "");
    }

    #[test]
    fn scoop_catalog_mappings_match_verified_manifest_evidence() {
        let evidence: serde_json::Value =
            serde_json::from_str(include_str!("../../docs/scoop-mappings-verification.json"))
                .unwrap();
        let apps = get_featured_apps();
        let mappings = evidence["mappings"].as_array().unwrap();
        assert_eq!(mappings.len(), 109);
        for item in mappings {
            let app = apps
                .iter()
                .find(|app| Some(app.id.as_str()) == item["id"].as_str())
                .unwrap();
            let source = app
                .sources
                .iter()
                .find(|source| source.source == "scoop")
                .unwrap();
            assert_eq!(
                Some(source.package_id.as_str()),
                item["package_id"].as_str()
            );
            assert_eq!(Some(source.version.as_str()), item["version"].as_str());
        }
    }

    #[test]
    fn catalog_search_respects_disabled_sources() {
        assert!(search_apps_sync("Firefox".into(), vec![]).is_empty());
        let result = search_apps_sync("F".into(), vec!["scoop".into()]);
        assert!(result.iter().any(|app| app.id == "Mozilla.Firefox"));
        assert!(result
            .iter()
            .all(|app| app.sources.iter().all(|source| source.source == "scoop")));
        assert!(result
            .iter()
            .all(|app| app.sources.is_empty() || app.default_source.as_deref() == Some("scoop")));
    }
}
