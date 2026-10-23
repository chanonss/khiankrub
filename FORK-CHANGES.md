# Changes in this fork

This is Chanon Sarasuk's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft), kept for a
Thai-speaking audience. This file lists what differs from the original, newest first. The original's own
progress notes are in `ROADMAP.md` and `docs/`; they are not edited here. Each entry matches a commit on the
`my-changes` branch.

## 2026-10-23

### File › Recover: readable in the light theme
- The Recover page was drawn on black, with its text forced to white. In the light theme its buttons kept their white
  fill, so Restore, Discard…, Continue without restoring and Refresh showed as blank white boxes, and the column
  headings were faint. The page now looks like the other File pages (blue sidebar, the theme's own background and text
  colours) in both themes (`recovery_page` in `crates/ui-egui/src/backstage.rs`).

### File menu: an icon for every entry
- Each entry of the File menu's blue sidebar now has a white icon before its label, as in Word: Home, New, Open,
  Info, Recover, Save, Save As, Print, Export, Close, Options, Back, Discord community and About.
- New icons drawn in code (`crates/ui-egui/src/icons.rs`, original artwork): `recover` (a clock turning back), `saveAs`
  (the disk with a pencil), `options` (a gear), `back` (an arrow in a circle) and `closeFile` (a page with a cross).
  The others reuse existing icons; Discord community uses the plain speech bubble, not Discord's logo.

### Navigation pane: Thai headings and page thumbnails
- **Thai text in the interface.** The interface fonts (Inter, Source Sans 3) have no Thai, so Thai headings in the
  Navigation pane (and other document text shown in the interface: style names, file names) drew as `?`. An installed
  Thai font is now added after them: Thonburi on macOS, Leelawadee UI on Windows, Noto Sans Thai on Linux, else any
  installed font with Thai (`thai_ui_faces` in `crates/ui-egui/src/theme.rs`). Nothing is bundled.
- **Page thumbnails.** Navigation › Pages shows a small picture of each page with its number below, the current page
  outlined in blue, as in Word (was a plain list "Page 1, Page 2 …"). Click one to go to that page. Thumbnails are
  drawn only when scrolled into view, at most two per frame, and again only when that page changes, so typing stays fast.
- New interface command `ui.navTab` (`{"tab": "headings" | "pages" | "results"}`) opens the Navigation pane on a tab.
- Tests: `panes::navigation_tests::thai_heading_text_has_glyphs`, `pages_tab_shows_a_thumbnail_per_page`.

### Repository renamed to chanonss/khiankrub
- The GitHub repository is now <https://github.com/chanonss/khiankrub> (was `chanonss/wordcraft`). GitHub redirects the
  old address, so earlier links, release downloads and clones keep working. It is still shown as a fork of
  storytold/wordcraft.
- The links in the app follow: the About screen's source link, the LaTeX export header and the MCP server's
  `websiteUrl` and instructions. `HANDOFF.md` too. Older entries below keep the old address.

### Release v0.7.0-thai.3
- First release under the name KhianKrub: [v0.7.0-thai.3](https://github.com/chanonss/wordcraft/releases/tag/v0.7.0-thai.3),
  built from commit `e893820`, file `KhianKrub-0.7.0-thai.3-macos-apple-silicon.zip` (Apple silicon, ad-hoc signed).
  SHA-256 `89f783b68582030f719ca9179498a40fbfa79f1390e12c8cdd244882c4a33d96`. v0.7.0-thai.2 stays published.

## 2026-10-22

### The name is written as one word: KhianKrub
- "Khian Krub" is now **KhianKrub** everywhere it shows: the window title, the About screen, menus, dialogs and messages,
  the Dock and Finder (`CFBundleName`, `CFBundleDisplayName`), the app file (`~/Applications/KhianKrub.app`), the Windows
  `.exe` properties, the files it writes (PDF creator and producer, DOCX `Application`, ODT/RTF/HTML generator, LaTeX
  header), the MCP server title, `NOTICE`, `README.md`, `ATTRIBUTION.md` and `HANDOFF.md`. The Thai name เขียนครับ, the
  bundle id and the executable name `KhianKrub` are unchanged. Older entries below keep the old spelling.

### Ribbon icons sized and spaced like Word's
- Small ribbon buttons are larger, as in Microsoft Word: a 20 px icon (was 17) in a 26 × 24 button (was 24 × 22). Split
  buttons and small menu buttons follow. The ribbon is 6 px taller (content 72 px, was 66) so three rows still fit.
- Columns of small buttons are now centred in each group's height instead of hugging the top: two-row groups (Font,
  Paragraph) sit in the middle with an 8 px gap between rows, three-row columns (Clipboard, Editing, Links…) fill the
  height. Large buttons moved down 3 px to stay centred. This applies to every tab, including the contextual ones.
- The floating mini toolbar over a selection uses the same larger buttons (it is a little wider).
- Code: `crates/ui-egui/src/widgets.rs` (`SMALL_W`, `SMALL_H`, `SMALL_ICON`, `ROW_GAP`, `CONTENT_H`) and `stack` in
  `crates/ui-egui/src/ribbon.rs`.

### Thai list numbers and letters
- Lists (and any other numbering that uses these formats, such as page numbers) in Word's three Thai number formats are
  now drawn in Thai instead of 1, 2, 3. Opening and saving already kept the formats; only the labels were wrong.
  - **Thai numbers** (`thaiNumbers`): Thai digits, ๑ ๒ ๓ … ๑๐ ๑๑.
  - **Thai letters** (`thaiLetters`): ก ข ค … ฮ, then กก ขข …, then กกก. Like Word, the obsolete ฃ and ฅ are skipped and
    the number of repeats starts again after 30.
  - **Thai counting** (`thaiCounting`): Thai number words, หนึ่ง สอง … สิบ สิบเอ็ด ยี่สิบ … หนึ่งร้อยเอ็ด, with เอ็ด for a
    final 1 and ศูนย์ for 0.
- Every label was checked against Microsoft Word's own labels for the same lists (480 items, plus start values up to
  99999) and is pinned by the test `doc::section::tests::thai_formats_follow_word` (`crates/doc/src/section.rs`).
- Not changed: `bahtText` still shows 1, 2, 3, as Word itself shows it in a list. The list dialogs do not offer the Thai
  formats yet; they come from opened documents.

## 2026-10-21

### New name and icon: Khian Krub (เขียนครับ)
- The app is now called **Khian Krub** (Thai: เขียนครับ, "write, please"). The name appears in the window title, the About
  screen, menus, dialogs, the Dock and Finder, the help text, and in the files it writes (PDF producer, DOCX
  `Application`, ODT/RTF/HTML generator, the LaTeX header comment) and in the MCP server's identity.
- About screen: "A free, open-source word processor for Thai and English. Based on WordCraft by the ArtCraft team.
  Fork by Chanon Sarasuk." It links to this fork's source and to the original project, and shows both copyright
  lines. The ArtCraft website and Discord links are gone from it (the File menu and Help tab still carry them).
- New icon: the Thai letter ข (kho khai, first consonant of เขียน) in gold on a deep blue rounded square. The outline
  is traced from the open-licence font Sarabun Bold (SIL OFL 1.1; licence in `assets/app-icon/OFL-Sarabun.txt`,
  credited in `NOTICE` and `ATTRIBUTION.md`) and nudged so the weight of the letter sits in the middle. All icon
  sizes and formats are regenerated by `packaging/icons.sh`.
- Mac app: `packaging/macos/local-app.sh` now builds `~/Applications/Khian Krub.app` (bundle id
  `io.github.chanonss.khiankrub`, executable `KhianKrub`), shows "เขียนครับ" on a Mac set to Thai, and also copies the
  Sarabun licence into the app. The old `~/Applications/WordCraft.app` is not touched and can be moved to the Bin.
  `package.sh` (the original's release script) only follows the new executable and icon names.
- The Windows build's file properties (product name, description, company, copyright) carry the new name too.
- The default author name for comments and tracked changes is "Khian Krub User".
- **Deliberately unchanged:** the crate names (`wordcraft-*`), the icon file names, the settings folder
  (`~/Library/Application Support/WordCraft`, so existing settings survive), the internal app id and the original's
  DMG background, so merging the original's later work stays easy. The Thai name is not in the interface
  (English only).

### Fixes from testing (tabs, number fields, space marks, alignment name)
- **Tabs and lists no longer move under the pointer.** egui gave a hovered selectable label a border that
  counted toward its size, so tab strips and lists grew by 2 px on hover and pushed their neighbours (and a
  centred dialog by 1 px). All 125 such labels now use `SteadyUi` (`steady_label`, `steady_value`,
  `steady_toggle`, `steady_scope` in `widgets.rs`). It affected 15 dialogs: Paragraph, Icons, Link, New and
  Modify Table Style, Go To, Find, Replace, Organizer, Borders, About, Manage Styles, Enclose, Define List and
  Customize Ribbon. A test sweeps every dialog, Options page, File page and ribbon tab and checks that hovering
  any control moves nothing (`hover_stability.rs`; the full sweep is `--ignored`).
- **Number fields are plain text boxes.** The fields for lengths, sizes and counts no longer change when dragged
  sideways and no longer show the resize cursor: you click, the number is selected, and you type. They keep the
  range, decimals and units (`NumField` in `numfield.rs`, replacing egui's `DragValue` at 56 places). Up and Down
  step the value; Escape goes back to the number before the edit. Sliders (the zoom slider) are unchanged.
- **Space marks centred on Thai text.** With formatting marks on, the dot for a space sat high, near the tops of the
  Thai letters. In a font that has Thai it is now centred on the middle of the letter bodies (measured on the
  shape of ก) and in the middle of the gap. Other fonts are unchanged.
- **Paragraph dialog names Thai Distribution.** Its Alignment box showed the raw text `thaidistribute` and OK without
  a change could not read it back. It now shows "Thai Distribution" and keeps it; "Distributed" is listed too.

### Other interface languages removed
- The 22 interface translations (Bulgarian, Czech, German, Spanish, Estonian, French, Croatian, Hungarian, Italian,
  Japanese, Korean, Polish, Portuguese (Brazil), Russian, Serbian in both scripts, Swedish, Turkish, Ukrainian,
  Vietnamese, Simplified and Traditional Chinese) are deleted, with the tests about them and the
  `interface-languages` build option. The interface is English only; there is no Thai interface yet. New labels are
  written in English only.
- Thai support for documents is unchanged: Thai Distribution, Thai word breaking, Thai fonts and typing.
- The Language page in Options keeps "Proofing language" and no longer has an interface language section.
- Merging the original's later changes will conflict on `crates/ui-egui/src/i18n/` (the files deleted here).

### Text boundaries: a thin grey outline around the text area
- Print Layout now draws a thin grey outline around each page's text area (inside the margins), as Apple Pages does.
  Word shows only crop marks in the corners. It is on screen only: it is never printed and never goes into a PDF.
- On by default. View › Show › **Text Boundaries** turns it off and on (command `view.textBoundaries`). It is hidden
  in Read Mode and in the Web and Outline views, like the gridlines.
- (The label was translated in the 22 interface languages at the time; those were removed later the same day.)

## 2026-10-20

### ArtCraft logos removed
- The original's brand license says a published, modified version must remove the ArtCraft logos and may only
  say in plain text that it is based on WordCraft by the ArtCraft team. This fork therefore no longer contains
  `docs/brand/` (the logos and their license file), the two logos in the README, or their `ATTRIBUTION.md` rows.
  `NOTICE` and the README say so and link to the original's brand terms. The app's About screen still has the
  original wording and links (to be reviewed).

### ICU4X credit
- The `NOTICE` entry for ICU4X now names all eleven `icu_*` crates in the build (all Unicode-3.0), not only the
  Thai word breaker's, with the full licence text. The published release v0.7.0-thai.1 still carries the earlier
  wording of the same licence text; the next release will carry this one.

### Release v0.7.0-thai.2
- [v0.7.0-thai.2](https://github.com/chanonss/wordcraft/releases/tag/v0.7.0-thai.2) replaces v0.7.0-thai.1: the same
  program, built from commit `13f20be`, with no ArtCraft logos in the repository and the licence files inside the
  app updated (complete ICU4X credit). Its zip's SHA-256 is in the release notes.

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
- Heading translated in the 22 interface languages (first draft; those languages were removed later, see above).

### View only mode (Editing menu)
- The Editing menu (top right) has a third entry, **View only**: you can read, scroll, search, copy and print,
  but nothing that edits the document runs. Comments, suggestions (Track Changes), formatting, typing, cut,
  paste, undo and redo are all refused, and their buttons grey out. Opening, saving and closing files still work.
- It is a session setting, not saved in the file. Choosing Editing or Reviewing turns it off.
- Command `review.viewOnly` (`{"value": true|false}`); the engine refuses edits in `Session::run`.
- New eye icon; the labels were translated in the 22 interface languages (since removed; first-draft translations, not reviewed
  by native speakers).

### macOS app helper
- `packaging/macos/local-app.sh` builds a double-clickable `WordCraft.app` (in `~/Applications`) from the release
  binary, with the app icon, signed for this Mac only. After a rebuild:
  `cargo build --release -p wordcraft && packaging/macos/local-app.sh`

### English-only interface by default (superseded: the other languages are now deleted)
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
