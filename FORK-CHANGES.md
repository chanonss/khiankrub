# Changes in this fork

This is Chanon Sarasuk's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft), kept for a
Thai-speaking audience. This file lists what differs from the original, newest first. The original's own
progress notes are in `ROADMAP.md` and `docs/`; they are not edited here. Each entry matches a commit on the
`my-changes` branch.

## 2026-10-17

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
