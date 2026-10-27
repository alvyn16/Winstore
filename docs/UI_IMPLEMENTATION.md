# UI implementation contract

The supplied dark overview is the primary reference: a 52px compact toolbar, centered navigation, graphite page, blue-to-purple feature panel with a left collection introduction and a three-column/nine-item app shelf, then four-column compact horizontal app cards. App cards prioritize the icon, name, and two-line description; source details and package actions appear in the details dialog.

Light mode follows the supplied light reference: a white toolbar, near-white canvas, six colorful featured tiles in three columns, compact four-column Trending shelf, and five category tiles. Gaming uses a wide feature panel and purple app surfaces. Its background is an original generated 3D gaming-room illustration, with the title and controls rendered as real UI.

At 900px, the shelves use three columns. Below 650px they use two columns and toolbar controls compress. The minimum supported layout width is 320px; the native window retains its existing 900px minimum. Dialogs use native HTML dialog focus containment, Escape dismissal, and focus restoration. Loading, empty, unavailable-source, operation-failure, and indeterminate-operation states are explicit. Reduced motion follows the system setting.

The UI removes unsupported verification claims and fixed demo installation/update flags. Installed status comes from Scoop, Chocolatey, and Winget inventory, with explicit manager records and source-aware actions. Package manager output remains selectable. The original app architecture remains Tauri, React, and Rust.


## App details redesign

App cards now open a wide native HTML dialog patterned on the supplied Exhibit details reference. The design has a large icon/name/publisher header, a prominent package action, functional Website and saved-app controls, six factual metadata badges, a screenshot carousel with neighboring previews and a nested enlargement dialog, an About section, and source selection. The view has dark and white themes, a theme toggle inside the view, back/close controls, focus restoration, and responsive layouts.

Visual similarity does not require fabricated download counts, license declarations, security badges, or download sizes. Unknown download size is labeled Varies. Real screenshots are currently bundled for Visual Studio Code and PowerToys; other apps show an unavailable-preview state. The developer collection now uses original generated workstation artwork and places developer apps first in its shelf.

## Compact details and window chrome

Apps without screenshots use a small inline notice rather than reserving an empty gallery. Their details dialog sizes to its content, with About and package-source selection immediately below the facts. The header spacing is tighter and its navigation remains available while scrolling.

Winstore draws a 34px themed title bar and retains native Tauri minimize, maximize/restore, close, drag, and double-click maximize actions. The bar is graphite `#222226` in dark mode and white in light mode. Native decorations are disabled so Windows accent settings cannot override this color. Each modal renders the same window controls in its top layer, keeping those controls usable while the rest of the app is inert.

The generated market-stall icon replaces the previous icon in the app toolbar, window title bar, favicon, Windows ICO, and generated platform assets. Its original Bazaar reference and generation prompt are documented in `WINSTORE_ICON_PROMPT.md`.

All dialogs share a document scroll lock. Only the first dialog hides the page scrollbar, nested dialogs keep the lock active, and the last dialog to close restores the original scroll position. A stable scrollbar gutter prevents the page from shifting horizontally; overscroll containment prevents wheel input at a dialog's edge from moving the background.
