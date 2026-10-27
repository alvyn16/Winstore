# Icons for live search and installed apps

The missing icons were caused by live Winget/Scoop results and inventory-only app entries having `icon: ""`. Only curated entries had bundled artwork. The image component also retained its failure flag when the image source changed.

`AppIcon` now lazily resolves empty or failed sources for visible cards, details, saved apps, and queue entries. Requests are shared and limited to four concurrent lookups. Changing an app or icon resets the relevant failure state; repeated broken sources do not create a retry loop. An unavailable icon has readable initials rather than a generic empty package symbol.

Resolution order:

1. Existing bundled/catalog icons, including exact source identities and canonical app names. LocalSend's official 512px PNG is bundled for desktop and CLI family entries, independently of the catalog's app count.
2. Previously cached PNGs.
3. Installed Windows apps' registered `DisplayIcon` resources. A fixed helper reads exact display-name matches and extracts the icon through Windows Shell. App names are passed through an environment variable; registry values are not run as commands. Only local-drive files are considered. Missing/stale installation resources are skipped.
4. An exact normalized match in the indexed selfh.st/icons collection. The current index contains 2,974 default PNG icon names, not 2,974 guaranteed package mappings. Unknown and ambiguous names are not assigned an arbitrary logo.

Additional PNGs are requested over validated HTTPS, limited to 1 MB and 2048px dimensions, and returned as image data URLs. No remote HTML/SVG is inserted into the app. Valid files are cached under Tauri's app-cache directory in `icons-v1`. The cache holds up to 256 PNGs, refreshes after 30 days, and retains valid stale data if refreshing fails. Invalid cache contents are ignored. Cache filenames are opaque; package names do not become filesystem paths.

Run `node scripts/update-icon-index.mjs` to refresh the list of available icon names from the upstream tree. It does not download all icons. Attribution and license copies ship in `public/licenses/`; the settings page credits the additional collection.

Some apps have no registered icon resource or matching published artwork. For example, this PC's PyCharm uninstall registration referenced an executable that no longer existed. Those cases retain initials rather than showing an unrelated app's logo.

Verification:

- Rust tests cover dynamic/catalog reuse, LocalSend and CLI resolution, normalized matching, invalid image rejection, opaque cache paths, and cached images without an online provider match.
- Actual Tauri WebView checks decoded LocalSend's four live search results, extracted CPU-Z's 256px Windows resource, decoded Windows Terminal and qBittorrent PNGs, and read back cached icons.
- A component regression check changed a failed image to LocalSend's valid image on the same component instance and verified it decoded at 512px.
- Screenshot: `screenshots/winstore-localsend-icons.png`.

The catalog remains 144 apps. The fix supplies icons to dynamic entries without inventing install-source mappings or verification claims.
