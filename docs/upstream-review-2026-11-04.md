# Upstream review before the Windows port (2026-11-04)

Status: review for the owner to decide · nothing ported yet

What the three upstream projects added since our forks last took from them, and what is worth bringing into
KhianKrub, PhutKrub and KhitKrub before the Windows port. "Worth it" is judged for Thai users of the web apps and the
coming Windows builds.

| App | Upstream | Our base | New upstream work | How it comes in |
|---|---|---|---|---|
| KhianKrub | storytold/wordcraft, now v0.15.1 | 2026-10-15 (`eae24217`) | 1,458 commits, 89 progress-log entries | By hand, as in UP14: the trees have diverged (egui 0.37, split files) |
| PhutKrub | storytold/deckcraft, now v0.15.0 | 2026-10-30 (`3a4e30e1`) | 258 commits | A merge: 23 files conflict (today's Presenter View and drafts work, branding, docs) |
| KhitKrub | storytold/gridcraft, now v0.16.0 | 2026-10-31 (`981fad14`) | 199 commits | A merge: 8 files conflict (icons, theme, widgets, docs) |

## KhianKrub (WordCraft)

Already known from the last round: the desktop app has the UP14 fixes (`docs/upstream-v0.14.md`) but the web app
doesn't, and table copy/cut (#1784) is still waiting.

**Worth it**

| Change | Why for us |
|---|---|
| UP14 fixes into the web app (printing with links, PAGEREF table of contents, undefined style = Normal, font size steps, Table Properties on a selection…) | The web and desktop apps should behave alike before Windows |
| Copy and cut tables, rows and cells (#1784) | Everyday table editing |
| Tab leaders drawn as Word draws them (measured on TH SarabunPSK) | Thai forms and tables of contents use dot leaders |
| List numbers count as Word does when a deeper item comes first (10.1 → 11) | Numbered Thai documents (ข้อ 1, 1.1…) |
| A justified line ending in Shift+Enter stretches to the margin | Justified Thai text looked different from Word |
| Paragraphs that each start a continuous section go onto new pages (they ran off the page) | Layout bug |
| Table rows: a picture isn't split off; middle/bottom cells of a split row line up | Tables in reports |
| Paragraph side borders outside the text; Line between for columns on selected text | Layout matching Word |
| `.doc` import: standard font (Aptos), sections found by index, table cell overrides | Thai offices still send `.doc` |
| DOCX: tables inside `mc:AlternateContent`, default font size kept on save, RTF metafile pictures | File fidelity |
| Header/footer: a click in the other one edits it; closing returns to where you were | Everyday editing |
| Find and Replace: regular expressions expand `$1` capture groups | Correct replacing |
| Security and speed caps (`.doc` import, pasted text boxes, layout resource caps, many relationships, comments save) | The never-crash rule |

**Maybe**

| Change | Note |
|---|---|
| Insert › Table live preview on the page | Nice; UI only |
| Text Highlight Color remembers the last colour | Small |
| Long comments stop at ten lines | Only if people comment a lot |
| Multiple documents and one native window per document (desktop) | Large; belongs with the Windows port if at all |
| AutoFormat: `*bold*` `_italic_`, heading styles, tab indents (all off by default) | Rarely used |
| Restrict Editing greys out edits; content control formatting | Forms users only |

**Skip**

Language packs and Options › Language redesign (European spelling packs; KhianKrub's Thai needs differ), CJK word
counting, Spanish and French proofing, Arabic interface, mail merge record navigation, MCP and parity-catalog changes,
Flatpak, FreeBSD and Linux-only fixes, CI.

## PhutKrub (DeckCraft)

**Worth it**

| Change | Why for us |
|---|---|
| Presenter View in its own browser window (#730) | Upstream built the same feature, more completely: the presenter sees the audience's real frame (animation steps too), with Retry and Reopen buttons and origin-checked messages. **Recommendation: take upstream's version and drop ours**, keeping our Thai labels and the blocked pop-up note |
| File › About replaces the Account page | Cleaner File tab |
| No Fill shapes let clicks through to what's under them | Everyday editing |
| Comments pane keeps its width; Add Animation menu scrolls | Bug fixes |
| A paragraph's own numbering start; automatic lists without Replace text as you type; symbol-font bullets keep their character | Lists |
| PPTX fidelity: table right-to-left, alternate language, reflection, tiled fills | Files from PowerPoint |
| Security and speed bounds (fonts, rendering, PPTX read/write, decoders) | The never-crash rule |

**Maybe**: Format Shape effect presets and Scale Height/Width; Reuse Slides scaling and table styles; Duplicate naming
like PowerPoint; read-only `slide.actions`.

**Skip**: long English date formats, benchmarks, Flatpak, CI.

## KhitKrub (GridCraft)

**Worth it**

| Change | Why for us |
|---|---|
| CSV and text open detect each column's date order (31/05/2026 → day first) | Thai dates are day first |
| Frozen panes keep their text, gridlines and borders while scrolling | Bug fix |
| Formulas › Function Library as labelled buttons, Recently Used, Insert Function | The Formulas tab people use |
| Find and Replace within the selected range | Everyday |
| Google Sheets' exported formulas open as the original formula | Our users come from Google Drive |
| Dialogs take ranges by selecting on the grid (Goal Seek, Solver, …) | Dialog usability |
| Floating objects share one selection with resize handles | Charts and pictures |
| The formula bar can be resized; drag-and-drop asks before replacing data; R1C1 kept | Everyday |
| Security and speed bounds (formulas, PivotTables, ODS, XLSX, PDF export) | The never-crash rule |

**Maybe**: chart data tables, Solver options/reports and Analysis ToolPak, opening SYLK and web-page tables, Excel 2003
menu access keys, Trace Error arrows.

**Skip**: Python cells (code is stored, never run; adds weight for no use here), equations in sheets, pasting pictures
from the system clipboard (desktop only), GNOME/Wayland window fixes.

## Suggested order

1. **KhitKrub first**: smallest merge (8 conflicts, mostly docs and theme), clear wins (dates, frozen panes).
2. **PhutKrub**: merge, taking upstream's Presenter View in place of ours; re-check drafts, Thai labels and the
   Save Changes prompt after the merge.
3. **KhianKrub**: by hand, in batches: UP14 into the web app, then layout and list fixes, then file fidelity and the
   security caps, then table copy/cut.
4. Then the Windows port of all three.

Each step: CI, a look at the result in Thai, README and Thai release-note drafts for the owner.
