<p align="center">
  <img src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.wordcraft.png" alt="KhianKrub icon: a gold Thai letter ข on a deep blue rounded square" width="96" height="96">
</p>

<h1 align="center">KhianKrub (เขียนครับ)</h1>

<p align="center">
  <b>A free, open-source word processor for Thai and English, for the Mac.</b><br>
  โปรแกรมพิมพ์เอกสารฟรี โอเพนซอร์ส สำหรับภาษาไทยและภาษาอังกฤษ บน Mac
</p>

<p align="center">
  <a href="https://github.com/chanonss/khiankrub/releases/latest"><b>⬇️ Download for Mac (Apple silicon)</b></a> ·
  <a href="FORK-CHANGES.md">What's new</a> ·
  <a href="#license-and-credits">Licence and credits</a>
</p>

<p align="center">
  <img src="docs/images/khiankrub-hero.png" alt="KhianKrub with a Thai test document open: the Home tab of the ribbon, the Navigation pane listing the document's Thai headings, and a page with Thai text in TH Sarabun New" width="100%">
</p>

KhianKrub is a fork of [WordCraft](https://github.com/storytold/wordcraft) by the ArtCraft team, adapted for Thai
users by Chanon Sarasuk. It opens, edits and saves Word documents (`.docx`), with a ribbon laid out like Microsoft
Word's, and adds what Thai writing needs. It is not affiliated with or endorsed by the ArtCraft team.

## Made for Thai

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/khiankrub-thai-distribution.png" alt="Two Thai paragraphs: ordinary Justify with large gaps between words, and Thai Distributed with the extra space spread evenly between letters" width="100%"><p align="center"><sub><b>Thai Distribution.</b> Justified Thai text with the extra space spread evenly, as in Word, instead of large gaps.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/khiankrub-thai-lists.png" alt="Lists numbered with bullets, Thai digits ๑ ๒ ๓ ๔ and Thai letters ก ข ค" width="100%"><p align="center"><sub><b>Thai list numbering.</b> Thai digits (๑ ๒ ๓), Thai letters (ก ข ค) and Thai counting words, matching Word.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/khiankrub-page-thumbnails.png" alt="The Navigation pane's Pages tab with a thumbnail of each page, the first outlined in blue" width="100%"><p align="center"><sub><b>Navigation.</b> Thai headings and page thumbnails; click one to go there.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/khiankrub-file-menu.png" alt="The File menu: a blue sidebar with an icon beside each entry, and template thumbnails" width="100%"><p align="center"><sub><b>File menu.</b> New, open, recover, save, print and export to PDF.</sub></p></td>
</tr>
</table>

- **Thai word breaking:** lines wrap between Thai words, even in long text without spaces.
- **Thai Distribution** alignment (Thai Distributed), on the Home tab next to Justify.
- **Thai list numbers, letters and counting words** in lists and page numbers.
- **Text boundaries:** a thin grey outline around the text area of each page (View › Show › Text Boundaries).
- **View only** mode in the Editing menu: read a document without changing it by accident.
- English menus; Thai text in documents, Thai fonts and Thai typing fully supported.

Everything that differs from the original, newest first, is in [FORK-CHANGES.md](FORK-CHANGES.md).

## Download and install

1. Go to the [latest release](https://github.com/chanonss/khiankrub/releases/latest) and download
   **KhianKrub-…-macos-apple-silicon.zip** (under *Assets*).
2. Double-click the zip to unzip it, then drag **KhianKrub** into your **Applications** folder.
3. **First time only: right-click › Open.** The app is not signed by Apple, so macOS says it cannot verify the
   developer. Right-click (or Control-click) KhianKrub, choose **Open**, then click **Open** again. If the dialog
   only offers *Move to Trash*, open **System Settings › Privacy & Security** and click **Open Anyway** next to
   KhianKrub.

ดาวน์โหลดไฟล์ zip จากหน้า [Releases](https://github.com/chanonss/khiankrub/releases/latest) ดับเบิลคลิกเพื่อแตกไฟล์
แล้วลาก KhianKrub ไปไว้ในโฟลเดอร์ Applications ครั้งแรกที่เปิดให้คลิกขวาที่แอป เลือก Open แล้วกด Open อีกครั้ง

**Needs:** a Mac with Apple silicon (M1 or later) and macOS 11 or later. There is no Intel, Windows or Linux build
yet. Thai text looks best with a Thai font installed, such as TH Sarabun New.

## Build it yourself

KhianKrub is written in Rust. With [Rust](https://rustup.rs) installed:

```bash
git clone https://github.com/chanonss/khiankrub.git
cd khiankrub
cargo build --release -p wordcraft
packaging/macos/local-app.sh
```

The last step makes `~/Applications/KhianKrub.app`. `cargo xtask ci` runs all the checks (it also needs
`rustup target add wasm32-unknown-unknown`). The crates keep the original `wordcraft-*` names. The original
project's README, with its full documentation (command line, automation over MCP, architecture), is at
[storytold/wordcraft](https://github.com/storytold/wordcraft#readme).

## License and credits

KhianKrub, like WordCraft, is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Copyright (c) 2026 ArtCraft Team and the WordCraft contributors; modifications copyright (c) 2026 Chanon Sarasuk.
Required notices are in [NOTICE](NOTICE).

**Based on [WordCraft](https://github.com/storytold/wordcraft) by the ArtCraft team.** Thank you to the ArtCraft
team and the WordCraft contributors, who wrote the program this fork builds on.

The icon's letter ข is traced from Sarabun Bold by Cadson Demak (SIL Open Font License 1.1). Bundled fonts, icons,
images and other assets keep their own open licenses; each one is listed with its author, source and license in
[ATTRIBUTION.md](ATTRIBUTION.md).

Thai word breaking uses [ICU4X](https://github.com/unicode-org/icu4x) by Unicode, Inc. (Unicode License v3);
its licence text and the list of its crates are in [NOTICE](NOTICE).

The English spelling dictionary and hyphenation come from Grady Ward's public-domain Moby
Hyphenator II word list. Polish spelling reads the Hunspell `pl_PL` dictionary installed on the system (not bundled) and
Polish hyphenation the TeX patterns by Hanna Kołodziejska, Bogusław Jackowski and Marek Ryćko (MIT
option). The sample documents and templates are original text written for WordCraft.

The ArtCraft name and logos are trademarks of the ArtCraft Team and are not covered by this license. This fork
does not include the ArtCraft logos: the original's brand license requires a modified version to remove them,
and allows saying in plain text that the work is based on WordCraft by the ArtCraft team. It is not an ArtCraft
product. The original's brand terms are at [LICENSE-brand.txt](https://github.com/storytold/wordcraft/blob/main/docs/brand/LICENSE-brand.txt).

<sub>Microsoft and Microsoft Word are trademarks of the Microsoft group of companies. KhianKrub is an independent,
open-source project and is not affiliated with, sponsored by or endorsed by Microsoft Corporation; the name is used
only to describe the workflows it is compatible with.</sub>
