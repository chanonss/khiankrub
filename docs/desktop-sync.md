# Desktop sync from the web app

Source reviewed: local `khiankrub-web` branch `web`, commit `3d7e5696` (2026-10-28).
Desktop baseline: `47674b1b`, v0.9.0. This is a local development update; nothing is published.

| Web change | Desktop result |
|---|---|
| Empty window File-tab fix (`5f7eab0a`) | File stays clickable after leaving the Start screen; New/Open work. |
| Mac context menus and numbering (`a0c1432e`) | Right-click selects the number under the pointer and offers restart, list definition and indent actions. Native Control-click remains supported. |
| Context-menu icons (`adffb32b`) | Original ribbon icons appear in action rows and submenu headings, with disabled colours and shortcuts. |
| Problem-report form (`cc13edc5`) | Ribbon button opens the existing form with version and desktop OS/architecture, without document content. |
| Original 13-family SIPA bundle (`7498da9a`, `23a4441b`) | Desktop uses locally installed fonts. SIPA licences restrict standalone sale and require written notice before modification, so those files do not meet the repository’s open-asset requirements for bundling. |
| Browser Drive authentication, picker and save-back | As chosen by the owner, use the desktop system file picker for files available through Google Drive for desktop. The browser-only sign-in and Picker are not ported. |
| Browser recovery | Desktop retains its existing native recovery system. |
| Web loading animation, phone warning, Chrome installation, automatic site refresh, analytics and public pages | Website/browser features; not part of the native app. |
| Browser File menu with one Save page and no Close | Desktop retains native Save, Save As and Close. |
| Browser Thai default, optional system UI-font permission and hidden Zotero | Desktop retains OS-based language selection, installed UI fonts and native Zotero integration. |


Validation: `cargo xtask ci` passed all six checks (format, Clippy, workspace tests, asset attribution, dependency layers and WASM). The UI regressions include File access, Control-click, numbering menus, Thai labels and the report form.
