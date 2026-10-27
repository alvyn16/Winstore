# WinBazaar - Product Architecture & UI Design Plan

An elegant, modern software store for Windows inspired by **Bazaar (Linux / Flathub)**, integrating **Winget**, **Scoop**, and **Chocolatey** into a unified, distraction-free app discovery and management experience.

---

## 1. UI Design References & Visual Identity

The UI directly mirrors the clean, rounded GNOME / Libadwaita aesthetic found in Bazaar, seamlessly blended with Windows 11 Fluent accents.

The reference designs provided by the user have been captured and saved to:
- **Dark Mode Overview**: `assets/ui-reference/bazaar_dark_overview.png`
- **Hero Showcase & Gaming Carousel**: `assets/ui-reference/bazaar_hero_gaming.png`
- **Light Mode & Trending Grid**: `assets/ui-reference/bazaar_light_trending.png`

### Key Visual Principles
1. **Pill-Style Header Navigation**:
   - Centered segmented pill buttons: `[ ✦ Explore ]`, `[ 🗃 Installed ]`, `[ 🔄 Updates ]`
   - Left side: App branding and quick search bar.
   - Right side: Download Queue flyout button (with active count badge), Source Filter toggle, and Window controls.
2. **Prominent Hero Carousel**:
   - Large rounded cards (`rounded-2xl` / 16px radius) featuring vibrant thematic gradients (purple violet for Mobile/On-the-go, deep slate for Gaming, emerald green for Productivity).
   - High-impact headlines, descriptions, call-to-action buttons ("Browse Category"), and featured mini-app cards.
3. **App Grid & Card Anatomy**:
   - Uniform 3-to-4 column responsive grid.
   - High-contrast rounded card with subtle borders (`border-white/10` in dark mode, `border-black/5` in light mode).
   - Left: Rounded squircle app icon (48x48px).
   - Right: App Name in bold, verified badge (blue checkmark), 1-2 line description.
   - Hover effect: smooth lift (`transform: translateY(-2px)`), glow highlight, and quick-action install button.
4. **Colorful Category Tiles**:
   - Vibrant pill banners for categories (e.g., Development, Gaming, Audio & Video, Utilities, Productivity).

---

## 2. Multi-Source Architecture (Winget + Scoop + Chocolatey)

The core differentiator is giving users total transparency and choice over where an application comes from.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        WinBazaar UI (Tauri / Web)                      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ IPC Commands (Rust)
┌───────────────────────────────────▼────────────────────────────────────┐
│                    Rust Core - PackageManager Trait                    │
├──────────────────────┬─────────────────────────┬───────────────────────┤
│    WingetEngine      │       ScoopEngine       │      ChocoEngine      │
│  (CLI & REST API)    │   (Buckets / Shims)     │      (Choco CLI)      │
├──────────────────────┼─────────────────────────┼───────────────────────┤
│ • Pre-installed      │ • User-space (No UAC)   │ • Broad legacy app    │
│ • Official MS repo   │ • Dev & CLI tools       │   catalog             │
│ • Desktop apps       │ • Clean isolation       │ • Enterprise packages │
└──────────────────────┴─────────────────────────┴───────────────────────┘
```

### Source Selection Workflow
1. When searching for an app (e.g. "Git" or "VLC"):
   - The engine searches across all enabled sources concurrently.
   - If an application is found in multiple sources, WinBazaar groups them into a single primary card with a source switcher.
2. User selects their preferred source:
   - `[ Winget v2.44.0 ]` (Default if pre-installed)
   - `[ Scoop v2.44.0 (main) ]` (Ideal for zero-UAC installation)
   - `[ Chocolatey v2.44.0 ]`
3. Download & Installation Queue:
   - Non-blocking async queue manager in Rust.
   - Progress and stdout lines stream back to the UI in real time.
   - UAC elevation is handled cleanly without freezing the user interface.

---

## 3. Technology Stack

- **Desktop Shell**: **Tauri v2**
  - Ultra-lightweight memory footprint (~30-50MB vs Electron's 200MB+).
  - Native Windows webview (WebView2) with Mica/Acrylic background support.
- **Backend (Rust)**:
  - Asynchronous process orchestration via `tokio::process`.
  - Fast JSON/output parser for Winget, Scoop, and Chocolatey.
  - SQLite local database for offline catalog caching and install state history.
- **Frontend**:
  - React + TypeScript + Vite.
  - Tailwind CSS configured with Bazaar / Adwaita color palette and dark/light modes.
  - Lucide Icons for clean, consistent UI glyphs.

---

## 4. Implementation Roadmap

### Phase 1: Project Setup & Tauri v2 Scaffolding
- Initialize the Tauri v2 + React/TypeScript workspace.
- Set up Tailwind CSS with rounded corners, custom scrollbars, and Bazaar dark/light themes.
- Create window chrome with custom titlebar matching Bazaar.

### Phase 2: Engine Abstraction & Backend Commands
- Implement source availability detector (`check_sources`: winget, scoop, choco).
- Build the search aggregator: queries `winget`, `scoop`, and `choco` asynchronously and groups matches by app name/ID.
- Build the installation runner: handles streaming execution output to the download queue drawer.

### Phase 3: Core UI Pages (Bazaar Style)
- **Explore / Curated View**: Hero banner carousel, Trending apps grid, Categorized shelves (Development, Audio/Video, Gaming, Productivity).
- **Search & Filter View**: Real-time multi-source search with instant source-switching pills.
- **Installed View**: Lists all applications detected on the machine with update checks and 1-click upgrade buttons.
- **App Details Modal**: Screenshots, description, permissions/package manifest details, and source dropdown.
- **Download Queue Drawer**: Slide-out panel showing active and queued installs with progress animations.

### Phase 4: Polish & Windows-Specific Enhancements
- UAC elevation handling for installers requiring administrator privileges.
- Local catalog caching for instantaneous app browsing even when offline.
- Keyboard navigation (Search shortcut `Ctrl+F`, navigation tabs).
