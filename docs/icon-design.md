# KrubOffice icon design

Status: 2026-11-01 · ICON.6 approved square ท Thai Distribution icon; PhutKrub paragraph alignment added locally.

Quiet Glass uses original 24×24 drawings, usually inside a 20×20 area, with 1.75-unit outlines and round caps. Display sizes remain controlled by each app. At 16 px the drawings omit fine details and keep at least a 1 px stroke. Foreground ink follows the caller, including its alpha; gold tints disappear on flat band icons and disabled icons. Common action mappings change painting only, never command IDs, tooltips, enabled state or hit targets.

`crates/krub-icons` is an identical vendored library in KhianKrub, KhitKrub and PhutKrub. Each copy uses the host workspace's egui version (0.36 or 0.37), so a checkout builds independently. Changes to this library must be copied to all three repositories and checked with `python3 packaging/check-icon-sync.py <sibling-repo> <sibling-repo>`; no cross-repository path dependencies. The catalog includes 574 distinct names (including aliases), spanning 318 document icon spellings, 131 spreadsheet variants and 246 presentation variants, plus document menu diagrams. Aliases route common actions to identical geometry. Specialized symbols retain their original meaning, adapted from contributor-original Rust artwork to shared proportions, stroke weight and neutral ink with translucent gold surfaces. This consolidates and restyles the existing openly licensed drawings; it does not claim every path was newly invented. Content previews (actual inserted pictures, shape/style galleries, worksheet and document colours) continue to depict the content they represent.

Logo masters are the app-named SVGs under `assets/app-icon/`. Their gold glass treatment comes from the reviewed richer-gold draft; the existing Thai contours are from Sarabun Bold, SIL OFL 1.1, with its licence beside the artwork. Web loading screens embed the master; install icons are 192/512 px PNG renderings. Start-screen marks use a clipped colour gradient over the app's existing Thai font, so they remain usable without an SVG loader. Native installer icons and the separate KrubOffice launcher are outside this first preview.

## Validation history

ICON.1 initial preview, 2026-11-01: native offscreen application previews captured and inspected; icon tests (finite tessellation at 16/20/24/32 px in normal, flat and disabled states; invalid bounds ignored), shared-crate Clippy with warnings denied, formatting, asset attribution, dependency layering and UI WASM checks passed in all three repositories. Vendored library equality checked. Full workspace CI and complete icon migration remain pending; no commit, push or publication in this preview stage. Existing unused-patch, optional craft-fonts and WASM dead-code warnings remain.

## Balanced toolbar gold · ICON.3

The owner approved Balanced after reviewing the palette comparison. Toolbar gold is `#F2B705` with primary alpha 230/255 and surface alpha 107/255. Caller alpha and drawing opacity are multiplied into those strengths, without a minimum opacity floor. The deck toolkit identifies soft surfaces through its explicit tint role, rather than classifying rendered pixel alpha. Document fills and spreadsheet primary rectangles share the same primary role; worksheet UI washes and surface rectangles use the soft role. Spreadsheet rectangles resolve their surface role once, avoiding double tinting. Flat band icons and disabled controls remain monochrome, including specialized document/spreadsheet routes. The preserved workbook content renderer and document artwork are unchanged.

## Revision history

| Date | Change |
|---|---|
| 2026-11-01 | ICON.3: approved Balanced gold roles; preserve dimmed alpha and monochrome flat/disabled states. |
| 2026-11-01 | ICON.1: shared library and first common action mappings; richer gold glass web logos and start-screen lettering. |

| 2026-11-01 | ICON.2: full shared catalog, common action aliases, document-menu migration, complete inventory/mesh checks and gallery tooling. |

## ICON.2 final verification · 2026-11-01

Full `cargo xtask ci` passed. After the final round-cap adjustment, all 9 shared-catalog tests, Clippy and the UI WASM check passed again. Native screenshots and the full 573-name gallery were rendered and inspected.

The three vendored library copies are byte-identical. Commands, shortcuts and document format handlers retain their IDs and behaviour. Changes remain local and uncommitted; no GitHub push, deployment or social publication was performed.

## ICON.3 verification · 2026-11-01

All 11 shared-icon tests pass against egui 0.36.2 and the exact egui 0.37 fork revision used by KhitKrub/PhutKrub. Library Clippy with warnings denied and library WASM checks pass for both. Source formatting and vendored-library equality checks pass. The equality helper ignores Finder's `.DS_Store` metadata while comparing every source/configuration file.

Actual tessellated source icons were rendered without a GPU and visually inspected in light/dark themes, including 32/16 px, dimmed caller opacity, disabled and flat states. The complete 573-name catalog was rendered and inspected at 28/16 px. These are source-icon previews, not complete application ribbon screenshots.

Validation used isolated manifests and a source snapshot verified byte-identical to all three vendored copies. Full workspace CI, whole-app WASM checks and complete app screenshots were not rerun during ICON.3; the broader gate limitations in ICON.2 remain to be resolved before committing/releasing. No commit, push, deployment or social publication was performed.

## Full local web previews · 2026-11-01

Rebuilt all three complete web apps from the updated working trees into separate temporary preview folders. The sample document (KhianKrub), sales workbook and charts (KhitKrub), and Northern Lights presentation (PhutKrub) loaded successfully in the in-app browser with Thai UI labels and the Balanced icons. No runtime errors appeared in the captured browser logs; the existing unsupported web viewport title/theme warnings are non-fatal. Home toolbars were visually inspected in all apps; KhianKrub's Insert ribbon was also checked. This is a browser startup/visual smoke check, not full workspace CI.

Local links: KhianKrub `http://127.0.0.1:8771/?sample`, KhitKrub `http://127.0.0.1:8772/?sample=sales`, PhutKrub `http://127.0.0.1:8773/?sample`. The servers bind only to loopback. Fresh WASM hashes differ from the old pre-redesign `dist/web` bundles. HTML assets and manifest icon paths were checked. No public release or deployment.

## Approved Thai Distribution icon · ICON.6 · 2026-11-01

The owner reviewed `thai-distribution-square-draft.png`, approved it, and requested Thai Distribution in PhutKrub too. The icon uses an 18×18 square on the 24-unit master with a 1.75-unit neutral outline, Balanced soft gold fill, and a centered ท derived from the existing Sarabun SemiBold face. The outline is preflattened Rust geometry with a true counter/hole, so the glyph is accurate and does not depend on installed Thai fonts. Its SIL OFL licence is included in `crates/krub-icons/OFL-Sarabun.txt` and attributed. Flat and disabled states keep the established monochrome/alpha rules. Named document `thaiDistribute` and presentation `Icon::ThaiDistribute` / `thai-distribution` routes use identical painting. No font binaries were added.

PhutKrub gains Home › Paragraph › Thai Distribution immediately after Justify, plus the corresponding paragraph menu/dialog option and Thai tooltip. `format.thaiDistribute` is a command-registry entry with normal enabled state and undo/redo. The model preserves a distinct Thai alignment (`thaiDist` in DrawingML) instead of importing it as ordinary Justify. Wrapped lines share space at Thai grapheme/cluster boundaries and spaces; combining marks remain attached, Latin letters within words retain their spacing, and final paragraph lines remain unstretched. Generic alignment/paragraph command parameters accept `thaiDistribute` too. KhianKrub keeps its existing command/behaviour and receives the new icon only. KhitKrub's vendored library is synchronized without adding a spreadsheet command.

Targeted validation details and final build identifiers are recorded in the local handoff. Full workspace CI remains a separate gate before commits or release.

| Date | Change |
|---|---|
| 2026-11-01 | ICON.6: approved font-independent square ท icon and PhutKrub Thai Distribution. |

### ICON.6 final verification

All 12 shared-icon tests passed on both egui versions; library Clippy with warnings denied passed for both; vendored sources are identical. All three asset-attribution checks passed. PhutKrub's two command/PPTX/undo tests, two Thai/mixed-language layout tests, and the native ribbon-click regression passed. The latter verifies the button follows Justify and applies `Align::ThaiDistribute`. Complete KhianKrub and PhutKrub web builds passed and their new buttons were visually inspected and clicked without browser runtime errors. Formatting and whitespace checks passed. Full workspace CI was not rerun; no commit or publication.

## Clipboard icon pilot · 2026-11-08

Chanon requested a small set before the larger icon redraw. KhianKrub has original vector drawings for Paste, Cut, Copy and Format Painter in `crates/ui-egui/src/icons.rs`, on a shared 24-unit grid with consistent stroke and restrained gold. The pilot is intentionally local to KhianKrub pending owner review, so the shared vendored `krub-icons` crate remains unchanged. It uses the existing ribbon commands, hit targets and tooltips. The set was inspected at the actual 20 px and 32 px ribbon sizes in light and dark themes. The 27 ribbon tests, UI Clippy, WebAssembly check, formatting and diff checks passed; full CI and broader rollout remain.

## Home ribbon icon set · 2026-11-08

After the Clipboard pilot, Chanon requested the rest of KhianKrub's Home ribbon first. Original vector artwork on the same 24-unit grid now covers the Home font, paragraph, editing, voice and editor actions as well as Clipboard. The previously approved Thai Distribution icon remains unchanged, and style previews and navigation chevrons retain their existing purpose. The artwork is currently local to KhianKrub; commands, tooltips and button behavior are unchanged. Home icons reused elsewhere in KhianKrub also use the new artwork. Further ribbons and the other apps have not been redesigned in this pass.

### Home submenu icon follow-up · 2026-11-08

The Home submenu pass covers the existing Paste, Find and Select menu icons, plus original border-menu diagrams for each edge, inside and diagonal option, horizontal line, table drawing, gridlines, and borders/shading. Active edges use dark ink and inactive edges use continuous light grey guides; disabled entries retain the disabled ink. The border menu is shared with Table Design, so that menu receives the same diagrams. Text-only choices and content previews remain text/content, and existing menu commands and enabled states are unchanged.

## Insert ribbon icons · 2026-11-08

KhianKrub's Insert ribbon and its table and page-number submenu diagrams use original vector artwork on the Home set's 24-unit grid. Shared actions reuse the Home set. Content galleries keep showing the actual shapes, equations, symbols and WordArt they insert. The artwork is local to KhianKrub for review; command behavior remains unchanged.

## Draw ribbon icons · 2026-11-08

KhianKrub's Draw ribbon uses the same original vector style, including pen tools, colour and thickness, conversion, canvas and replay. Add Pen has matching menu icons; the pen gallery's colour/thickness swatches remain functional content indicators. Shared icon names can update other contexts too.

## View ribbon icons · 2026-11-08

KhianKrub's View ribbon uses matching original vectors for view modes, page movement, Show, Zoom, theme, window, macro and template commands. The previous View layout and commands remain unchanged. Text-only submenu choices remain text.

## Layout ribbon icons · 2026-11-08

KhianKrub's Layout ribbon uses matching original vectors. Existing text-direction, margin, orientation, column, break and line-number submenu diagrams were redrawn to distinguish each option while retaining their commands and selected states. The paper-size and arrangement menus that use text-only options remain text.

## Design ribbon icons · 2026-11-08

KhianKrub's Design ribbon and its existing Paragraph Spacing submenu icons use the original vector style. Style/theme previews and color palettes continue to show the actual document choices; text-only entries remain text.

## References ribbon icons · 2026-11-08

KhianKrub's References ribbon uses matching original vector icons, with distinct images for table of figures and table of tables. Existing text-only menus remain text and reference commands are unchanged.

## Review ribbon icons · 2026-11-08

KhianKrub's Review ribbon uses matching original vectors for its proofing, speech, accessibility, language, commenting, change tracking and protection controls. Existing text-only submenu entries are preserved.

## Picture Format and Table Layout icons · 2026-11-08

KhianKrub's Picture Format distinguishes the picture adjustment actions and has matching crop and alt-text icons. Table Layout includes matching table actions, nine distinct cell-alignment diagrams, and Select/Delete/AutoFit submenu diagrams. Existing commands and selected states remain unchanged; common icons may be reused in other tabs.
