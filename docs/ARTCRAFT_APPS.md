# Artcraft standalone apps

Added all 13 apps listed at https://getartcraft.com/apps. The Artcraft launcher is excluded from the catalog and all installer URLs. The Artcraft collection filter shows these apps together; their normal Graphics, Documents, and Audio & Video categories also remain available. The catalog now has 144 apps.

## Distribution

| App | Winget package | Standalone version |
| --- | --- | --- |
| PhotoCraft | Publisher download | v0.16.0 |
| VectorCraft | ArtCraft.VectorCraft | v0.17.0 |
| FilmCraft | ArtCraft.FilmCraft | v0.13.0 |
| LightCraft | Publisher download | v0.14.0 |
| PdfCraft | ArtCraft.PdfCraft | v0.14.0 |
| EffectCraft | ArtCraft.EffectCraft | v0.17.0 |
| DesignCraft | Publisher download | v0.13.0 |
| FigCraft | Publisher download | v0.11.2 |
| WordCraft | Publisher download | v0.11.0 |
| SoundCraft | ArtCraft.SoundCraft | v0.13.1 |
| DeckCraft | Publisher download | v0.11.1 |
| CADCraft | ArtCraft.CadCraft | v0.13.0 |
| GridCraft | ArtCraft.GridCraft | v0.12.1 |

All 13 entries offer an individual Windows x64 MSI from their official `storytold/<app>` GitHub release. Seven also have verified Winget entries; those installer manifests were checked to ensure they install the individual app, not the launcher. Scoop remains preferred wherever the catalog has a supported Scoop mapping; no Scoop mapping was invented for these apps.

Six apps have no verified package-manager mapping in this snapshot. Their primary button says Download and opens the standalone installer URL in the default browser. The user then opens the downloaded installer. This path does not create an installation task or mark the app installed. Apps available through Winget retain the normal Install/Update behavior and also provide a standalone-download alternative.

The apps are labeled as early releases in their details view. Publisher-only downloads have no package-manager default and are not falsely advertised as managed through Scoop or Winget. Installed/update detection retains the existing package-manager inventory limitations.

## Verification and assets

Evidence is recorded in `artcraft-apps-verification.json`, collected on 2026-10-27 from the official project GitHub releases and READMEs. The GitHub releases were newer than some download versions shown on the Artcraft website, so the standalone links use the verified GitHub release assets. These links and versions are a bundled snapshot, not an automatic publisher update service.

Thirteen official icons and two real screenshots per app are bundled under `public/icons/artcraft/` and `public/previews/artcraft/`. Their source URLs are recorded in the verification JSON; logos and screenshots remain the property of the respective projects. No Artcraft source implementation was imported.

The release assets were verified from GitHub release metadata; installer binaries were not executed. Five Rust tests, strict Clippy, and the frontend production build pass. The desktop smoke pass checks all 13 cards, all preview images, package-vs-publisher states, and the exact download arguments using a test-only stub of the opener module. The real opener module is restored by reloading before handoff. Native URL opening was also observed once during initial verification. No installers were run.

Screenshots: `docs/screenshots/winstore-artcraft-collection.png` and `winstore-photocraft-details.png`.
