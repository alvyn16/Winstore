# Bundled icon sources

Logos identify the respective software packages and remain the property of their projects. They are stored locally to avoid broken remote images at runtime.

| Asset | Source |
| --- | --- |
| app-0.ico | https://code.visualstudio.com/assets/favicon.ico |
| app-1.ico | https://git-scm.com/favicon.ico |
| app-2.svg | https://www.rust-lang.org/static/images/favicon.svg |
| app-3.png | https://nodejs.org/static/images/favicons/favicon.png |
| app-4.ico | https://store.steampowered.com/favicon.ico |
| app-5.ico | https://heroicgameslauncher.com/favicon.ico |
| app-6.ico | https://prismlauncher.org/favicon.ico |
| app-10.ico | https://open.spotifycdn.com/cdn/images/favicon.0f31d2ea.ico |
| app-11.ico | https://discord.com/assets/favicon.ico |
| app-12.ico | https://www.mozilla.org/media/img/favicons/firefox/browser/favicon.f093404c0135.ico |
| vlc.png | https://raw.githubusercontent.com/videolan/vlc/master/share/icons/128x128/vlc.png |
| dolphin.png | https://raw.githubusercontent.com/dolphin-emu/dolphin/master/Data/dolphin-emu.png |
| obs.png | https://raw.githubusercontent.com/obsproject/obs-studio/master/frontend/cmake/linux/icons/obs-logo-256.png |
| 7zip.ico | https://raw.githubusercontent.com/ip7z/7zip/main/C/Util/7zipInstall/7zip.ico |
| powertoys.png | https://raw.githubusercontent.com/microsoft/PowerToys/main/doc/images/icons/PowerToys%20icon/PNG/PowerToysAppList.targetsize-256.png |

`public/winstore.svg` is an original vector mark. The gaming scene in `public/artwork/gaming-room.png` was generated with the built-in ImageGen tool. Its prompt is saved in `docs/GAMING_ARTWORK_PROMPT.md`. The saved Bazaar screenshots are supplied design references.


## App previews and developer banner

The developer banner `public/artwork/developer-workspace.png` was generated with the built-in ImageGen tool. The exact prompt is in `docs/DEVELOPER_ARTWORK_PROMPT.md`.

Locally bundled screenshots come from official project documentation:

- `public/previews/vscode-interface.png`: https://code.visualstudio.com/assets/docs/getstarted/userinterface/hero.png
- `public/previews/vscode-debugging.png`: https://code.visualstudio.com/assets/docs/editor/debugging/debugging_hero.png
- `public/previews/powertoys-fancyzones.png`: https://raw.githubusercontent.com/microsoft/PowerToys/main/doc/images/overview/FancyZones_large.png
- `public/previews/powertoys-colorpicker.png`: https://raw.githubusercontent.com/microsoft/PowerToys/main/doc/images/overview/ColorPicker_large.png
- `public/previews/powertoys-commandpalette.png`: https://raw.githubusercontent.com/microsoft/PowerToys/main/doc/images/overview/CmdPal_large.png

Screenshots remain the property of their respective projects. Other apps show an explicit missing-preview state rather than generated software screenshots.

## Winstore app icon

`public/winstore.png` is an ImageGen-created market-stall icon inspired by Bazaar's original striped storefront, with four software tiles in place of the fruit. Reference: https://github.com/bazaar-org/bazaar/blob/main/data/icons/hicolor/scalable/apps/io.github.kolunmi.Bazaar.svg. The reference is retained only under `assets/ui-reference/`. Prompt details are in `WINSTORE_ICON_PROMPT.md`; generated platform icon files are under `src-tauri/icons/`. The old `public/winstore.svg` is a superseded source asset and is no longer referenced by the interface or favicon.


## Artcraft apps

The 13 Artcraft app icons and 26 screenshots are bundled from the official `storytold` app repositories. See `artcraft-apps-verification.json` for each exact source URL and `ARTCRAFT_APPS.md` for distribution details. No launcher branding or installer is used as an app entry.
