# Handoff: continuing KhianKrub on another Mac

Written 2026-10-21 for Chanon Sarasuk, updated 2026-10-22 on the second Mac. It lets a new Claude Code session (or you)
pick up exactly where the last one stopped. Nothing here is secret: no passwords or tokens.

**A new session should read this file and [`FORK-CHANGES.md`](FORK-CHANGES.md) first.** Claude Code's memory does not
travel between Macs (see section 10).

## 1. Where things stand

- **App:** KhianKrub (เขียนครับ), Chanon's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft) for Thai users
  (Rust, egui). Fork: <https://github.com/chanonss/khiankrub>, branch **`my-changes`** (the default branch).
  The repo was renamed from `chanonss/wordcraft` on 2026-10-23; GitHub redirects the old address (web, releases,
  git). **Never create a new repo named `wordcraft` under chanonss**: that would break the redirect.
- **Done, in order:** Thai Distribution and Thai word breaking, View only mode, English-only interface (22 other
  languages deleted), hover-jump fix, plain number fields, Thai space-dot centring, ArtCraft logos removed, rename to
  KhianKrub with the gold ข icon on blue (traced from Sarabun Bold, OFL), and on 2026-10-22 **Thai list numbering**
  (`thaiNumbers` ๑ ๒ ๓, `thaiLetters` ก ข ค, `thaiCounting` หนึ่ง สอง สาม, checked against Word's own labels), then
  **ribbon icons sized and centred like Word's**, and the name written as one word, **KhianKrub**. Every
  change is listed, newest first, in [`FORK-CHANGES.md`](FORK-CHANGES.md).
- **Published:** release `v0.7.1` (2026-10-23, tag on `06b64e40`, the latest; plain version numbers from here on),
  `v0.7.0-thai.4` (tag on `4d4013e`), `v0.7.0-thai.3` (tag on `e893820`,
  the first as KhianKrub) and the older
  `v0.7.0-thai.2` (tag on `13f20be`, old WordCraft name and icon), kept on purpose.
- **Everything is on GitHub** (pushed 2026-10-23). Later local commits, if any, show with
  `git log --oneline origin/my-changes..HEAD`. Chanon decides when to push (rule 1).
- **Untracked scratch files in `repo`, not to be committed:** `issue-draft.md`, `thai-test.txt`.
- **Outside the repo, in the project folder** (`~/Documents/Codex/WordCraft/` on the second Mac):
  - `thai-test.docx`: a Word test document covering Thai word breaking, the five alignments including Thai
    Distributed, tone marks, mixed Thai/English, Thai digits, character formats, six Thai fonts, three Thai list types
    and a poem.
  - `make_thai_docx.py`: regenerates it (`python3 make_thai_docx.py thai-test.docx`; plain Python, no libraries).

## 2. Set up another Mac

**Where the code lives (since 2026-10-24):** a git clone at `~/dev/khiankrub` on each Mac (`C:\Users\csara\dev\khiankrub`
on the PC), synced through GitHub: pull before starting on a machine, commit and push before leaving it. Never keep the
repo in Google Drive (Drive drops executable bits and can half-sync `.git`). The Drive folder `My Apps/KhianKrub/` holds
only non-code: `facebook/`, `release/`, `thai-test.docx`, `make_thai_docx.py`.

```bash
xcode-select --install                                   # compiler tools, if missing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust (rustup)
rustup target add wasm32-unknown-unknown                 # needed by `cargo xtask ci` (the wasm step)
git clone https://github.com/chanonss/khiankrub.git && cd khiankrub && git checkout my-changes
cargo install resvg --locked                             # only needed to regenerate the icon files
```

- **Do not skip the wasm target.** On the second Mac the first `cargo xtask ci` failed at the last step with
  "can't find crate for `core`" until it was added.
- Apple silicon Mac for the app build (the packaged app is arm64 only, macOS 11 or later).
- First build takes several minutes. `cargo build --release -p wordcraft` is about 3 to 4 minutes.
- GitHub: Chanon signs in themself (never ask for or type a password). `git push` over HTTPS needs a GitHub login or
  token on that Mac; on the first Mac `GIT_TERMINAL_PROMPT=0 git push origin my-changes` worked because credentials were
  already stored. Pushing has not been tried from the second Mac.
- Commit identity is passed per commit, not stored in git config:
  `git -c user.name="Chanon Sarasuk" -c user.email="chanon.s@rmutk.ac.th" commit ...`, with the message ending in a
  `Co-Authored-By: Claude … <noreply@anthropic.com>` line naming the Claude model in use.

## 3. Everyday commands

| Task | Command |
|---|---|
| Full check before every commit | `cargo xtask ci` (fmt, clippy -D warnings, tests, assets, layers, wasm) |
| Format | `cargo fmt --all` |
| Build the release binary | `cargo build --release -p wordcraft` |
| Make the Mac app | `packaging/macos/local-app.sh` → `~/Applications/KhianKrub.app` (ad-hoc signed, this Mac only) |
| Render a document to PDF with KhianKrub | `cargo build --release -p wordcraft-cli` then `target/release/wordcraft-cli convert in.docx out.pdf` |
| Regenerate all icon files | `packaging/icons.sh` (needs `resvg`; uses `assets/app-icon/wordcraft.svg`) |

**Chain them so a failure stops everything.** Twice, a piped or chained command hid a failed check and an old build was
packaged. Use `set -euo pipefail`, no `| tail` on the checking command itself:

```bash
cargo xtask ci && cargo build --release -p wordcraft && packaging/macos/local-app.sh
```

The zip for a release is made with `ditto -c -k --keepParent --norsrc --noextattr "KhianKrub.app" <name>.zip`. Unzip a copy,
run `codesign --verify --strict --deep`, and put the zip's SHA-256 in the release notes.

## 4. Working rules Chanon has set (keep following them)

1. **Push only when told.** Make small changes, commit locally after each (check passes first), rebuild the local app so
   Chanon can try it, say how many commits are waiting, and push everything in one go when Chanon says so.
   Do **not** create, replace or delete a GitHub release without asking first ("tell me before you delete the old release").
2. **Update `FORK-CHANGES.md`** with a dated entry for every change. The original's own progress docs (`ROADMAP.md`,
   `docs/`) are not edited in this fork, even where they are now out of date (for example `docs/layout-parity.md` still
   says Thai list formats show decimal).
3. **English-only interface.** Write new labels in English only (`tl!("...")`). Do not translate them, do not run or mention
   language tests, and do not ask again about a Thai menu: settled. Thai support in documents (Thai text, Thai
   Distribution, word breaking, Thai fonts, Thai list numbering) stays.
4. **Credit the original, never use its brand.** Keep `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, the README note and the About
   screen credit ("Based on WordCraft by the ArtCraft team. Fork by Chanon Sarasuk."). The original's brand licence forbids
   ArtCraft logos and marks in published forks: plain-text credit only (`docs/brand/` was deleted on purpose).
5. **Every asset file needs a row in `ATTRIBUTION.md`** (`cargo xtask assets` fails otherwise); new third-party material also
   needs a `NOTICE` entry. The app ships its licences inside (`local-app.sh` copies them into Contents/Resources).
6. **Never put a password, token or key into any page.** Chanon signs in to GitHub in the built-in browser themself; there is
   no `gh` CLI. Chanon attaches the release zip in the file picker.
7. Chanon writes in English. Replies in Thai only when Chanon asks. Facebook posts for the Thai group are written in Thai,
   without first-person pronouns.
8. Never use Microsoft, Adobe or similar assets (see `CLAUDE.md`). Never write panics in non-test code. Word may be
   observed black-box with synthetic documents (CLAUDE.md, "Clean room"); files Word produces are never committed.

## 5. The icon and the name

- **Name:** KhianKrub (one word, since 2026-10-22; it was "Khian Krub" before) in English letters for files and bundle ids, เขียนครับ as the Thai display name (Finder on a Thai Mac,
  via `th.lproj/InfoPlist.strings` written by `local-app.sh`). Bundle id `io.github.chanonss.khiankrub`, executable `KhianKrub`.
  "ครับ" is the male polite ending; Chanon knows and accepted it. A web search found no clash for "Khian Krub" (searched with the space; software names
  only, not trademark registers).
- **Icon:** the letter ข in gold `#F2B705` on a deep blue `#1E3A8A` rounded square (rx 112 in a 512 view box). The outline is
  traced from **Sarabun Bold** (Cadson Demak, SIL OFL 1.1, no reserved font name), chosen over TH Sarabun New (GPL) and
  Thonburi (Apple's). The path is stored in `assets/app-icon/wordcraft.svg`, nudged `translate(-11 -7)` so the visual weight is
  centred ("B2 balanced"). The licence text is `assets/app-icon/OFL-Sarabun.txt`. To redo the trace: get `Sarabun-Bold.ttf`
  from <https://github.com/cadsondemak/Sarabun>, read the outline of U+0E02 with CoreText (a short Swift script that
  walks `CTFontCreatePathForGlyph`), scale to 300 units tall inside the 512 box, and write the path as an even-odd fill.
- File names `wordcraft.svg`, `wordcraft.icns`, `wordcraft.ico`, `ai.storyteller.wordcraft.*` were **kept** so no code path
  changes.
- **Not yet confirmed by Chanon:** the second Mac's app was built and opened, but Chanon has not yet given feedback on
  the icon and name.

## 6. Deliberately left as the original (do not "fix")

- Crate names `wordcraft-*`, the internal app id passed to `eframe::run_native("WordCraft", ...)`, icon file names.
- Settings folder `~/Library/Application Support/WordCraft` (existing settings survive the rename).
- Info.plist keys `WordCraftVersion` and `WordCraftBuildCommit`.
- `packaging/macos/package.sh` and the DMG background: the original's release tooling. `package.sh` only follows the new
  executable and icon names; the DMG still shows the old icon and name. Not used for Chanon's zip releases.
- The Windows installer files (`packaging/windows/package.ps1`, `wordcraft.wxs`) and the `.msi`/`.zip` names still say
  WordCraft. The `.exe`'s own file properties (product name, icon, copyright) already say KhianKrub (`apps/wordcraft/build.rs`).
- The sample document's link to getartcraft.com. (The README was rewritten for KhianKrub on 2026-10-23.)
- The File menu's "Discord community" and the Help tab's community button still exist (only the title-bar button was removed).
- Merging the original's later changes will conflict on `crates/ui-egui/src/i18n/` (deleted here).

## 7. Technical gotchas learned

- egui 0.36: a hovered, unframed selectable label grows by 2 px because the frame stroke counts. All such labels go through
  `SteadyUi` (`steady_label`, `steady_value`, `steady_toggle`, `steady_scope` in `crates/ui-egui/src/widgets.rs`). Use them for
  new tab strips or lists. `hover_stability.rs` sweeps every dialog (the full sweep is `--ignored`).
- Number fields use `NumField` (`numfield.rs`), not `DragValue` (no drag, no resize cursor).
- UI tests read the AccessKit tree: a text box exposes a `TextInput` node and a `TextRun` child with the same value; pick the
  `TextInput`.
- A stale widget id refocused while the canvas is undrawn crashes AccessKit; `refocus_after_chrome_click` guards it.
- Thai in the interface: the embedded UI fonts have no Thai; `thai_ui_faces` (theme.rs) adds an installed Thai font
  (Thonburi on macOS) as a fallback, so Thai headings and names draw. The menus stay English (rule 3).
- A release zip must be made with `ditto ... --norsrc --noextattr`; `codesign --strict` rejects extended attributes.
- **List and page number formats** are drawn by `NumFormat::format` in `crates/doc/src/section.rs`. Every Word format is kept
  on save; the ones not drawn yet (Hebrew, Arabic, Hindi, Vietnamese, `bahtText`, `dollarText`) show decimal. Word itself
  shows `bahtText` as decimal in a list.
- **Checking how a document looks:** headless LibreOffice on these Macs only sees its own bundled fonts (no Thai), so its
  renders are useless for Thai. Use `wordcraft-cli convert in.docx out.pdf` instead (it uses the installed fonts).
  To turn PDF pages into images without Poppler, a short Swift script with PDFKit works (`page.thumbnail(of:for:)`).
- **Observing Word from the command line:** Word's PDF export only writes inside its sandbox
  (`~/Library/Containers/com.microsoft.Word/Data/`), which macOS will not let other programs read. To read Word's list
  labels instead, use AppleScript: `list string of list format of (text object of paragraph i of document "x.docx")`
  (asking `paragraph i` directly returns `missing value`).
- No `node`, `pandoc` or `pdftoppm` on the second Mac; the system `python3` is too old for `match` statements.

## 8. Building for Windows

Chanon asked how to make a Windows version. Three ways; nothing has been tried yet.

1. **Cross-compile on the Mac** (quickest test): `rustup target add x86_64-pc-windows-gnu`, `brew install mingw-w64`, then
   `cargo build --release -p wordcraft --target x86_64-pc-windows-gnu` → `target/x86_64-pc-windows-gnu/release/wordcraft.exe`.
   Zip it with `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md` and copy it to the PC. It is unsigned, so Windows
   shows "Windows protected your PC" (More info → Run anyway). The original builds with Microsoft's toolchain on
   Windows, so this route is untested and may need fixes. x64 PCs only; an ARM PC needs another target.
2. **Build on the PC**: install Rust (rustup.rs) and Visual Studio Build Tools ("Desktop development with C++"), clone,
   `git checkout my-changes`, `cargo build --release -p wordcraft`. `packaging/windows/package.ps1` also makes a portable zip
   and an `.msi` (needs WiX v5; see the script's header). Needs the local commits pushed first, or the `repo` folder copied.
3. **GitHub Actions** (since 2026-10-24, the way releases are built): `.github/workflows/windows-release.yml` runs
   when a `v*` tag is pushed (or by hand from the Actions tab with a tag name). On GitHub's Windows runner it installs
   WiX 5.0.2, runs `packaging\windows\package.ps1 -Arch x64`, writes the SHA-256 of both files in the run's summary,
   and attaches `khiankrub-<version>-windows-x64.msi` and `-portable.zip` to the tag's release, making a **draft**
   release when there is none. The tag must be `v<version>` or `v<version>-something` for the version in
   `Cargo.toml`. Notes, the Mac zip and publishing stay manual. Pushing a tag needs Chanon's OK (rule 1). The
   original's `release.yml`, `freebsd.yml`, `windows-arm64.yml` and `packaging-lint.yml` were removed: they need
   storytold's secrets and shared workflows.

A public Windows release would also need the installer files renamed to KhianKrub (section 6), with a FORK-CHANGES entry.
The PC needs a Thai font for documents; Windows has Leelawadee UI and Tahoma, TH Sarabun New only if installed.

## 9. What is next (nothing is forced)

- Try the new app (`~/Applications/KhianKrub.app`), open `thai-test.docx` in it, and give feedback on the icon and name.
- Optional: a Windows test build (section 8).
- Optional: offer the Thai list formats in the list numbering dialog (`crates/ui-egui/src/dialogs_lists.rs`); today they
  only come from opened documents.
- Next public build: a new release (for example `v0.7.2`; set `version` in the root `Cargo.toml` to match) named like
  `KhianKrub-0.7.1-macos-apple-silicon.zip`; the browser pane may need a GitHub sign-in again (Claude in Chrome also works);
  ask before replacing or deleting an older release. The Thai Facebook post for thai.3 and its before/after picture are
  in `~/Documents/Codex/WordCraft/facebook/`; the release notes in `.../release/`.
- The old `~/Applications/WordCraft.app` on the first Mac can be moved to the Bin.
- A Thai interface is **not** wanted; wait for what Chanon raises next.

## 10. Notes that do not travel with the repo

Claude Code's own memory lives in `~/.claude/projects/.../memory/` on each Mac separately. The first Mac's memory holds
the same content as sections 4 and 5. The second Mac's session did not save any memory. On a new Mac, tell the session to
read this file and `FORK-CHANGES.md` first.
