# WordCraft main updates for KhianKrub (UP16)

Status: batches 1–5 ported to the web app (and so to KrubOffice) · 2026-11-06 · Task UP16

Source: WordCraft `main` from `08afab50` (after v0.15.1, the UP15 base) to `beca3f34` (local ref
`refs/upstream-review/wordcraft-main-1106`), 87 changes. Each was first tried as a cherry-pick in a throwaway worktree:
7 applied cleanly, the rest conflict, mostly because upstream has split files (`props/`, `keys/`, the layout `para/`
modules, `tests/it/`) that KhianKrub keeps whole. Those are ported by hand, each with its upstream regression test.

## Batch 1 (this round)

| Change | Upstream | KhianKrub result |
|---|---|---|
| `.doc` import: paragraph and cell marks capped; table context only for table marks | [5476](https://github.com/storytold/wordcraft/pull/5486) | Cherry-picked. |
| `.doc` import: a field's cached result keeps its formatting per run | [5477](https://github.com/storytold/wordcraft/pull/5489) | Cherry-picked. |
| Text boxes: a press inside one goes to its text, not to a picture under it | [4240](https://github.com/storytold/wordcraft/pull/4240) | Cherry-picked. Its test used `Float.z`, which our model doesn't have: the box comes later in the paragraph instead, and the two cases with a picture stacked above the box are left out. |
| Metafiles: refuse a bitmap no renderer draws before decoding it; DIB bit depth chosen once | [5480](https://github.com/storytold/wordcraft/pull/5490), [5479](https://github.com/storytold/wordcraft/pull/5492) | Cherry-picked. |
| Date & Time in local time | [1782](https://github.com/storytold/wordcraft/pull/1782) | We already had local time (`jiff`, `now_local_iso`). Ported what was still UTC: the weekday (`dddd`) and a date content control's "today", a day behind before 07:00 in Thailand. New `local_day()` with a test. |
| A window restored onto no monitor opens on screen | [4742](https://github.com/storytold/wordcraft/pull/4742) | Ported onto our `ViewportInfo`-based `window_geometry` (a `Monitors` value from winit), with upstream's tests. |
| DOCX keeps a paragraph's `w:textAlignment` | [5498](https://github.com/storytold/wordcraft/pull/5498) | Ported into our single `props.rs`, with upstream's test. |

## Batch 2: fonts that aren't installed are flagged ([4770](https://github.com/storytold/wordcraft/pull/4770))

| Part | KhianKrub result |
|---|---|
| Layout records the fonts the shown text asks for that aren't installed, with the faces drawn instead (`ParaLayout::substituted`, `DocLayout::substituted_fonts`) | Ported into our single `layout/src/para.rs`; `substituted.rs` taken as is. Upstream's hidden-list-item test is left out: our layout has no #3982 collapse. |
| `font.missing`, `format.state.fontShownAs`, upstream's engine tests | Patches applied cleanly (`engine/src/cmd/font.rs`, `format.rs`; tests into `tests_fonts.rs`). |
| Metric twins: a missing font is drawn in an installed twin first, and a twin isn't flagged | Ported. **KhianKrub adds TH SarabunPSK / TH Sarabun New**: measured on this Mac, the same advance widths for Thai and Latin letters, marks, digits and punctuation in Regular and Bold; the Windows line height differs by 0.7% (1310 against 1301 thousandths). The UPC families weren't paired: no files to measure. New Thai test. |
| Font box warning mark and note (Home, mini toolbar, Font dialog's two boxes) | Fitted to our older combo widget: `widgets::font_combo_warned`, `missing_font_tip`, and the mark drawn in code (`paint_font_warning`). The note is the field's accessible label. |
| Status bar "N fonts not installed", the list (20 at most) and Add Font… | Ported, single window (no per-window memo). Counts worded with `tl!` templates (Thai has no plural forms). |
| Thai | Nine strings in `i18n/th.tsv`. |

Checked in the browser pane on the local release build: a new document in Angsana New shows the mark in the font box,
"ไม่ได้ติดตั้งฟอนต์ 1 ตัว" on the status bar, the list (font → Laksaman, the TLWG face drawn instead on the web) and the
Thai hover note. `cargo xtask ci` all 6 pass; the status-bar click test steps past the new item, as upstream's does.

## Batch 3: Windows desktop

| Change | Upstream | KhianKrub result |
|---|---|---|
| Copy and Cut put RTF beside the text and HTML; paste reads another program's RTF | [2630](https://github.com/storytold/wordcraft/pull/2630) | Ported for Windows ("Rich Text Format") and macOS (`public.rtf`), writing and reading; Windows already read it. Linux writing and X11 reading left out (KrubOffice ships for Windows and Mac). `Services::set_clipboard_rich` replaces `set_clipboard_html`. Upstream's engine tests plus one for Thai text through RTF. **The Windows code isn't compiled here** (no Windows target on this Mac): it matches upstream's on the same `clipboard-win` 5.4; the release workflow's Windows build is its first compile. Not tried against a real clipboard (it would replace the owner's). |
| Alt codes type their character in Find and other text boxes | [4403](https://github.com/storytold/wordcraft/pull/5405) | Ported into our single `keys.rs`. Upstream's test returns early on macOS; run here once on the Linux path (macOS check lifted for that run only) and passed. |
| The printer list doesn't wait on drivers; a job stuck on its driver can be cancelled | [5710](https://github.com/storytold/wordcraft/pull/5732) | **Not ported.** It moves craft-print to a newer craft-libs commit (from a different base than our pin) and adds a 592-line isolated print module; it only matters on Windows with a slow network printer. Needs a Windows machine to build and a printer to test: its own round. |

## Batch 4: DOCX fidelity

Upstream's patches applied after mapping its split paths onto ours (`props/para_props.rs` → `props.rs`,
`tests/it/` → `tests/`, `read/story/*` → `read/story.rs`…), conflicts resolved by hand.

| Change | Upstream | KhianKrub result |
|---|---|---|
| A text frame's anchor lock kept | [5382](https://github.com/storytold/wordcraft/pull/5382) | Ported; docx gains `serde_json` as a dev-dependency for its test. |
| A paragraph's adjust-right-indent kept, through styles | [5384](https://github.com/storytold/wordcraft/pull/5384) | Ported; added to our single overlay list. Upstream's `run_props` test as `adjust_right_indent.rs`. |
| Nil shading paints no fill; border theme colours win | [4705](https://github.com/storytold/wordcraft/pull/4705) | Ported; our cell diagonal borders read through the same `PropCtx` readers. |
| The nine compound border lines kept | [5499](https://github.com/storytold/wordcraft/pull/5499) | Ported for DOCX and the RTF reader; our RTF writer has no paragraph borders, so upstream's export half and its round-trip test are left out; a new RTF import test instead. |
| Section printer trays, chapter page numbers, `w:noEndnote`, `w15:footnoteColumns` kept | [5311](https://github.com/storytold/wordcraft/pull/5311) | Ported by hand (upstream's base had ODT section fields we don't). `w15` is declared on our document root. |
| Picture brightness, contrast and transparency | [1961](https://github.com/storytold/wordcraft/pull/1961) | **Not in this batch**: 47 files across layout, render, PDF and the picture commands. Its own round. |

## Batch 5: layout

| Change | Upstream | KhianKrub result |
|---|---|---|
| Justified lines keep the width of the spaces before their first character | [4307](https://github.com/storytold/wordcraft/pull/4307) | Ported into our inline justification, Thai Distribution included. KhianKrub also keeps leading spaces out of the room a Word 2013+ line may shrink. Upstream's test runs in Word 2010 mode: our space-shrinking model predates upstream's `88d936c3` ("shrink their spaces by Word's measured limits", 2026-10-20, never ported), so a justified line there breaks later than upstream's; a check that shrinking leaves leading spaces alone is added. |
| Ordinary tabs follow paragraph alignment | [3915](https://github.com/storytold/wordcraft/pull/3915) | Ported; we have no positional tabs, so that exception is left out. |
| The empty line after a trailing line break: its caret follows the alignment | [4293](https://github.com/storytold/wordcraft/pull/4293) | Ported. |
| Times and Courier take Times New Roman's and Courier New's line spacing (Mac) | [3241](https://github.com/storytold/wordcraft/pull/3241) | Ported. |
| A line of only pictures takes no descent; multiple spacing adds its share | [3984](https://github.com/storytold/wordcraft/pull/3984) | **Not ported**: our line measuring predates upstream's base for it (lowered pictures, legacy check boxes), and it changes every line with an inline picture. |
| Room below bottom paragraph borders; a section's page start; floats in cells; floats on later pages | [2437](https://github.com/storytold/wordcraft/pull/2437), [5289](https://github.com/storytold/wordcraft/pull/5291), [4545](https://github.com/storytold/wordcraft/pull/4545), [4250](https://github.com/storytold/wordcraft/pull/4267) | **Not ported**: they build on upstream's reorganised page flow (`flow.rs`, `boxes.rs`, `floats.rs`, "rooms" for borders), which our `lib.rs` doesn't have. |

## Next batches

**Paused after batch 5 (owner, 2026-11-07).** The round stops here; the batches below wait until there is a reason to
take them (for the layout catch-up: a user's document with pictures in table cells, say, where #4545 alone can come first).


1. **Layout catch-up**: upstream's page-flow and float reorganisation, then #3984, #2437, #5289, #4545, #4250, and the
   measured space-shrinking model (`88d936c3`). A round of its own, checked against Thai documents page by page.
2. **Larger**: picture brightness/contrast (1961), Undo memory (2694, 50 files), N-up and booklet printing (3916), status-bar count choice (3187), modal
   dialogs (4076), Font dialog preview (4058), ODT full support (5559, 30k lines: its own round).

**Skip**: Arabic interface, Japanese line breaking, Linux (Wayland/X11, RPM, Speech Dispatcher), CI and Dependabot,
mail-merge CSV, MCP/control-channel changes, page host mode for the web (unused by the site).
