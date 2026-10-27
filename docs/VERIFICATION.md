# Verification results

Verified on Windows on 2026-10-26 against the running Tauri application's actual WebView2 page through a temporary local CDP debugging connection. Catalog and package-manager read requests used the Rust backend; the UI was not supplied mock app data.

- `npm run build`: passed after the final UI/artwork changes.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 3 passed (localized Winget table parsing, package ID validation, disabled-source catalog filtering).
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- Desktop WebView smoke checks: dark/light themes and theme persistence; all carousel slides; Show More/Less; app details; Escape dismissal and focus restoration; dialog Tab containment; source dialog; empty task drawer; category filtering and return navigation; Installed and Updates views; real Winget search; no-result searches; and disabled-source searches all passed.
- Responsive checks: no horizontal document overflow at 320, 768, 1024, and 1440px. The desktop viewport was restored to 1200x800 for handoff.
- All displayed app icons loaded. No page errors or failed HTTP responses were recorded during the smoke pass.
- Live inventory in this verification run detected 8 installed curated apps and 6 available updates. These are observations of that run, not hardcoded UI data.
- IPC boundary checks confirmed install/uninstall/update reject `--all` as a package ID and reject unknown package sources before executing a process.
- No software was installed, removed, or upgraded during verification. Successful installer/UAC workflows, cancellation, and recovery after closing Winstore remain unverified. This earlier run predates the multi-manager inventory and Scoop search described below. Chocolatey live repository search remains unimplemented.

Screenshots in `docs/screenshots/` are actual rendered Winstore WebView captures: dark, light, gaming, details, and narrow viewport. They do not include the Windows title bar. The original Bazaar reference photos remain in `assets/ui-reference/`.


## App details and developer artwork follow-up

- Final frontend production build passed.
- Checked the actual running Tauri WebView with real backend data. Both themes were rendered and visually inspected, with the details dialog verified to occupy over 90 percent of the desktop viewport width.
- Passed screenshot carousel next/previous, nested screenshot enlargement, Escape dismissing only the enlargement, details Escape/back/close, and focus restoration.
- Passed saved-app toggle and restoration, source switching reflected in the primary action, honest missing-preview state, developer banner loading, developer category navigation, and layouts at 320, 768, 1024, and 1440px.
- No page errors or failed HTTP responses occurred in the final interaction pass. No packages were installed, updated, or uninstalled. Rust was unchanged in this follow-up, so the earlier Rust verification was not repeated.
- New real WebView captures: `winstore-app-details-dark.png`, `winstore-app-details-light.png`, `winstore-app-details-narrow.png`, and `winstore-developer.png` under `docs/screenshots/`.

## Compact layout, themed title bar, and app icon

- Frontend production build, Rust library tests (3), and strict Clippy passed after the window-theme changes. The application was rebuilt and restarted with the new icon resources and window configuration.
- VLC details now use a 42px missing-preview notice. At a 1200x800 viewport the dialog measured approximately 580px high, with About/source selection fully visible and no reserved blank area below the content.
- Verified exact custom title-bar backgrounds: dark `rgb(34, 34, 38)` and light `rgb(255, 255, 255)`. OS decorations are disabled; the themed bar invokes native window actions.
- Maximize, restore, minimize, unminimize, and double-click maximize/restore were exercised on the actual Tauri window while its app-details modal was open. Window controls remain in the active modal's top layer. The close button binds to Tauri's close action; the final window was deliberately left open.
- Checked the icon loads, galleries still work, source dialogs retain window controls, and no horizontal dialog overflow at 320/768/1024/1440px. No page errors, console errors, or failed HTTP responses were recorded in the final window-control smoke pass.
- Generated the icon with ImageGen and exported ICO/PNG/platform sizes with Tauri's icon command. The 32px icon was visually inspected. Prompt and source reference: `WINSTORE_ICON_PROMPT.md`.
- Final captures: `docs/screenshots/winstore-compact-dark.png` and `winstore-compact-light.png`. These WebView captures include the new app-drawn title bar. Native desktop capture remains unavailable because the Computer Use native pipe cannot be reached.


## Default catalog expansion

- Final catalog: 131 apps, including all 128 requested entries and 3 preserved extras; no duplicate package identities and no Amazon packages.
- Read-only verification: 127 requested packages found in the local Winget community catalog; WhatsApp’s Store product ID verified separately. Mapping evidence: `catalog-requested-apps.json`.
- Rust integrity test checks all requested IDs, valid package identifiers, locally bundled icons, default source consistency, and absence of fabricated installed/update state. Four Rust tests and strict Clippy pass.
- Actual desktop WebView checks pass for all eight category counts, Show More exposing 131 cards, decoding every curated icon, WinRAR details and website/source, 7-Zip deduplication, and responsive widths from 320 through 1440px. No page errors or failed asset responses occurred.
- No package actions were executed. The app was rebuilt and left running with the updated bundled catalog.

## Modal background scrolling fix

- Frontend production build passed. In the actual desktop WebView, app details, package-source settings, and the task drawer lock document scrolling while retaining their own scrolling.
- Checked wheel input over the backdrop and at the bottom of the details dialog: the underlying page remains at its original scroll position.
- Opening a screenshot enlargement acquires a second lock; closing it keeps the parent dialog's lock active. Closing the final dialog restores the previous scroll position and overflow setting, and normal wheel scrolling resumes.
- Verified the page's rendered content width does not change as the scrollbar is hidden. The root uses a stable scrollbar gutter, and dialog overscroll is contained.
- The regression check starts with a nonzero page scroll position and exercises details, nested screenshots, settings, and the queue. No page errors occurred. Screenshot: `docs/screenshots/winstore-dialog-scroll-lock.png`.


## Artcraft standalone app expansion

- Added 13 app entries, with no Artcraft launcher; total catalog 144. Seven actual Winget IDs and their standalone installer manifests were verified. All 13 standalone x64 MSI URLs were matched to official GitHub release assets.
- Five Rust tests pass, including exclusion of launcher assets, required standalone downloads/previews, and availability of publisher downloads when package-manager sources are disabled. Strict Clippy and the final frontend build pass.
- Actual desktop catalog/details rendering was checked for all 13 apps. All icons and previews decode. The primary Download button is enabled for publisher-only apps, and no fake package source or completed installation is shown.
- All 13 secondary download buttons were exercised with a test-only opener-module stub and checked against the exact standalone MSI URL. The stub was removed and the page reloaded before handoff. A native URL opening was observed once in the initial check; no installers were executed.
- Checked the dedicated collection, responsive widths, and no page errors or failed assets. Captures: `winstore-artcraft-collection.png` and `winstore-photocraft-details.png`.


## Multi-manager inventory and Scoop search

- 109 Scoop mappings verified against upstream manifests; curated app count remains 144. Bucket-qualified names replace ambiguous IDs.
- Eleven Rust tests, production frontend build, and strict Clippy passed. Tests cover inventory/scopes/holds, manager overlap, live manifest changes and bucket identity, long and empty Winget columns, mapping evidence, and existing catalog guarantees.
- Actual Tauri WebView: all three sources detected, 128 installed app/package groups and 49 update groups in the observed snapshot, no inventory warnings. Counts are machine observations, not fixtures or hardcoded UI data.
- Scoop-only live search returns noncurated `main/ripgrep`, with current local manifest metadata. Read-only direct searches took about 80–105 ms in the measured run.
- Safe UI action routing verified `main/git` + Scoop/user and `nodejs.install` + Chocolatey/machine. Action commands were stubbed; subsequent inventory was real, and simulated uninstall success did not falsely remove Node.js. Ambiguous Oh My Posh registrations had their action disabled.
- Checked 320/768/1200px layouts with no horizontal overflow or page errors. Module stubs were removed and real modules restored before handoff. No software was installed, updated, or removed.
- Screenshots: `winstore-scoop-installed.png`, `winstore-live-scoop-search.png`. See `INVENTORY_AND_SCOOP.md` for limitations.


## Queue, package metadata, Saved apps, and unsigned installer

See `TASKS_DETAILS_AND_UPDATES.md` for the implemented behavior, stand-in-command queue/restart evidence, persistence/layout/metadata checks, and unsigned/manual-update boundaries. Windows signing was explicitly declined. No package installation, removal, or upgrade was executed in these tests.

Final unsigned NSIS artifact: Winstore_0.1.0_x64-setup.exe (25426405 bytes), SHA-256 08F61CD7C844C6155BA54B630DF1283E2DE4ABA3A628F6595B4D9C5443BE44EB, Authenticode NotSigned. All 15 tests, strict Clippy, production frontend build, installer bundle build, and packaged Tauri runtime checks passed. Production UI is served from tauri.localhost and loads its embedded assets. The release executable is running for handoff. No installer/UAC execution or hosted feed request was performed.

## Dynamic app icon fix

Live-search and inventory-only entries previously had empty icon fields. Shared lazy lookup now reuses bundled images, extracts registered Windows icon resources, and caches matching additional PNGs. LocalSend desktop and CLI family icons are bundled. All 18 Rust tests, strict Clippy, frontend build, and unsigned installer/portable rebuild passed. In the newly extracted portable production app, all four LocalSend search-result icons and its details icon decoded successfully with zero page errors. CPU-Z extraction, Windows Terminal/qBittorrent images, disk cache, and recovery after changing a previously failed image were also checked. See APP_ICON_RESOLUTION.md. No package installations were performed.

## WinRAR banner icon correction

Replaced the 128px opaque favicon with the unchanged 256px transparent icon resource from the installed official WinRAR executable. The actual banner image decoded at 256x256; canvas checks confirmed alpha 0 at its corner and bottom edge. Visually checked the purple banner without the previous white rectangle. Frontend production build passed. This correction changes artwork and provenance only; application logic is unchanged. Capture: screenshots/winstore-winrar-transparent.png.

Unsigned installer and portable ZIP regenerated with the corrected WinRAR artwork. The portable executable was launched for handoff. The artwork was visually verified in the development WebView; the final packaged window was not recaptured.
