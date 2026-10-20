# Changes in this fork

This is Chanon Sarasuk's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft), kept for a
Thai-speaking audience. This file lists what differs from the original, newest first. The original's own
progress notes are in `ROADMAP.md` and `docs/`; they are not edited here. Each entry matches a commit on the
`my-changes` branch.

## 2026-10-20

### Release v0.7.0-thai.1
- First published build for Mac: [v0.7.0-thai.1](https://github.com/chanonss/wordcraft/releases/tag/v0.7.0-thai.1),
  `WordCraft-0.7.0-macos-apple-silicon.zip` (Apple silicon Macs only, macOS 11 or later; not signed by Apple, so the
  first launch needs right-click Open). Built from commit `ee1c0b0`; the notes give the zip's SHA-256.
- `my-changes` is now the repository's default branch, so the front page shows this fork's README.

### Ready to share
- `packaging/macos/local-app.sh` now puts the licences inside `WordCraft.app` (Contents/Resources): the project's
  `LICENSE-MIT` and `LICENSE-APACHE`, `NOTICE` and `ATTRIBUTION.md`, and the four font licences (`OFL-*.txt`), as
  the licences require when the app is passed on.
- The README starts with an "About this fork" note: who adapted it, what changed, a link here, the credit to the
  ArtCraft team, and that it is not affiliated with or endorsed by them.

### Thai test document
- `examples/thai-test.docx` is the project's Thai test file: Thai text in TH Sarabun New 16 pt with Thai
  Distribution on most paragraphs (the long name of Bangkok, tone marks and stacked vowels, numerals, a poem)
  and a small table. Attributed in `ATTRIBUTION.md`.
- Test `thai_test_document_keeps_thai_distribution_and_fills_its_lines` opens it, checks the alignment survives
  a save and re-read, and that every wrapped Thai Distribution line ends at the right edge.

### No Discord button in the title bar
- The "Discord" join button at the top right of the title bar is removed, and so is its Options checkbox
  ("Show the community button in the title bar"). The old saved setting is ignored. The File menu's
  "Discord community" link and the Help tab's "Community Discord" button are still there.

## 2026-10-17

### Table Properties: default cell margins
- The Table Properties dialog (Table Layout › Properties) has a **Default cell margins** section: Top, Bottom,
  Left and Right, in the document's unit (up to 2 inches). OK sends them as `cellMargins` to `table.properties`.
  A cell with margins of its own keeps them. The Table Layout › Cell Margins button still resets to the defaults.
- Heading translated in the 22 interface languages (first draft, not reviewed by native speakers).

### View only mode (Editing menu)
- The Editing menu (top right) has a third entry, **View only**: you can read, scroll, search, copy and print,
  but nothing that edits the document runs. Comments, suggestions (Track Changes), formatting, typing, cut,
  paste, undo and redo are all refused, and their buttons grey out. Opening, saving and closing files still work.
- It is a session setting, not saved in the file. Choosing Editing or Reviewing turns it off.
- Command `review.viewOnly` (`{"value": true|false}`); the engine refuses edits in `Session::run`.
- New eye icon; the labels are translated in the 22 interface languages (first-draft translations, not reviewed
  by native speakers).

### macOS app helper
- `packaging/macos/local-app.sh` builds a double-clickable `WordCraft.app` (in `~/Applications`) from the release
  binary, with the app icon, signed for this Mac only. After a rebuild:
  `cargo build --release -p wordcraft && packaging/macos/local-app.sh`

### English-only interface by default
- New Cargo feature `interface-languages` on `wordcraft-ui-egui`, **off by default**: the 22 other interface
  catalogs are not compiled in (about 2.4 MB smaller) and the Language page hides the Interface language section.
  Turn it on with `--features wordcraft-ui-egui/interface-languages`. Tests that need other languages run only then.

### File menu: Back, and a crash fix
- The "<" button at the top of the File screen is replaced by **Back** after Options.
- Fixed a macOS crash (AccessKit: "Focused ID … is not in the node list"): a click on the File screen's chrome that
  closed it refocused the page before it had been drawn that frame. The page is now refocused only if it was drawn
  in the current pass (`refocus_after_chrome_click`).

### Thai
- New paragraph alignment **Thai Distribution** (`w:jc="thaiDistribute"` in .docx): like Justify, but the extra
  space is also shared between the characters of Thai words. Command `para.thaiDistribute`, a ribbon button after
  Justify (a bold T over the Justify lines), and an entry in the Paragraph dialog's Alignment list. Saved as justify
  in ODT, RTF and HTML.
- Thai word breaking: Thai has no spaces between words, so lines now wrap between dictionary words, using
  ICU4X `icu_segmenter` (Unicode-3.0, dictionary mode; notice in `NOTICE`). Only Thai-to-Thai breaks are added.

## Building and checking
- App: `cargo build --release -p wordcraft` (about 3 to 4 minutes), then `packaging/macos/local-app.sh`.
- Full check before committing: `cargo xtask ci` (fmt, clippy, tests, assets, layers, wasm).
- Interface languages back on: add `--features wordcraft-ui-egui/interface-languages` to cargo commands.
