# Installed inventory and Scoop search

Implemented October 27, 2026. The bundled catalog remains 144 apps; packages detected outside it appear in Installed without expanding the curated Explore collection.

## Installed packages and actions

- Scoop: structured `scoop export` records include bucket, installed version, user/global scope, and held status. `scoop status --local` supplies available versions without fetching or changing buckets.
- Chocolatey: `choco list --limit-output` and `choco outdated --limit-output` provide actual package IDs, versions, updates, and pins. Enhanced exit code 2 is handled as a normal result.
- Winget: list/upgrade tables are parsed by localized header column positions, including blank columns and long IDs/versions separated by a single space. Its standard no-results/no-update exit codes are handled. The WindowsApps execution alias is found even when missing from inherited PATH.
- Explicit Scoop/Chocolatey ownership and Winget registration are distinct evidence. Different installed versions remain visible as separate source choices. Same-version overlaps have the Winget action disabled; duplicate Winget registrations are marked ambiguous and must be managed in Windows Settings. Winstore does not infer the original installer from Winget detection.
- Opening an installed app prefers its detected manager. Actions are based on the selected copy, not the app-wide installed flag. Update and uninstall validate package ID, manager, and scope against fresh inventory. Scoop global copies retain `--global`; pinned/held updates are blocked. Uninstall remains available separately when an update exists.
- Inventory is read again after each task. CLI success is reported in the task queue; installed/update badges come from readback, not an optimistic toggle. Disabled sources do not hide installed inventory.
- A verified local Chocolatey `nodejs.install` identity is also mapped to the Node.js catalog entry; the Chocolatey `nodejs` metapackage remains a distinct installed package.

## Scoop mappings and search

109 catalog mappings were verified against official Main, Extras, Versions, and the known Games bucket manifests. Bucket-qualified IDs prevent similarly named packages from another bucket being treated as the curated app. The audit lists package IDs, manifest URLs, versions, homepages, and rejected candidates in `scoop-mappings-verification.json`. The package data is a verification snapshot, not a network updater.

Live search reads configured Scoop bucket manifests on every search; there is no stale application cache. It matches package names literally, preserves each bucket's identity, reads the current manifest version, and retains curated icons/metadata for exact verified mappings. It opens only matching manifest files, rather than parsing every JSON file on every keystroke. Results are limited to 80 live Scoop entries; curated matches are kept too. Queries need at least two characters for live manager searches.

Scoop searches use locally configured buckets. They do not silently add buckets, update repositories, or search every remote bucket. Run `scoop update` to refresh manifests. An install from an unconfigured bucket fails with an actionable message before launching Scoop. Chocolatey live repository search remains outside this change.

## Validation

Backend tests cover structured inventory, scopes/holds, manager overlap, bucket identity, live manifest changes, literal search, invalid IDs, long Winget columns, disabled sources, verified mapping evidence, and existing catalog/Artcraft guarantees. Frontend production build and strict Rust Clippy are required.

Actual Tauri WebView checks exercise installed counts, selected manager/version, ambiguous registration protection, Scoop-only search for a noncurated package, and layouts at 320/768/1200px. Package-action commands are intercepted in the UI test to verify exact IDs and scopes; post-action readback still uses real inventory. No software is installed, updated, or removed by this verification. Installer execution, elevation, and Windows-native window visuals are not proven by these tests.

Upstream formats: [Scoop export](https://github.com/ScoopInstaller/Scoop/blob/master/libexec/scoop-export.ps1), [Scoop list](https://github.com/ScoopInstaller/Scoop/blob/master/libexec/scoop-list.ps1), [Scoop status](https://github.com/ScoopInstaller/Scoop/blob/master/libexec/scoop-status.ps1), [Scoop search](https://github.com/ScoopInstaller/Scoop/blob/master/libexec/scoop-search.ps1), [Chocolatey list](https://docs.chocolatey.org/en-us/choco/commands/list/), [Chocolatey outdated](https://docs.chocolatey.org/en-us/choco/commands/outdated/), [Winget exit codes](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerSharedLib/Public/AppInstallerErrors.h).
