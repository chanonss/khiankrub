# WordCraft main updates for KhianKrub (upstream round before the Windows port)

Status: ported to the web app · 2026-11-05 · Task UP15

Source: WordCraft `main` after v0.15.1 (local ref `refs/upstream-review/wordcraft-main`), chosen from the review in
[`upstream-review-2026-11-04.md`](upstream-review-2026-11-04.md). Upstream is on egui 0.37 and has split many files
(the layout crate, RTF, the table commands) into modules that KhianKrub doesn't have, so most changes were ported by
hand. Each keeps its upstream regression test.

| Change | Upstream | KhianKrub result |
|---|---|---|
| UP14 into the web app: printing with links, `PAGEREF` tables of contents, undefined style formatted as Normal, font size steps per run, Table Properties on a selection, and the rest of [`upstream-v0.14.md`](upstream-v0.14.md) | v0.13.2, v0.14.0 | Taken from the desktop branch; the web print path waits for fonts as before. |
| Lists count as Word does when a deeper item comes first (10.1 → 11), level text and labels bounded | [5108](https://github.com/storytold/wordcraft/pull/5108) | Ported. |
| Tab leader dots spaced and placed as in Word | [5144](https://github.com/storytold/wordcraft/pull/5144) | Ported. |
| Paragraph side borders lie outside the text, at their distance from it | [5036](https://github.com/storytold/wordcraft/pull/5036) | Ported. |
| Paragraphs that each start a continuous section go onto the next page instead of running off it | [3907](https://github.com/storytold/wordcraft/pull/3907) | Ported. |
| Closing the header or footer returns to the body selection | [3331](https://github.com/storytold/wordcraft/pull/3331) | Ported. |
| `.doc`: Word 365 files open in Aptos (the stylesheet's standard font); sections found by an index of paragraph ends | [5066](https://github.com/storytold/wordcraft/pull/5066), [5373](https://github.com/storytold/wordcraft/pull/5373) | Ported. |
| `.doc` reader brought up to main (34 changes: one override per table cell [5374](https://github.com/storytold/wordcraft/pull/5374), a picture decoding budget, compound-file guard, notes, pieces, metadata dates) | `crates/docbin` | Taken whole, except its drawing code, which needs shape and gradient types we don't have; ours stays. |
| DOCX writes the default font size when the document leaves it unset | [4092](https://github.com/storytold/wordcraft/pull/4092) | Ported. |
| A document with many relationships opened in minutes; now one pass | [3809](https://github.com/storytold/wordcraft/pull/3809) | Ported, with the unqualified-attribute test; the signature-line test needs work we don't have. |
| RTF keeps Windows and Enhanced Metafile pictures (they were dropped), sized from their mapping mode | [4648](https://github.com/storytold/wordcraft/pull/4648) | Ported into our single `rtf.rs`. |
| Copy and cut whole tables, rows and cells: Select Table → Cut removes the table, Copy of rows or cells puts a table of them on the clipboard, Cut of rows or cells clears their text | [1784](https://github.com/storytold/wordcraft/pull/1784) | Ported, without the Spike parts (KhianKrub has no Spike). |
| Justified line ending in Shift+Enter; regular-expression `$1` in Replace | | Already in the web app (UP12). |

## Not ported

- Need upstream's split layout files (`table_flow.rs`, `flow.rs`): the picture row that isn't split off (#4272), split
  rows' middle and bottom cells lining up, Line between for columns on selected text.
- Layout resource caps (#3866, #3867: linear-time lookups, linked text-box chains, repeated header rows, a picture
  decoding budget): about 1,150 lines across layout files we don't have. Worth a round of its own; our layout already
  has its own caps, and the `.doc` reader brought its decoding budget.
- Maybe-list items in the review (Insert › Table live preview, highlight colour memory, long comment limit, AutoFormat
  extras, Restrict Editing greying) were left for later.

## Validation

`cargo xtask ci` passed all six checks (format, Clippy, workspace tests, asset attribution, dependency layers and
WASM) on every commit of the round.

## Revision history

| Date | Change |
|---|---|
| 2026-11-05 | UP15: selected WordCraft main changes ported to the web app |
