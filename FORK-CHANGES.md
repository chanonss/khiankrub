# Changes in this fork

This is Chanon Sarasuk's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft), kept for a
Thai-speaking audience. This file lists what differs from the original, newest first. The original's own
progress notes are in `ROADMAP.md` and `docs/`; they are not edited here. Each entry matches a commit on the
`my-changes` branch.

## 2026-10-25

### Modify Style: icons and more settings
- Each page in Modify Style's list has its icon (drawn in code, from the ribbon's set).
- New pages, after LibreOffice's style dialog (the idea only), limited to what a Word document keeps:
  - **Font Effects:** underline style (single, words only, double, thick, dotted, dashed, dot dash, wave) and
    colour; strikethrough, double strikethrough, hidden, superscript, subscript, small caps, all caps, outline,
    shadow, emboss, engrave (the effects that exclude each other untick each other). The Font page keeps the fonts,
    sizes, colour.
  - **Position:** raise or lower (pt), scale width (%), character spacing, kerning from a font size up.
  - **Tabs:** the style's tab stops with position, alignment (left, center, right, decimal, bar) and leader; New,
    Delete, Delete All. Deleting a stop the style inherits writes a clear for it.
- **Text Flow** adds Suppress line numbers and Don't hyphenate; **Borders & Shading** adds Distance from text.
- A character style has General, Font, Font Effects and Position.
- Colibre (LibreOffice's icon set) was considered and not used: its README says it follows "the monoline style set
  by Redmond", i.e. Microsoft Office's look, which this project's asset rule forbids; and the linked copy has no
  licence file (in LibreOffice it falls under MPL 2.0).
- Test: `effects_position_tabs_and_more_reach_the_style`.

### Modify Style with pages (round 2)
- Three more pages in Modify Style:
  - **Text Flow:** Widow/Orphan control, Keep with next, Keep lines together, Page break before.
  - **Borders & Shading:** the sides with a line (Top, Bottom, Left, Right, Between paragraphs; None and Box set
    them at once), the line's style (Single, Double, Dotted, Dashed, Thick, Dot dash), width and colour; the
    shading fill, or No Color. The sides left off are set off, so a base style's borders don't show through.
  - **Numbering:** number the style's paragraphs with one of the document's lists, a new multilevel list
    (1. 1.1. 1.1.1., I. A. 1. a., ๑. ๑.๑. ๑.๑.๑., ๑. ก. (๑) (ก)) or none, at a chosen level. The level is linked to
    the style, as Link level to style does from the list's side.
- `styles.modify` takes `list` (`{"num"}`, `{"kind"}` or null), `listLevel` and `clear` (`shading`, `borders`).
- The preview's sample is one line, so a bottom border shows under it.
- Test: `flow_borders_and_numbering_reach_the_style`.

### Modify Style with pages (round 1)
- **Modify Style** is a larger window with a list of pages on the left and a live preview below
  (`crates/ui-egui/src/dialogs_style.rs`), after the layout of LibreOffice's style dialog (the idea only; no code):
  - **General:** name, Based on, Style for following paragraph, style type.
  - **Font:** **Latin text** and **Thai text (complex script)** side by side, each with its font, size, Bold and
    Italic, and "Thai text like Latin text" to copy one to the other; Underline, Strikethrough, Small caps,
    Superscript, Subscript, All caps, font colour and character spacing.
  - **Indents & Spacing:** left, right, Special (first line / hanging) and By; space before and after; line
    spacing (Multiple, At least, Exactly); "Don't add space between paragraphs of the same style".
  - **Alignment:** Left, Center, Right, Justify, Distributed, Thai Distribution; left-to-right or right-to-left.
  - A character style has only General and Font.
- The preview draws sample Latin and Thai text with the edits applied, before OK.
- OK sends only what changed, so what the style inherits from its base stays inherited. A changed Latin font, size,
  bold or italic is sent with the Thai one beside it, so the box shown is what the style gets.
- `styles.modify` takes `basedOn` (a style or null) and refuses a base that is based on the style itself.
- Tests: `the_font_page_sets_thai_text_apart`, `indents_alignment_and_base_reach_the_style`;
  `modify_style_font_box_finds_the_family` opens the Font page.

### Modify Style sizes reach Thai text
- **Modify Style** (and `styles.modify`) now sets the style's size, Bold and Italic for complex-script (Thai) text
  too, as the Font size box already did; a changed font goes for Thai text too. Before, Heading 1 set to 18 pt kept
  its Thai size (24 pt in a new document), so a Thai heading showed bigger than the 18 in the size box; it showed once
  a linked list level cleared the text's direct size. A font sent unchanged (the dialog sends it every time) leaves a
  separate Thai font alone, so a Word document with Latin Calibri and Thai Angsana New headings keeps Angsana.
- An existing document with the mismatch is fixed by opening Modify Style on the heading and clicking OK.
- Test: `modify_style_size_reaches_thai_text`, with Chanon's steps.

### Linked heading style shows on the text; Backspace and Delete take a list number off
- **Link level to style, again:** the paragraph given the linked style (Heading 1) now also loses its direct font,
  size and colour, as when a style is applied from the Styles gallery. Thai documents often carry their font and size
  directly on the text (TH SarabunPSK 16), which kept the heading's look from showing although the style was set.
  Test: `a_linked_style_shows_over_direct_font`.
- **Backspace at the start of a heading numbered by its style** (a list level linked to it) takes the number off and
  makes the paragraph Normal in one press. Before, the number was missed (it isn't the paragraph's own) and Backspace
  joined the heading to the paragraph above. Word keeps the heading style here; KhianKrub returns to Normal as its
  users expect. Backspace at the start of an ordinary list item is unchanged (number off, then the indent).
- **Delete in an empty numbered paragraph** takes the number off instead of pulling the next paragraph up into the
  list. Test: `backspace_and_delete_take_a_list_number_off`. (Typing changes are recorded here; the original's
  `docs/typing-parity.md` is not edited in this fork.)

### Define New Multilevel List: linked styles, the number's font, My Lists
- **Link level to style** now works as in Word: OK gives the selected paragraph the style its level is linked to (a
  level linked to Heading 1 makes the paragraph a Heading 1, so its text takes the heading's look), and its number
  comes from that style. A paragraph already in a linked style keeps its style and is numbered by it. Before, the
  paragraph got the number but stayed Normal (List Paragraph). Test: `a_level_linked_to_a_style_gives_the_paragraph_that_style`.
- **Font** now changes the number. The font, Bold and Italic are set for complex-script text too (`font_cs`,
  `bold_cs`, `italic_cs`): a Thai number, or any number in a Thai paragraph, is drawn in the complex-script font, so
  a font set for Latin text only showed no change. Unticked Bold and Italic follow the paragraph instead of forcing
  them off. The font is picked from the installed fonts, as in the ribbon's font box, with Clear to go back to the
  paragraph's. Test: `a_level_font_covers_thai_numbers`.
- **My Lists:** the dialog has a List name (kept with the list in the document) and "Save in My Lists, for every
  document". Saved lists are kept with the interface settings (`ui.json`, `savedLists`; up to 50, a list saved again
  under the same name replaces it) and listed in **Home › Multilevel List › My Lists** with how they number
  (`Thesis   ๑. ๑.๑. i.`); picking one numbers the selected paragraphs with it. **Remove from My Lists** takes one
  out (lists already in documents stay). Test: `a_saved_list_is_kept_and_numbers_another_document`.

### Table borders on the selected cells, as in Word
- In a table, the border choices (Home › Borders and Table Design › Borders) now act on the selected cells, or the
  caret's cell, instead of always on the whole table. Bottom Border with a row selected is the line under that row;
  Top, Left and Right the selection's edge on that side; Outside frames the selection; Inside, Inside Horizontal and
  Inside Vertical go between its cells; All and No Border cover just those cells. Choosing a side again takes it off.
  The cell on the other side of an edge gets the same border, so a line taken off doesn't still show from there.
  With the whole table selected, the choices set the table's own borders as before (`cell_borders` in
  `crates/engine/src/cmd/table.rs`).
- The Home tab's Borders button (its main half) also sets the selected cells' bottom border in a table, instead of
  the cell paragraph's.
- Test: `table_borders_follow_the_selected_cells`; `table_border_presets_mask_the_sides_they_clear` and
  `table_diagonal_and_inside_borders` now select the whole table where they mean it.

### Borders menu like Word's, with diagonal borders
- **Home › Paragraph › Borders** and **Table Design › Borders** are now the same menu, laid out as Word's, with a
  picture before every item: Bottom, Top, Left and Right Border; No, All, Outside and Inside Borders; Inside
  Horizontal, Inside Vertical, Diagonal Down and Diagonal Up Border; Horizontal Line; Draw Table, View Gridlines (with
  its check mark) and Borders and Shading…. New in the menu: Inside Horizontal, Inside Vertical, the two diagonals,
  Draw Table and View Gridlines.
- In a table the Home menu now sets the table's borders, as Table Design's does (before, it gave the paragraph in
  the cell a border). Outside a table it sets the paragraph's; Inside Vertical and the diagonals are then greyed out.
- **Diagonal borders** (`table.borders` `diagDown` / `diagUp`): a line across each selected cell, corner to corner;
  choosing it again takes it off, and No Border clears it. A merged cell's diagonal crosses all its rows. Kept in
  .docx as `w:tl2br` / `w:tr2bl` (ECMA-376 §17.4.79–80) and now also read from Word files, which lost them before
  (`CellProps::diag_down` / `diag_up`). A slanted line cut at a page break keeps its slope.
- Test: `table_diagonal_and_inside_borders`; `layout_menu_pictures_all_draw` covers the new pictures.
- The dotted lines in the border pictures, and the dots of the ribbon's Borders icon, are darker and denser: at
  normal (1×) screen size they had almost vanished. Checked at 1× and 2×.

### Pictures in five more menus
- Each item of these menus has a small picture before its name, like the Layout menus since v0.7.1 (`pic_item` and
  `layout_picture` in `crates/ui-egui/src/ribbon.rs`; all drawn in code):
  - **Table Design › Borders:** a 2 × 2 table with the borders the choice sets drawn solid, the others dotted.
  - **Table Layout › Select:** a 3 × 3 table with the cell, column, row or whole table shaded.
  - **Table Layout › Delete:** the same, crossed out.
  - **Table Layout › AutoFit:** the table drawn in to its text, stretched to the page's edges, or with its column
    widths fixed on a ruler.
  - **Layout › Margins:** the page with its text area outlined as wide as the preset leaves it, two facing pages for
    Mirrored; the current preset stays highlighted.
- Checked in the light and dark themes. Test: `layout_menu_pictures_all_draw` now covers the new pictures.

## 2026-10-24

### Universal Mac build (Apple silicon and Intel)
- From the next release, the Mac download is one app for Apple silicon and Intel Macs:
  `KhianKrub-<version>-macos-universal.zip` (about 48 MB, was 23 MB). Both still need macOS 11 or later.
- `packaging/macos/local-app.sh --universal` joins the `aarch64-apple-darwin` and `x86_64-apple-darwin` builds with
  `lipo`, as the original's `package.sh` does. The release workflow's macOS job builds both, checks that the app has
  both halves, and runs each (the Intel one through Rosetta when the runner has it).
- Tried on this Mac: both halves start (`arch -arm64` and `arch -x86_64`), each built for macOS 11.0, and the
  signature check passes. Not yet tried on a real Intel Mac.
- README: "Download for Mac", with the new zip name and what each version needs.

### Release v0.7.3
- [v0.7.3](https://github.com/chanonss/khiankrub/releases/tag/v0.7.3), built from commit `0357bfe`, the first
  release whose files were all built by GitHub Actions:
  `KhianKrub-0.7.3-macos-apple-silicon.zip` (SHA-256 `e553ae6fc9b49edced0706816f640ce2b4ceb3ddcd673d3c02057ab862918206`),
  `khiankrub-0.7.3-windows-x64.msi` (`ee8095b3e396c74f071d115be48a509a8fb3fb5f1f9cd9622898adf80b5eb743`) and
  `khiankrub-0.7.3-windows-x64-portable.zip` (`677c3b472a5a99a6ac1672aa2c837fd9f1bc40d80479f1d67aab4d8f409a82dd`).
  The published Mac zip was downloaded again and checked (signature, version 0.7.3, commit `0357bfe`).

### Mac release zip built on GitHub Actions
- The release workflow (renamed `.github/workflows/release-files.yml`) also builds the Mac zip on GitHub's
  Apple silicon runner, the same way as by hand (`local-app.sh`, `ditto`, a strict `codesign` check of the unzipped
  copy), and attaches it next to the Windows files. One job makes the draft release first, so the two builds don't
  each create one. Started by hand, it takes a tag and `both`, `windows` or `mac`.
- Why: uploading the Mac zip in the browser failed twice (it stalls partway even on a fast connection).

### Version 0.7.3
- The app's version is 0.7.3 (was 0.7.2). It brings the fixes from testing v0.7.2 (a window for each document, File ›
  Recover only after a crash, TH Sarabun New 16 for new documents, Thai caption labels, Header and Footer in the
  Styles gallery, Thai Distribution on the floating format bar), for Mac and Windows together. It is the first
  release whose Windows files are built by GitHub Actions.

### Windows builds on GitHub Actions
- `.github/workflows/windows-release.yml`: pushing a `v*` tag builds the Windows installer and portable zip on
  GitHub's Windows runner (WiX 5.0.2, `packaging/windows/package.ps1 -Arch x64`), lists their SHA-256 in the run's
  summary and attaches them to the tag's release, as a draft release when the tag has none. It can also be started
  by hand from the Actions tab with a tag name. The tag must match the version in `Cargo.toml` (`v0.7.3`, or
  `v0.7.3-test1` for a trial). The files are unsigned, like the PC-built ones. From the next release on, the PC is
  only for test installs.
- Removed the original's `release.yml`, `freebsd.yml`, `windows-arm64.yml` and `packaging-lint.yml`: they call
  storytold's shared workflows and need its signing secrets, so they can't run here.
- HANDOFF.md: the code lives in a git clone at `~/dev/khiankrub` on each machine, synced through GitHub, not in
  Google Drive.

### Fixes from testing v0.7.2
- **A new window for each document.** File › Open (and recent files, Finder, a file dropped on the Dock icon) and
  File › New open in a window of their own once the current window holds a document, as in Word. The untouched blank
  document a window starts with is still replaced, and a template still opens in the current window. Each window is
  its own copy of the app (`new_window` in `apps/wordcraft/src/main.rs`), so on the Mac each has its own Dock icon. A
  new blank window starts with `--new`. Test: `open_and_new_use_a_new_window_once_this_one_has_a_document`.
- **File › Recover no longer comes back after a normal close.** A document's recovery copy is now removed when the
  user chooses Don't Save and when the app closes normally (`Session::discard_recovery_copy`), so only a crash leaves
  one. And a file opened at launch (Finder, the command line) stays in front: the Recover page shows by itself only
  on an empty start; otherwise the status bar says copies can be recovered from File › Recover. Copies already left
  by earlier versions show once more; Discard removes them. Test: `letting_a_document_go_removes_only_its_copy`.
- **New documents use TH Sarabun New 16.** The desktop app's new blank documents (and opened plain text) use
  TH Sarabun New for Thai and English text, 16 pt body text, with every built-in style's size stepped up to match
  (Heading 1 24 pt, Heading 2 20 pt, Heading 3 18 pt, Title 37 pt, Caption 12 pt). Opened documents and templates keep
  their own fonts; headless use (`wordcraft-cli`, MCP) keeps the original's defaults
  (`Session::set_thai_defaults`, `StyleSheet::apply_thai_defaults`). When the font isn't installed, a notice at
  launch offers its download page, <https://www.f0nt.com/release/th-sarabun-new/>, with "Don't show this again". The
  font is not bundled. Test: `new_documents_use_the_thai_fonts`.
- **Thai caption labels.** Insert Caption offers ภาพที่ and ตารางที่ and uses them by default (ตารางที่ in a table,
  ภาพที่ elsewhere); Figure, Table and Equation are still there. Cross-reference lists Thai captions under Figure and
  Table, and Insert Table of Figures uses ภาพที่ when the document has such captions. Test:
  `thai_captions_list_and_cross_reference`.
- **Header and Footer in the Styles gallery**, for new documents and for opened Word files (where Word hides them or
  leaves them out); a file's own style named Footer is left alone. Tests: `header_and_footer_are_in_the_gallery`,
  `styles_with_qformat_keep_their_gallery`.
- **Thai Distribution on the floating format bar**, after Justify. Test: `the_bar_sets_thai_distribution`.

### Release v0.7.2
- [v0.7.2](https://github.com/chanonss/khiankrub/releases/tag/v0.7.2), built from commit `9f474710`. Windows files
  `khiankrub-0.7.2-windows-x64.msi` (SHA-256 `bccb048c09faa119dff61489ba3adb5498d514d457ce9b3cd37882ed8ec06a7c`) and
  `khiankrub-0.7.2-windows-x64-portable.zip` (`c359f96a99cf886c49f095574781f4ca3c9a7accb1744999007bcd5a48a3046d`);
  Mac file `KhianKrub-0.7.2-macos-apple-silicon.zip`, SHA-256
  `d402084070dc44454d7da8288dc52af863e8e0bf9b6d484f3f6208cd0bfe1f66`, built on the Mac after `cargo xtask ci` passed
  there. On the Mac the bold Thai interface test (`thai_heading_text_has_glyphs`) passes; it fails only on Windows.

### Version 0.7.2
- The app's version is 0.7.2 (was 0.7.1). It brings the Thai lists from the menus and is the first version released
  for Windows and Mac together; the Windows build comes first, the Mac build follows.

### Thai lists from the menus
- Thai numbering can now be made in KhianKrub itself, not only shown from Word files (see "Thai list numbers and
  letters", 2026-10-22):
  - **Home › Numbering** has ๑. ๒. ๓. (`thaiNumbers`), ก. ข. ค. (`thaiLetters`) and หนึ่ง. สอง. สาม.
    (`thaiCounting`). Deeper levels go on in Thai: ๑. then ก., ก. then ๑., หนึ่ง. then ๑. then ก.
  - **Home › Multilevel List** has ๑. ๑.๑. ๑.๑.๑., every level in Thai digits, and ๑. ก. (๑) (ก). The first is
    not Word's legal numbering (`isLgl`), which would show the levels it repeats in Arabic digits.
  - **Define New Multilevel List › Number style for this level** offers ๑, ๒, ๓, … / ก, ข, ค, … / หนึ่ง, สอง, สาม, ….
- Choosing another numbering style on a list now switches it. Before, ๑. (or A., i.) on a "1." list removed the
  numbering, because every one of them reads "%1."; a style now matches a list only when its number format matches
  too.
- Saved files keep the Thai formats, so Word shows the same labels. Tests: `thai_list_kinds_number_in_thai`
  (`crates/doc/src/numbering.rs`) and `thai_numbering_and_multilevel_lists_apply_switch_and_save_to_word`
  (`crates/engine/src/tests.rs`).
- Not changed: Insert › Page Number › Format Page Numbers does not offer the Thai formats yet.

### README: Windows download
- The README offers KhianKrub for Mac and Windows: a "Download for Windows (64-bit)" link at the top, a Windows part
  in "Download and install" (the MSI, More info › Run anyway, the portable zip, in English and Thai) and how to build
  the installer. The Windows files are added to the v0.7.1 release; the app code is the same as the Mac build's.

### Windows installer as KhianKrub
- `packaging/windows/package.ps1` now makes `khiankrub-<version>-windows-<arch>.msi` and
  `khiankrub-<version>-windows-<arch>-portable.zip`. Cargo still builds `wordcraft.exe`; it is packaged and installed
  as `KhianKrub.exe`.
- The MSI (`packaging/windows/wordcraft.wxs`) installs **KhianKrub** by Chanon Sarasuk into `Program Files\KhianKrub`
  with a KhianKrub Start Menu entry, and links to the GitHub project in Settings › Apps. Its UpgradeCode and component
  GUIDs are new, so it never upgrades or removes an installed WordCraft. The file types are registered as
  `KhianKrub.*` ProgIds and under `RegisteredApplications` as `KhianKrub`, the name File › Options › General already
  opens in Settings › Default apps. The licence page says KhianKrub is a fork of WordCraft, not affiliated with the
  ArtCraft team.
- Built and installed on Windows 10 x64 with WiX v5.0.2; the MSI and the app inside are unsigned.

## 2026-10-23

### Release v0.7.1
- [v0.7.1](https://github.com/chanonss/khiankrub/releases/tag/v0.7.1), built from commit `06b64e40`, file
  `KhianKrub-0.7.1-macos-apple-silicon.zip`, SHA-256 `c272bd19489aae404052d509c6ad8c7cca73ba12a2cfea0cfa46b02368331661`.
  Brings the menu bar, Page Number Position, the page-break fixes, the hanging indent and the menu pictures. The earlier
  releases stay published.

### Version 0.7.1
- The app's version is 0.7.1 (was 0.7.0, the original's). Releases from now on use plain version numbers (v0.7.1); the
  earlier ones were named v0.7.0-thai.1 to v0.7.0-thai.4 ("the original's 0.7.0 plus Thai build N"), from before the
  app had its own name.

### README: the newest additions
- A "Newest additions" section with screenshots of the macOS menu bar, Page Number Position (the dialog and the result
  with its distances drawn in), page breaks, the hanging indent, and the menus with pictures (credited in
  `ATTRIBUTION.md`). The menu bar picture shows only the app's own menu bar and menu window.

### macOS menu bar
- KhianKrub has a real macOS menu bar: the app menu (About, Settings… ⌘,, Services, Hide, Quit), **File** (New, Open,
  Close, Save, Save As, Export as PDF, Page Setup, Print), **Edit** (Undo, Redo, Cut, Copy, Paste, Paste Special, Paste
  and Match Formatting, Select All, Find, Replace, Go To), **Insert** (breaks, table, picture, text box, link,
  bookmark, cross-reference, comment, footnote, header, footer, Page Number Position, date and time, symbol,
  equation), **Format** (Font, Paragraph, Bullets and Numbering, Borders and Shading, Columns, Styles, bold, italic,
  underline, font size, alignment including Thai Distribution, Clear Formatting), **View** (the views, Ruler,
  Navigation Pane, Formatting Marks, zoom, Full Screen), **Window** (Minimize, Zoom, Bring All to Front, the open
  windows) and **Help** (Commands, the project on GitHub, Report a Problem). The ribbon and the File panel are unchanged.
- Each item runs the app's existing command. An item with a shortcut shows it and, when chosen, gives the app that key
  press, so it does exactly what the key does wherever the focus is (⌘C in a dialog's text box copies that text). Quit
  closes the window the usual way, so unsaved changes are asked about first.
- Built with muda (MIT OR Apache-2.0), macOS only (`apps/wordcraft/src/mac_menu.rs`). Test:
  `menu_shortcuts_parse_and_match_the_command_keys`.

### Keyboard page break (⌘Enter) and Layout › Breaks › Page
- These used a separate command (`text.pageBreak`) that still left the cursor beside the break on the old page. They
  now share the fixed Insert › Page Break behaviour: the cursor starts the next page.

### Page break puts the cursor on the next page; tighter table grid; Page Number menu pictures
- **Page break fixed.** Insert › Page Break (and Ctrl+Enter) only put the break character in: the cursor stayed beside it
  on the old page, so what was typed next showed there, under the break. Now it works as Word's: the break ends its
  paragraph and a new paragraph starts the next page with the cursor in it; text after the cursor moves on with it.
  In a table cell or a text box the break goes in without a new paragraph (`insert.pageBreak`). Test:
  `page_break_puts_the_caret_on_the_next_page`.
- With formatting marks on, a paragraph ending in a page break draws its ¶ at the end of the "Page Break" line, as Word
  does, instead of alone on a line below it (where it looked like the next line had not moved to the next page). Test:
  `a_page_break_ending_its_paragraph_draws_its_mark_on_the_break_line`. Layout is unchanged: as measured in Word, such a
  paragraph's mark stays on the page before and the next paragraph starts the new page.
- **Insert › Table grid** squares sit close together as in Word (the grid had widened each column to egui's minimum
  width). Insert Table…, Draw Table and Convert Text to Table… show icons and line up (Draw Table was indented).
- **Insert › Page Number** shows a picture for each choice: number at the top, at the bottom, "1/2", within a line of
  text, number formats, and the position from the edges.

### Page Number Position: the page number at set distances from the paper's edges
- New **Insert › Page Number › Page Number Position…** dialog for layouts such as a thesis's "the page number 1 inch
  from the top edge and 1 inch from the right edge". It offers: from the **top or bottom edge**, a distance measured to
  the number's **baseline**; **left, centre or right**, a distance measured to the number's near edge (its right end
  from the right edge, its start from the left edge); and **Don't show on the first page of the section** (Different
  First Page). A small page preview shows where the number goes. Opening it again shows the current placement.
- The number goes in a text frame anchored to the page (`w:framePr`) in the section's header (top) or footer (bottom),
  so Word opens the file with the number in the same place; the header or footer's other content stays, and a number
  placed before is moved rather than added again. The frame's top is found by laying the page out and correcting it
  until the baseline lands within 0.05 pt of the distance asked for.
- Command `layout.pageNumberPosition` (`vertical`, `fromEdge`, `horizontal`, `fromSide`, `firstPage`) for scripts and
  agents (`crates/engine/src/cmd/insert.rs`); dialog in `crates/ui-egui/src/dialogs_layout.rs`. Tests:
  `page_number_position_measures_from_the_paper_edges` (top-right, bottom-left, centred, first page, saved and opened
  again) and `page_number_position_dialog_reads_and_sets_the_placement`.

### Page and column breaks named, as in Word
- With formatting marks on (¶), a page break shows as a dotted line to the margin with **Page Break** in the middle,
  and a column break with **Column Break**, as in Word and like the section breaks (which keep their double line).
  Before, a page break showed only a small ⤓ arrow and a plain dotted line. On screen only: never printed or exported
  (`break_mark` in `crates/layout/src/display.rs`). Test: `page_breaks_show_their_name_with_formatting_marks`.

### Layout tab menus: a picture for each choice
- The Layout tab's Text Direction, Orientation, Columns, Breaks and Line Numbers menus show a small picture before each
  choice, as in Word: the page's lines in each direction, a tall and a wide page, one to three columns and the
  left/right layouts, each kind of page, column, text wrapping and section break, and lines with numbers continuous
  or restarting. The current Text Direction, Orientation and Columns setting stays highlighted. Drawn in code in the
  theme's colours (`pic_item` and `layout_picture` in `crates/ui-egui/src/ribbon.rs`). Test:
  `layout_menu_pictures_all_draw`.

### Paragraph dialog: Special indent (First line, Hanging) as in Word
- The Paragraph dialog's Indentation shows **Special: (none) / First line / Hanging** and **By**, as in Word, instead
  of one "First line" box where a hanging indent had to be typed as a negative number. A hanging indent now shows as
  Hanging with its size. Choosing First line or Hanging when there was none starts at 0.5" (1.27 cm), as Word does;
  (none) removes it. The document still stores one first-line indent (negative = hanging), so files open and save
  exactly as before (`SpecialIndent` in `crates/ui-egui/src/dialogs.rs`).
- Test: `paragraph_dialog_offers_hanging_and_first_line_indents`.

### README: a KhianKrub front page
- `README.md` (the page GitHub shows for the repository) is rewritten for KhianKrub: name and icon, an English and Thai
  description, new screenshots of KhianKrub (`docs/images/khiankrub-*.png`, credited in `ATTRIBUTION.md`), the Thai
  features, download and install steps linking to the latest release, and how to build it. The licence and credit
  section is kept in full. The original's Discord badges, ArtCraft apps table and WordCraft download instructions
  are gone; the original README is linked for its full documentation. The old screenshots in `docs/images/` stay.
- Then more screenshots: the ribbon before and after, four bugs fixed shown before and after (Thai headings in the
  Navigation pane, space marks on Thai text, the Paragraph dialog's alignment name, File › Recover), and Thai word
  breaking, View only and the Shapes menu. The "before" pictures come from commit `a1538b9` (as forked, plus text
  boundaries), built only to take them.

### Release v0.7.0-thai.4
- [v0.7.0-thai.4](https://github.com/chanonss/khiankrub/releases/tag/v0.7.0-thai.4), built from commit `4d4013e`, file
  `KhianKrub-0.7.0-thai.4-macos-apple-silicon.zip`, SHA-256
  `3684ed741a5fcac93bf387515f7bdb33c6d8aeb253632b51cb27f015e6d56209`. Brings the Navigation pane, File menu, Recover,
  Icons and Shapes changes below. v0.7.0-thai.3 and v0.7.0-thai.2 stay published.

### Insert › Shapes: a picture of each shape
- The Shapes menu shows a small drawing of each shape before its name, as in Word: rectangle, rounded rectangle, oval,
  triangle, diamond, line, block arrow, star, heart, a dashed frame for New Drawing Canvas, and each connector
  (straight, elbow, curved, with or without an arrowhead). Drawn in code in the theme's colours (`shape_item` and
  `shape_preview` in `crates/ui-egui/src/ribbon.rs`). Test: `shapes_menu_items_insert_their_shape`.

### Insert › Icons: category names readable
- The last categories of the Icons dialog (Symbols, Technology, Transport) were squeezed into the space left on the
  row and drawn one letter per line. Category names no longer wrap; the dialog widens a little to show all twelve on
  one line (`icons_body` in `crates/ui-egui/src/dialogs.rs`).

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
