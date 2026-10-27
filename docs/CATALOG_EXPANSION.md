# Default catalog expansion

The original screenshot expansion supplied 131 apps: the 128 requested screenshot/list entries and 3 existing extras (Dolphin Emulator, Spotify, and PowerToys). WinRAR and 7-Zip are both included once. Amazon Corretto and other Amazon packages are excluded. The screenshot tooltip obscures one browser entry; Zen Browser was included in that position.

The app data is maintained in `src-tauri/catalog/apps.json`, compiled into the Rust app. It is available without performing a live search. New category filters expose Browsers, Communications, Development, Documents, Gaming, Audio & Video, Graphics, and Utilities.

The current catalog additionally includes 13 Artcraft apps (144 total); see `ARTCRAFT_APPS.md`.

## Original screenshot-expansion category counts

| Category | Apps |
| --- | ---: |
| Audio & Video | 14 |
| Browsers | 15 |
| Communications | 18 |
| Development | 37 |
| Documents | 18 |
| Gaming | 18 |
| Graphics | 7 |
| Utilities | 4 |

## Verification and sources

Package identities were cross-checked against the screenshot source configuration (https://github.com/ChrisTitusTech/winutil/blob/main/config/applications.json) and the actual Winget catalog returned by `winget search --source winget --query "" --accept-source-agreements --disable-interactivity` on 2026-10-26. 127 requested entries exist in Winget’s community source. WhatsApp was separately verified with `winget show --id 9NKSQGP7F2NH --exact --source msstore --accept-source-agreements --disable-interactivity`. ChatGPT uses the official `OpenAI.ChatGPT` entry, not similarly named third-party wrappers.

The complete reviewable requested-app mapping and the versions observed during verification are in `catalog-requested-apps.json`. Observed versions are evidence from that run; the UI uses Latest and lets the selected package manager resolve current versions. This does not pin future installations to the recorded versions.

App descriptions are short, independently written summaries. Website/package names and IDs are metadata; no WinUtil application implementation was imported. All curated icons are bundled locally. Individual icon origins are recorded in `catalog-icon-sources.json`; original icons remain the property of their respective projects. Sources include official vendor favicons and repositories, Dashboard Icons, and Google’s favicon cache for a few otherwise unavailable favicons.

## Behavior and limits

Existing source choices are preserved. New catalog entries use their verified Winget package identity, including the Microsoft Store product ID for WhatsApp. No unsupported Scoop/Chocolatey mapping is invented. The install button uses the package runner; installed/update detection now reads all three managers (see `INVENTORY_AND_SCOOP.md`). WinRAR is a vendor-licensed application; inclusion in this catalog does not grant a license.

No applications were installed, uninstalled, or upgraded during this work. Verified package availability is not evidence that every installer/UAC workflow has been exercised.

“zZip” was interpreted as 7-Zip. The existing 7-Zip entry was retained and not duplicated.

For new installs, the source preference is Scoop, then Winget, then Chocolatey. Installed apps open with their detected manager. The catalog now has 109 verified, bucket-qualified Scoop mappings; see `scoop-mappings-verification.json` and `INVENTORY_AND_SCOOP.md`. Unsupported apps retain valid alternatives.
