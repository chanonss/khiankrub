# WordCraft v0.12 updates for KhianKrub

Status: ported to the desktop app · 2026-11-02 · Task UP12 (first done in the web app on 2026-10-29)

Source release: [WordCraft v0.12.0](https://github.com/storytold/wordcraft/releases/tag/v0.12.0), published 2026-10-28. These are selected backports to KhianKrub's existing Rust and egui 0.36 code; no application framework migration is required.

| Task | Change | Upstream sources |
|---|---|---|
| UP12.1 | Tab indents a whole selected paragraph; partial selections still receive a tab | [4291](https://github.com/storytold/wordcraft/pull/4291) |
| UP12.1 | Font preflight before WOFF/WOFF2 allocations, unsigned WOFF2 contour counts, bounded face enumeration and colour-glyph payloads | [3869](https://github.com/storytold/wordcraft/pull/3869), [4384](https://github.com/storytold/wordcraft/pull/4384) |
| UP12.1 | DOCX XML attribute cap and layout memory budgets | [3857](https://github.com/storytold/wordcraft/pull/3857), [3867](https://github.com/storytold/wordcraft/pull/3867) |
| UP12.1 | Style sets preserve imported styles and Thai document defaults | [4013](https://github.com/storytold/wordcraft/pull/4013) |
| UP12.2 | PDF and shaping performance | [4602](https://github.com/storytold/wordcraft/pull/4602) |
| UP12.2 | Justification at manual line breaks and DOCX compatibility setting | [4023](https://github.com/storytold/wordcraft/pull/4023) |
| UP12.2 | Zero-width bookmark markers do not create line breaks; nonbreaking spaces use ordinary-space widths | [4211](https://github.com/storytold/wordcraft/pull/4211), [4210](https://github.com/storytold/wordcraft/pull/4210) |
| UP12.3 | Picture context-menu Crop activates crop handles | [2440](https://github.com/storytold/wordcraft/pull/2440) |
| UP12.3 | Open Standard-encrypted Office packages | [2628](https://github.com/storytold/wordcraft/pull/2628) |
| UP12.3 | Regex Replace expands numbered and named capture groups | [4201](https://github.com/storytold/wordcraft/pull/4201) |

## Adaptation notes

- KhianKrub already draws outlines unhinted. The newer upstream hinting interpreter and its breaker/cache machinery are not introduced; the WOFF2 contour-count crash fix applies independently.
- KhianKrub's table layout does not allocate columns from `gridAfter`; the hostile `gridAfter` regression is retained without introducing newer upstream grid-gap functionality.
- Applying theme fonts and accents to a style set happens before overlaying it onto the document. This also preserves unrelated imported font choices and Thai default font, size and language.
- Picture transparency budgets retain this fork's existing crop geometry.
- Upstream's reported 24% PDF improvement was measured on Linux. Browser/Thai speedups are not assumed.

## Validation

- `cargo xtask ci` on the desktop source: see the desktop sync notes ([desktop-sync.md](desktop-sync.md)) for the result of this port.
- Added regressions cover reverse Thai paragraph selections and undo, Thai style defaults/imported font preservation through DOCX save/reopen and undo, hostile fonts/XML/layout, manual-break justification in Thai and English, NBSP and bookmark layout, Standard encryption and regex replacement groups.
- `kept_heading_cache_preserves_mixed_thai_english_pdf_bytes` confirms identical PDF bytes with cached and uncached heading lookahead. Shaping equivalence and the 64-plan bound are also tested.
- English and Thai context-menu tests activate Crop through the actual menu (the Thai menu shows **ครอบตัด**).
- No fresh Word GUI measurements or real Word-written encrypted-file corpus were used; that interoperability validation remains in `docs/gaps.md`.

## Revision history

| Date | Change |
|---|---|
| 2026-10-29 | UP12: selected changes and fork-specific adaptations recorded (web app) |
| 2026-11-02 | UP12 ported to the desktop app |
