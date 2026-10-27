# Tasks, package details, Saved apps, and unsigned installers

Implemented October 27, 2026. Windows signing was explicitly declined; no signing keys or certificates were created or used.

## Tasks

Package actions run sequentially in a backend queue. The drawer shows live stdout/stderr, queued/running/completed/failed/cancelled/timed-out/interrupted states, timestamps, attempt count, cancellation, retry, and history removal. Duplicate active actions for one app are rejected. Up to 20 active tasks and 100 history entries are retained. Output keeps the latest 32 KB per task; there is no invented percentage progress.

The queue is saved atomically to the Tauri app-data directory's `tasks.json` (`%APPDATA%\com.winstore.app\tasks.json` on this machine). An exclusive lock prevents multiple Winstore instances from writing that history at once. Each transition and the first output chunk are persisted; subsequent output is flushed at most once per second. A forced process termination can therefore lose the last second of live output. Unreadable history is copied aside before replacement; a backup failure prevents replacement.

Frontend reloads do not interrupt a backend task. Restarting Winstore marks unfinished tasks interrupted and never automatically replays an installation or removal. Retry creates a new attempt. Every package update/removal still checks fresh source inventory and scope. Inventory is refreshed after terminal task events instead of assuming installation success.

On Windows, child processes start suspended, are attached to a Job Object with kill-on-close, and then resume. Cancel/timeout terminates the managed process tree. Normal successful completion leaves apps launched by an installer running. Source checks are individually bounded to 20 seconds; cancellation during a source check waits for that bounded query to return, then prevents launching the action. UAC-brokered/external installers may escape the managed process tree, and stopping a process does not roll back changes already made.

Task timeouts are configurable from 5 to 120 minutes in settings. Retries are explicit, never automatic.

## Package details

- Scoop versions and architecture options come from the configured bucket manifest. A single selected asset's size is requested with HTTPS HEAD; multi-asset or unreported sizes remain unknown.
- Winget queries `show` for the exact ID and this PC's selected architecture. Version and release-note URL are parsed only when explicitly reported. Download size uses the selected installer's HTTPS Content-Length. Unrecognized/localized fields remain unknown rather than guessed.
- Chocolatey supplies its live repository version and a link to package release information. Size and architecture remain unreported when absent.
- Artcraft publisher releases retain their verified x64 asset version/size and fetch release notes from the official GitHub release.

Requests use normal TLS validation, bounded response sizes, and timeouts. Changing the selected source discards stale detail responses. Installed and available versions are shown separately. A source can report a newer local version than the bundled catalog snapshot; these values are not overwritten with demo data.

## Saved apps and preferences

Saved apps have their own navigation page and support catalog or live-search entries. Metadata is retained for saved entries outside the bundled catalog. Existing saved IDs are preserved. Recommended/name ascending/name descending/publisher/updates-first sorting is available, and source preferences, sorting, text size (100–130%), timeout, and update-check preference persist locally. Unavailable sources keep their preference but cannot run actions. Dialogs continue to lock background scrolling.

## Unsigned installer and update checks

The Windows release builds a per-user NSIS installer with no Authenticode signing and no Tauri updater signatures. It does not silently download or execute application updates. A configured build can check an HTTPS feed at startup or from settings, display a newer version and release notes, and open its unsigned installer in the browser for manual installation.

There is no update-hosting URL or public release configured in this checkout. The current build clearly reports that condition and disables the check button. No fictitious release URL is shipped. Publishing/hosting an update feed remains a separate deployment step.

Build an unsigned installer with `npm run tauri build -- --bundles nsis`, or use PowerShell 7:

```powershell
./scripts/build-windows.ps1
# When actual hosting is available:
./scripts/build-windows.ps1 -UpdateFeedUrl https://YOUR-HOST/latest.json -InstallerBaseUrl https://YOUR-HOST/releases/
```

The helper embeds `WINSTORE_UPDATE_FEED_URL` for that build and optionally creates `latest.json` beside the installer. It prints the installer SHA-256 and does not upload or publish anything. Host the actual installer and feed at the chosen URLs. Feed format:

```json
{
  "version": "0.2.0",
  "notes": "Describe this release.",
  "platforms": {
    "windows-x86_64": { "url": "https://YOUR-HOST/releases/Winstore_0.2.0_x64-setup.exe" }
  }
}
```

The checker accepts strictly newer semantic versions and HTTPS URLs without embedded credentials. It does not treat an unsigned feed as proof of authenticity or automatically install its contents.

## Verification evidence

- Backend tests cover streaming before process exit, cancellation and timeout, interrupted-task recovery, bounded UTF-8 output, HTTPS/version validation, source metadata, and the existing inventory/catalog checks.
- Actual backend queue tests substituted a harmless `scoop.cmd` in the test process's PATH. The stand-in printed markers, waited, and returned failure. These tests exercised FIFO, duplicate protection, queued/running cancellation, retry, live drawer output, failure output, page-reload survival, and full app-process restart recovery. No software was installed, removed, or upgraded. Stand-in processes and task records were removed afterward; normal package-manager lookup was restored.
- Running WebView checks covered Saved apps, sorting, source/text-size/timeout persistence, 130% text, widths 320/768/1200, missing update-feed handling, and metadata from Scoop, Winget, Chocolatey, and an official Artcraft release. No page errors occurred in that pass.
- Captures: `winstore-saved-apps.png`, `winstore-persistent-settings.png`, and `winstore-live-task-queue.png`. The queue capture is explicitly a stand-in-command verification, not evidence of an actual installer run.

Installer execution/UAC and a hosted update check remain unverified. The installer is built and checked as a local artifact; it is not published.
