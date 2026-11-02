# WordCraft v0.13.2 and v0.14.0 updates for KhianKrub

Status: ported to the desktop app · 2026-11-02 · Task UP14

Source releases: [WordCraft v0.13.2](https://github.com/storytold/wordcraft/releases/tag/v0.13.2) (2026-10-31) and [v0.14.0](https://github.com/storytold/wordcraft/releases/tag/v0.14.0) (2026-11-01). Upstream has since moved to egui 0.37 and split many files into modules; KhianKrub stays on egui 0.36 with its own file layout, so each change below was ported by hand (none cherry-picked cleanly) and keeps its upstream regression test.

| Change | Upstream | KhianKrub result |
|---|---|---|
| Printing: the printer's PDF leaves out link annotations, so a document with a link or a table of contents prints (the driver refused to fit annotated pages to the paper). Save these pages as PDF and exported PDFs keep their links. | [4817](https://github.com/storytold/wordcraft/pull/4817) | Ported. |
| macOS launch arguments (`-psn_…`, `-NS…`, `-Apple…`) are not opened as files; a failed open from the command line, Finder or a non-Unicode path names the file and the cause on the status bar (Thai: เปิด {path} ไม่ได้). | [2622](https://github.com/storytold/wordcraft/pull/2622) | Ported, except upstream's plumbing for opens held back by a Save prompt, which needs modules the fork doesn't have. |
| PDF export writes a glyph id above 65535 as .notdef instead of failing the export. | [4554](https://github.com/storytold/wordcraft/pull/4554) | Ported. |
| Page parameters (`edit.goto`, `caret.set`, `file.exportPng`, MCP `render_page`) saturate instead of wrapping to page 1 on 32-bit and wasm. | [4289](https://github.com/storytold/wordcraft/pull/4289) | Ported. |
| A kept heading moves with a three-line paragraph that widow control moves whole. | [4991](https://github.com/storytold/wordcraft/pull/4991) | Ported. |
| Built-in heading styles have their outline level without `w:outlineLvl`, so the Navigation pane lists them; `w:customStyle="on"` marks a custom style. | [4706](https://github.com/storytold/wordcraft/pull/4706) | Ported. |
| A paragraph naming an undefined style is formatted as Normal, as in Word. | [5104](https://github.com/storytold/wordcraft/pull/5104) | Ported. |
| A hyperlinked table of contents writes its page numbers as `PAGEREF` fields, as Word does. | [5097](https://github.com/storytold/wordcraft/pull/5097) | Ported. |
| Format Painter paints a double-clicked word or triple-clicked paragraph. | [4502](https://github.com/storytold/wordcraft/pull/4502) | Ported. |
| Increase/Decrease Font Size and Grow/Shrink by 1 pt step each selected run from its own size. | [4910](https://github.com/storytold/wordcraft/pull/4910) | Ported, without the text-box target (the fork's formatting commands don't target a selected text box). |
| Table Properties: row, column and cell settings apply to every selected row, column and cell, in one undo step. | [4896](https://github.com/storytold/wordcraft/pull/4896) | Engine ported; the dialog's mixed check boxes are not. |

## Validation

`cargo xtask ci` passed all six checks (format, Clippy, workspace tests: 2,888 passed, 0 failed, 3 ignored; asset attribution, dependency layers and WASM).

## Not ported

- Not needed here: [4755](https://github.com/storytold/wordcraft/pull/4755) (the fork's HTML import has no relative `<font size>` arithmetic to overflow), [4659](https://github.com/storytold/wordcraft/pull/4659) (the fork's Markdown import has no reference links), [4505](https://github.com/storytold/wordcraft/pull/4505) (the fork's Shading buttons already show the last colour).
- Later, in a batch of its own: [1784](https://github.com/storytold/wordcraft/pull/1784), copying and cutting whole tables, rows and cells (585 lines across the document model, clipboard and table commands).
- Not relevant to the macOS and Windows builds: Flatpak, FreeBSD and X11 fixes, CI changes. Left for later: the Macros dialog, CJK word counting, the proofing engine rework, keytips for Backstage pages.

## Revision history

| Date | Change |
|---|---|
| 2026-11-02 | UP14: selected v0.13.2 and v0.14.0 changes ported to the desktop app |
