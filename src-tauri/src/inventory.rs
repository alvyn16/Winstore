use super::{
    command, get_featured_apps, parse_winget_table, valid_package_id, AppItem, CatalogState,
    SourceOption,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub source: String,
    pub package_id: String,
    pub version: String,
    pub available_version: Option<String>,
    pub scope: String,
    pub held: bool,
}

pub struct Inventory {
    pub packages: Vec<(String, Installation)>,
    pub warnings: Vec<String>,
}

fn read_output(program: &str, args: &[&str]) -> Result<String, String> {
    let (result, text) = crate::process_runner::capture_output(
        command(program).args(args),
        std::time::Duration::from_secs(20),
    )?;
    let empty_winget = program == "winget.exe"
        && matches!(
            result.code.map(|code| code as u32),
            Some(0x8A150014 | 0x8A15002B)
        );
    if !result.success && !empty_winget && !(program == "choco.exe" && result.code == Some(2)) {
        return Err(result.status);
    }
    Ok(text.trim_start_matches('\u{feff}').to_owned())
}

pub fn scoop_json(script: &str) -> Result<serde_json::Value, String> {
    let text = read_output(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )?;
    serde_json::from_str(text.trim()).map_err(|e| format!("Unreadable Scoop output: {e}"))
}

fn array(value: &serde_json::Value) -> Vec<&serde_json::Value> {
    match value {
        serde_json::Value::Array(items) => items.iter().collect(),
        serde_json::Value::Object(_) => vec![value],
        _ => vec![],
    }
}

fn string<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
}

pub fn parse_scoop(
    export: &serde_json::Value,
    status: &serde_json::Value,
) -> Vec<(String, Installation)> {
    array(&export["apps"])
        .into_iter()
        .filter_map(|app| {
            let name = string(app, "Name");
            if !valid_package_id(name) || string(app, "Info").contains("Install failed") {
                return None;
            }
            let bucket = string(app, "Source");
            let id = if valid_package_id(bucket) {
                format!("{bucket}/{name}")
            } else {
                name.into()
            };
            let latest = array(status)
                .into_iter()
                .find(|row| string(row, "Name").eq_ignore_ascii_case(name))
                .map(|row| string(row, "Latest Version"))
                .filter(|version| !version.is_empty())
                .map(str::to_owned);
            Some((
                name.into(),
                Installation {
                    source: "scoop".into(),
                    package_id: id,
                    version: string(app, "Version").into(),
                    available_version: latest,
                    scope: if string(app, "Info").contains("Global install") {
                        "global"
                    } else {
                        "user"
                    }
                    .into(),
                    held: string(app, "Info").contains("Held package"),
                },
            ))
        })
        .collect()
}

pub fn parse_choco(list: &str, outdated: &str) -> Vec<(String, Installation)> {
    list.lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.trim().split('|').collect();
            if fields.len() != 2 || !valid_package_id(fields[0]) {
                return None;
            }
            let update = outdated
                .lines()
                .map(|line| line.trim().split('|').collect::<Vec<_>>())
                .find(|row| row.len() == 4 && row[0].eq_ignore_ascii_case(fields[0]));
            Some((
                fields[0].into(),
                Installation {
                    source: "choco".into(),
                    package_id: fields[0].into(),
                    version: fields[1].into(),
                    available_version: update.as_ref().map(|row| row[2].to_owned()),
                    scope: "machine".into(),
                    held: update.is_some_and(|row| row[3].eq_ignore_ascii_case("true")),
                },
            ))
        })
        .collect()
}

pub fn collect_source(source: &str) -> Inventory {
    let mut inventory = Inventory {
        packages: vec![],
        warnings: vec![],
    };
    let result: Result<_, String> = (|| {
        match source {
            "scoop" => {
                let export = scoop_json(
                    "[Console]::OutputEncoding=[Text.UTF8Encoding]::new(); & scoop.ps1 export",
                )?;
                let status = scoop_json("[Console]::OutputEncoding=[Text.UTF8Encoding]::new(); ConvertTo-Json -InputObject @(& scoop.ps1 status --local 6>$null) -Depth 6 -Compress")
                    .unwrap_or_else(|_| { inventory.warnings.push("Could not check Scoop updates; installed packages are still shown.".into()); serde_json::Value::Null });
                inventory.packages = parse_scoop(&export, &status);
            }
            "choco" => {
                let list = read_output("choco.exe", &["list", "--limit-output"])?;
                let outdated = read_output("choco.exe", &["outdated", "--limit-output"]).unwrap_or_else(|_| {
                    inventory.warnings.push("Could not check Chocolatey updates; installed packages are still shown.".into()); String::new()
                });
                inventory.packages = parse_choco(&list, &outdated);
            }
            "winget" => {
                let list = read_output(
                    "winget.exe",
                    &[
                        "list",
                        "--accept-source-agreements",
                        "--disable-interactivity",
                    ],
                )?;
                let updates = read_output(
                    "winget.exe",
                    &[
                        "upgrade",
                        "--accept-source-agreements",
                        "--disable-interactivity",
                    ],
                )
                .map(|text| super::parse_winget_columns(&text))
                .unwrap_or_else(|_| {
                    inventory.warnings.push(
                        "Could not check Winget updates; installed packages are still shown."
                            .into(),
                    );
                    vec![]
                });
                inventory.packages = parse_winget_table(&list)
                    .into_iter()
                    .map(|(name, id, version)| {
                        let latest = updates
                            .iter()
                            .find(|row| row.len() >= 4 && row[1].eq_ignore_ascii_case(&id))
                            .map(|row| row[3].clone());
                        (
                            name,
                            Installation {
                                source: "winget".into(),
                                package_id: id,
                                version,
                                available_version: latest,
                                scope: "detected".into(),
                                held: false,
                            },
                        )
                    })
                    .collect();
                let mut unique: Vec<(String, Installation)> = vec![];
                for (name, item) in std::mem::take(&mut inventory.packages) {
                    if let Some((_, existing)) = unique.iter_mut().find(|(_, existing)| {
                        existing.package_id.eq_ignore_ascii_case(&item.package_id)
                    }) {
                        existing.scope = "ambiguous".into();
                        if !existing
                            .version
                            .split(" / ")
                            .any(|version| version == item.version)
                        {
                            existing.version.push_str(&format!(" / {}", item.version));
                        }
                    } else {
                        unique.push((name, item));
                    }
                }
                inventory.packages = unique;
                if inventory.packages.is_empty() {
                    inventory.warnings.push(
                        "Winget returned no readable packages. Its inventory may be incomplete."
                            .into(),
                    );
                }
            }
            _ => return Err("Unknown package manager".into()),
        }
        Ok(())
    })();
    if let Err(error) = result {
        inventory
            .warnings
            .push(format!("Could not read {source} inventory ({error})."));
    }
    inventory
}

pub fn same_package(source: &str, left: &str, right: &str) -> bool {
    if source == "scoop" && (!left.contains('/') || !right.contains('/')) {
        left.rsplit('/')
            .next()
            .unwrap_or(left)
            .eq_ignore_ascii_case(right.rsplit('/').next().unwrap_or(right))
    } else {
        left.eq_ignore_ascii_case(right)
    }
}

pub fn apply_inventory(apps: &mut Vec<AppItem>, inventory: Inventory) {
    for (name, installation) in inventory.packages {
        let known = apps.iter_mut().find(|app| {
            app.sources.iter().any(|option| {
                option.source == installation.source
                    && same_package(&option.source, &option.package_id, &installation.package_id)
            })
        });
        if let Some(app) = known {
            if let Some(option) = app
                .sources
                .iter_mut()
                .find(|option| option.source == installation.source)
            {
                option.version = installation
                    .available_version
                    .clone()
                    .unwrap_or_else(|| installation.version.clone());
            }
            app.installations.push(installation);
        } else {
            let option = SourceOption {
                source: installation.source.clone(),
                package_id: installation.package_id.clone(),
                version: installation
                    .available_version
                    .clone()
                    .unwrap_or_else(|| installation.version.clone()),
            };
            apps.push(AppItem {
                id: format!(
                    "installed:{}:{}",
                    installation.source, installation.package_id
                ),
                name,
                publisher: format!("{} inventory", installation.source),
                description: format!("Installed package · {}", installation.version),
                category: "General".into(),
                icon: String::new(),
                banner: None,
                website: None,
                publisher_download: None,
                screenshots: vec![],
                is_verified: false,
                is_installed: true,
                has_update: false,
                default_source: Some(installation.source.clone()),
                sources: vec![option],
                tags: vec![],
                installations: vec![installation],
            });
        }
    }
    for app in apps {
        // Preserve separate versions while avoiding duplicate actions on an
        // overlapping Winget registration of an explicitly managed package.
        let managed_versions: Vec<_> = app
            .installations
            .iter()
            .filter(|item| item.source != "winget")
            .map(|item| item.version.clone())
            .collect();
        for item in &mut app.installations {
            if item.source == "winget" && managed_versions.contains(&item.version) {
                item.scope = "overlap".into();
            }
        }
        app.is_installed = !app.installations.is_empty();
        app.has_update = app.installations.iter().any(|item| {
            item.available_version.is_some()
                && !item.held
                && !["ambiguous", "overlap"].contains(&item.scope.as_str())
        });
    }
}

pub fn catalog_state() -> CatalogState {
    let status = super::check_sources_sync();
    let mut apps = get_featured_apps();
    let mut warnings = vec![];
    std::thread::scope(|scope| {
        let jobs: Vec<_> = [
            ("scoop", status.scoop),
            ("choco", status.choco),
            ("winget", status.winget),
        ]
        .into_iter()
        .filter(|(_, available)| *available)
        .map(|(source, _)| scope.spawn(move || collect_source(source)))
        .collect();
        for job in jobs {
            match job.join() {
                Ok(inventory) => {
                    warnings.extend(inventory.warnings.clone());
                    apply_inventory(&mut apps, inventory);
                }
                Err(_) => {
                    warnings.push("A package inventory check failed. Refresh to try again.".into())
                }
            }
        }
    });
    if !status.scoop && !status.winget && !status.choco {
        warnings.push("No package managers detected. Installed status is unavailable.".into());
    }
    CatalogState {
        apps,
        warning: warnings.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scoop_inventory_keeps_bucket_scope_holds_and_skips_failed_installs() {
        let export = json!({"apps":[
            {"Name":"git","Version":"2.0","Source":"main","Info":"Global install, Held package"},
            {"Name":"7zip","Version":"1.0","Source":"main","Info":""},
            {"Name":"bad","Info":"Install failed"}
        ]});
        let rows = parse_scoop(&export, &json!([{"Name":"git","Latest Version":"3.0"}]));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1.package_id, "main/git");
        assert_eq!(rows[0].1.scope, "global");
        assert!(rows[0].1.held);
        assert_eq!(rows[0].1.available_version.as_deref(), Some("3.0"));
        assert_eq!(rows[1].1.scope, "user");
        assert!(rows[1].1.available_version.is_none());
        assert!(parse_scoop(&json!({"apps":[]}), &json!([])).is_empty());
    }

    #[test]
    fn choco_inventory_reads_versions_and_pins_without_parsing_log_lines() {
        let rows = parse_choco(
            "Chocolatey v2\nnodejs.install|22.1\nvscode|1.0\n",
            "nodejs.install|22.1|24.0|false\nvscode|1.0|2.0|true",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1.version, "22.1");
        assert_eq!(rows[0].1.available_version.as_deref(), Some("24.0"));
        assert!(rows[1].1.held);
    }

    #[test]
    fn explicit_manager_owns_curated_app_and_multiple_copies_remain_distinct() {
        let mut apps = get_featured_apps();
        for (source, id, scope) in [
            ("scoop", "main/git", "user"),
            ("scoop", "main/git", "global"),
            ("winget", "Git.Git", "detected"),
        ] {
            apply_inventory(
                &mut apps,
                Inventory {
                    packages: vec![(
                        "Git".into(),
                        Installation {
                            source: source.into(),
                            package_id: id.into(),
                            version: "1".into(),
                            available_version: Some("2".into()),
                            scope: scope.into(),
                            held: false,
                        },
                    )],
                    warnings: vec![],
                },
            );
        }
        let git = apps.iter().find(|app| app.id == "Git.Git").unwrap();
        assert_eq!(git.installations.len(), 3);
        assert_eq!(
            git.installations
                .iter()
                .filter(|item| item.source == "scoop")
                .count(),
            2
        );
        assert_eq!(
            git.installations
                .iter()
                .find(|item| item.source == "winget")
                .unwrap()
                .scope,
            "overlap"
        );
        assert!(git.is_installed && git.has_update);
        assert!(!same_package("scoop", "main/git", "custom/git"));
        assert!(same_package("scoop", "git", "main/git"));
    }
}
