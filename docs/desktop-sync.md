# Desktop sync from the web app

Source reviewed: local `khiankrub-web` branch `web`, commit `3d7e5696` (2026-10-28).
Desktop baseline: `47674b1b`, v0.9.0. The updated source version is **v0.9.1**.
Release installers and a release tag are managed separately.

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

## Sync 2 — 2026-11-02

Source reviewed: local `khiankrub-web` branch `web` up to `07e79478` (2026-11-02), the 52 commits after `3d7e5696`.
Desktop baseline: `5ae5731f` (v0.9.1). The version number is not changed yet.

| Web change | Desktop result |
|---|---|
| WordCraft v0.12 fixes, UP12 (`5777b01d`) | Ported: stability bounds for fonts, DOCX and layout; Thai style defaults kept by style sets; PDF/shaping caches; Shift+Enter justification; bookmark/NBSP layout; Crop on the picture menu; Standard-encrypted packages; regex capture groups. See [upstream-v0.12.md](upstream-v0.12.md). |
| Font menu sized by x-height (`74e32deb`) | Ported. |
| Clippy 1.95 fixes (`5955ac7f`) | Ported, so the gates pass on the current toolchain. |
| Character spacing up/down buttons (`07e79478`) | Ported: Font dialog and Modify Style › Font. |
| Quiet Glass icons, `krub-icons` crate (`9252c9a6`) | Not yet: tied to the web start screen code; to be split out if the desktop should match the web look. |
| Web start screen, KrubOffice launcher and app switcher, new tab per document, browser drafts, fonts loaded on request, bundled TLWG/SIPA fonts, Drive worker | Website/browser features; not part of the native app. |

Validation: `cargo xtask ci` passed all six checks (format, Clippy with Rust 1.95, workspace tests: 2,862 passed, 0 failed, 3 ignored; asset attribution, dependency layers and WASM). The ported regressions include `the_spinner_buttons_step_the_number` and `kept_heading_cache_preserves_mixed_thai_english_pdf_bytes`.

## Upstream WordCraft v0.13.2 / v0.14.0 — 2026-11-02

Eleven upstream fixes ported by hand to the desktop app (UP14); three were not needed in the fork and one (copying tables, #1784) is left for its own batch. See [upstream-v0.14.md](upstream-v0.14.md). The web app does not have these yet.
