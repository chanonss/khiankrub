# Handoff: continuing Khian Krub on another Mac

Written 2026-10-21 for Chanon Sarasuk. It lets a new Claude Code session (or you) pick up exactly where this one stopped.
Nothing here is secret: no passwords or tokens.

## 1. Where things stand

- **App:** Khian Krub (เขียนครับ), Chanon's fork of [storytold/wordcraft](https://github.com/storytold/wordcraft) for Thai users
  (Rust, egui). Fork: <https://github.com/chanonss/wordcraft>, branch **`my-changes`** (the default branch).
- **Done on this Mac, in order:** Thai Distribution and Thai word breaking, View only mode, English-only interface (22 other
  languages deleted), hover-jump fix, plain number fields, Thai space-dot centring, ArtCraft logos removed, **rename to
  Khian Krub, new gold ข icon on blue** (traced from Sarabun Bold, OFL). Every change is listed, newest first, in
  [`FORK-CHANGES.md`](FORK-CHANGES.md).
- **Published:** only release `v0.7.0-thai.2` (tag on commit `13f20be`). It still carries the old WordCraft name and icon.
- **NOT on GitHub yet:** the local commits after `a1538b9` (the last pushed commit): "Remove the 22 other interface languages",
  "Fix hover jumps, drag-to-change numbers, Thai space dots, and the alignment name", and the Khian Krub rename and icon.
  Check with `git log --oneline origin/my-changes..HEAD`. **A fresh clone on another Mac will not have them until they are
  pushed.** Chanon decides when to push (see rule 1 below). If you need to move now without pushing, copy the whole
  `repo` folder, or run `git bundle create ../khiankrub.bundle origin/my-changes..HEAD` here and
  `git pull ../khiankrub.bundle my-changes` there.
- **Untracked scratch files, not to be committed:** `issue-draft.md`, `thai-test.txt`.

## 2. Set up the other Mac

```bash
xcode-select --install                                   # compiler tools, if missing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust (rustup)
git clone https://github.com/chanonss/wordcraft.git && cd wordcraft && git checkout my-changes
cargo install resvg --locked                             # only needed to regenerate the icon files
```

- Apple silicon Mac for the app build (the packaged app is arm64 only, macOS 11 or later).
- First build takes several minutes. `cargo build --release -p wordcraft` is about 3 to 4 minutes.
- GitHub: Chanon signs in themself (never ask for or type a password). `git push` over HTTPS needs a GitHub login or token on
  that Mac; on this Mac `GIT_TERMINAL_PROMPT=0 git push origin my-changes` worked because credentials were already stored.
- Commit identity is passed per commit, not stored in git config:
  `git -c user.name="Chanon Sarasuk" -c user.email="chanon.s@rmutk.ac.th" commit ...`, with the message ending in the line
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.

## 3. Everyday commands

| Task | Command |
|---|---|
| Full check before every commit | `cargo xtask ci` (fmt, clippy -D warnings, tests, assets, layers, wasm) |
| Format | `cargo fmt --all` |
| Build the release binary | `cargo build --release -p wordcraft` |
| Make the Mac app | `packaging/macos/local-app.sh` → `~/Applications/Khian Krub.app` (ad-hoc signed, this Mac only) |
| Regenerate all icon files | `packaging/icons.sh` (needs `resvg`; uses `assets/app-icon/wordcraft.svg`) |

**Chain them so a failure stops everything.** Twice, a piped or chained command hid a failed check and an old build was
packaged. Use `set -euo pipefail`, no `| tail` on the checking command itself:

```bash
cargo xtask ci && cargo build --release -p wordcraft && packaging/macos/local-app.sh
```

The zip for a release is made with `ditto -c -k --keepParent --norsrc --noextattr "Khian Krub.app" <name>.zip`. Unzip a copy,
run `codesign --verify --strict --deep`, and put the zip's SHA-256 in the release notes.

## 4. Working rules Chanon has set (keep following them)

1. **Push only when told.** Make small changes, commit locally after each (check passes first), rebuild the local app so
   Chanon can try it, say how many commits are waiting, and push everything in one go when Chanon says so.
   Do **not** create, replace or delete a GitHub release without asking first ("tell me before you delete the old release").
2. **Update `FORK-CHANGES.md`** with a dated entry for every change.
3. **English-only interface.** Write new labels in English only (`tl!("...")`). Do not translate them, do not run or mention
   language tests, and do not ask again about a Thai menu: settled. Thai support in documents (Thai text, Thai
   Distribution, word breaking, Thai fonts) stays.
4. **Credit the original, never use its brand.** Keep `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, the README note and the About
   screen credit ("Based on WordCraft by the ArtCraft team. Fork by Chanon Sarasuk."). The original's brand licence forbids
   ArtCraft logos and marks in published forks: plain-text credit only (`docs/brand/` was deleted on purpose).
5. **Every asset file needs a row in `ATTRIBUTION.md`** (`cargo xtask assets` fails otherwise); new third-party material also
   needs a `NOTICE` entry. The app ships its licences inside (`local-app.sh` copies them into Contents/Resources).
6. **Never put a password, token or key into any page.** Chanon signs in to GitHub in the built-in browser themself; there is
   no `gh` CLI. Chanon attaches the release zip in the file picker.
7. Chanon writes in English. Replies in Thai only when Chanon asks. Facebook posts for the Thai group are written in Thai,
   without first-person pronouns.
8. Never use Microsoft, Adobe or similar assets (see `CLAUDE.md`). Never write panics in non-test code.

## 5. The icon and the name

- **Name:** Khian Krub in English letters for files and bundle ids, เขียนครับ as the Thai display name (Finder on a Thai Mac,
  via `th.lproj/InfoPlist.strings` written by `local-app.sh`). Bundle id `io.github.chanonss.khiankrub`, executable `KhianKrub`.
  "ครับ" is the male polite ending; Chanon knows and accepted it. A web search found no clash for "Khian Krub" (software names
  only, not trademark registers).
- **Icon:** the letter ข in gold `#F2B705` on a deep blue `#1E3A8A` rounded square (rx 112 in a 512 view box). The outline is
  traced from **Sarabun Bold** (Cadson Demak, SIL OFL 1.1, no reserved font name), chosen over TH Sarabun New (GPL) and
  Thonburi (Apple's). The path is stored in `assets/app-icon/wordcraft.svg`, nudged `translate(-11 -7)` so the visual weight is
  centred ("B2 balanced"). The licence text is `assets/app-icon/OFL-Sarabun.txt`. To redo the trace: get `Sarabun-Bold.ttf`
  from <https://github.com/cadsondemak/Sarabun>, read the outline of U+0E02 with CoreText (a short Swift script that
  walks `CTFontCreatePathForGlyph`), scale to 300 units tall inside the 512 box, and write the path as an even-odd fill.
- File names `wordcraft.svg`, `wordcraft.icns`, `wordcraft.ico`, `ai.storyteller.wordcraft.*` were **kept** so no code path
  changes.

## 6. Deliberately left as the original (do not "fix")

- Crate names `wordcraft-*`, the internal app id passed to `eframe::run_native("WordCraft", ...)`, icon file names.
- Settings folder `~/Library/Application Support/WordCraft` (existing settings survive the rename).
- Info.plist keys `WordCraftVersion` and `WordCraftBuildCommit`.
- `packaging/macos/package.sh` and the DMG background: the original's release tooling. `package.sh` only follows the new
  executable and icon names; the DMG still shows the old icon and name. Not used for Chanon's zip releases.
- The README's "Crafting Apps" table (upstream's sibling apps) and the sample document's link to getartcraft.com.
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
- Thai fonts in the interface: About text is English only, because it has not been checked that the egui UI fonts draw Thai.
- A release zip must be made with `ditto ... --norsrc --noextattr`; `codesign --strict` rejects extended attributes.

## 8. What is next (nothing is forced)

- Try the new app (`~/Applications/Khian Krub.app`) and give feedback on the icon and name. Then say "push" to upload the local commits.
- When Chanon wants a public build: new release (for example `v0.7.0-thai.3`) with the zip named for Khian Krub; ask before
  replacing or deleting `v0.7.0-thai.2`, and before pointing the Facebook post to it.
- The old `~/Applications/WordCraft.app` on the first Mac can be moved to the Bin.
- There is no other open work item. A Thai interface is **not** wanted; wait for what Chanon raises next.

## 9. Notes that do not travel with the repo

Claude Code's own memory lives in `~/.claude/projects/.../memory/` on the first Mac only. Its content is the same as sections 4
and 5 above: English-only menu (settled), the fork workflow, push-only-when-told, brand and licence rules, the chosen name.
On the other Mac a new session starts without it, so tell it to read this file and `FORK-CHANGES.md` first.
