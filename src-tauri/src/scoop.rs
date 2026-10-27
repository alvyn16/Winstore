use crate::{command, get_featured_apps, inventory::same_package, AppItem, SourceOption};
use std::path::PathBuf;

pub fn root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("SCOOP") {
        return Some(root.into());
    }
    let output = command("where.exe").arg("scoop.ps1").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let shim = PathBuf::from(text.lines().next()?.trim());
    Some(shim.parent()?.parent()?.to_path_buf())
}

pub fn bucket_available(package: &str) -> bool {
    let Some((bucket, name)) = package.split_once('/') else {
        return true;
    };
    root().is_some_and(|root| {
        root.join("buckets")
            .join(bucket)
            .join("bucket")
            .join(format!("{name}.json"))
            .is_file()
    })
}

// Read the same configured bucket manifests Scoop searches, without parsing terminal
// tables or treating a user's query as a regular expression or PowerShell code.
pub fn search_at(root: &std::path::Path, query: &str) -> Result<Vec<AppItem>, String> {
    let buckets = std::fs::read_dir(root.join("buckets"))
        .map_err(|e| format!("Could not read Scoop buckets: {e}"))?;
    let query = query.to_lowercase();
    let catalog = get_featured_apps();
    let mut results = vec![];
    let mut paths: Vec<_> = buckets.flatten().map(|bucket| bucket.path()).collect();
    paths.sort();
    for path in paths {
        let Some(bucket) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(path.join("bucket")) else {
            continue;
        };
        let mut files: Vec<_> = files.flatten().map(|file| file.path()).collect();
        files.sort();
        for file in files {
            if file.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(name) = file.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            // Filter filenames first: opening thousands of manifests per keystroke
            // is especially slow on Windows with real-time antivirus enabled.
            if !name.to_lowercase().contains(&query) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&text) else {
                continue;
            };
            if manifest.get("deprecated").is_some() {
                continue;
            }
            let description = manifest["description"].as_str().unwrap_or("");
            let package_id = format!("{bucket}/{name}");
            if !crate::valid_package_id(&package_id) {
                continue;
            }
            let version = manifest["version"].as_str().unwrap_or("Latest").to_owned();
            let option = SourceOption {
                source: "scoop".into(),
                package_id: package_id.clone(),
                version,
            };
            if let Some(known) = catalog.iter().find(|app| {
                app.sources.iter().any(|source| {
                    source.source == "scoop"
                        && same_package("scoop", &source.package_id, &package_id)
                })
            }) {
                let mut app = known.clone();
                app.sources = vec![option];
                app.default_source = Some("scoop".into());
                results.push(app);
            } else {
                results.push(AppItem {
                    id: format!("scoop:{package_id}"),
                    name: name.into(),
                    publisher: format!("Scoop · {bucket}"),
                    description: description.into(),
                    category: "General".into(),
                    icon: String::new(),
                    banner: None,
                    website: manifest["homepage"]
                        .as_str()
                        .filter(|url| url.starts_with("https://"))
                        .map(str::to_owned),
                    publisher_download: None,
                    screenshots: vec![],
                    is_verified: false,
                    is_installed: false,
                    has_update: false,
                    installations: vec![],
                    default_source: Some("scoop".into()),
                    sources: vec![option],
                    tags: vec!["scoop".into(), bucket.into()],
                });
            }
        }
    }
    results.sort_by_key(|app| {
        (
            !app.name.to_lowercase().contains(&query),
            app.name.to_lowercase(),
        )
    });
    results.truncate(80);
    Ok(results)
}

pub fn search(query: &str) -> Result<Vec<AppItem>, String> {
    let root = root().ok_or("Scoop location was not found. Check Scoop in Package sources.")?;
    search_at(&root, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_search_reads_current_versions_and_preserves_bucket_identity() {
        let root =
            std::env::temp_dir().join(format!("winstore-scoop-search-{}", std::process::id()));
        for bucket in ["main", "custom"] {
            let dir = root.join("buckets").join(bucket).join("bucket");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("git.json"),r#"{"version":"9.0","description":"Git tool","homepage":"https://gitforwindows.org"}"#).unwrap();
        }
        let results = search_at(&root, "GIT").unwrap();
        assert_eq!(results.len(), 2);
        let known = results.iter().find(|app| app.id == "Git.Git").unwrap();
        assert_eq!(known.sources[0].package_id, "main/git");
        assert_eq!(known.sources[0].version, "9.0");
        assert!(results.iter().any(|app| app.id == "scoop:custom/git"));
        assert!(search_at(&root, ".*").unwrap().is_empty());
        std::fs::write(
            root.join("buckets/main/bucket/git.json"),
            r#"{"version":"10.0"}"#,
        )
        .unwrap();
        assert_eq!(
            search_at(&root, "git")
                .unwrap()
                .iter()
                .find(|app| app.id == "Git.Git")
                .unwrap()
                .sources[0]
                .version,
            "10.0"
        );
        // The directory was created by this test under the temporary directory.
        std::fs::remove_dir_all(&root).unwrap();
    }
}
