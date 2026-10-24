<p align="center">
  <img src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.wordcraft.png" alt="KhianKrub icon: a gold Thai letter ข on a deep blue rounded square" width="96" height="96">
</p>

<h1 align="center">KhianKrub (เขียนครับ)</h1>

<p align="center">
  <b>A free, open-source word processor for Thai and English, for Mac and Windows.</b><br>
  โปรแกรมพิมพ์เอกสารฟรี โอเพนซอร์ส สำหรับภาษาไทยและภาษาอังกฤษ บน Mac และ Windows
</p>

<p align="center">
  <a href="https://github.com/chanonss/khiankrub/releases/latest"><b>⬇️ Download for Mac (Apple silicon)</b></a> ·
  <a href="https://github.com/chanonss/khiankrub/releases/latest"><b>⬇️ Download for Windows (64-bit)</b></a> ·
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

## Before and after

### The ribbon, sized and spaced like Word's

Larger icons, and the buttons centred in each group with even spacing, on every tab.

<p align="center">
  <img src="docs/images/khiankrub-ribbon-before-after.png" alt="The Home ribbon before, with small icons sitting at the top of each group, and after, with larger icons centred in their groups" width="100%">
</p>

### Bugs fixed

<table>
<tr>
<td width="35%" valign="top"><img src="docs/images/khiankrub-fix-navigation.png" alt="The Navigation pane before, listing Thai headings as question marks, and after, listing them in Thai" width="100%"><p align="center"><sub><b>Thai headings</b> in the Navigation pane showed as <code>?</code>; they now show in Thai.</sub></p></td>
<td width="65%" valign="top"><img src="docs/images/khiankrub-fix-space-dots.png" alt="Thai text with formatting marks on: before, the dot for each space floats near the top of the letters; after, it sits in the middle" width="100%"><p align="center"><sub><b>Space marks on Thai text</b> (with ¶ on) floated near the top of the letters; they now sit in the middle.</sub><br><br></p>
<img src="docs/images/khiankrub-fix-paragraph-dialog.png" alt="The Paragraph dialog's Alignment box: before it shows the raw text thaidistribute, after it shows Thai Distribution" width="70%"><p align="center"><sub><b>Paragraph dialog</b> showed the raw name <code>thaidistribute</code>; it now says Thai Distribution and keeps it.</sub></p></td>
</tr>
<tr>
<td colspan="2"><img src="docs/images/khiankrub-fix-recover.png" alt="File, Recover: before, a black page with a blank white button; after, the blue File sidebar with icons and readable buttons" width="100%"><p align="center"><sub><b>File › Recover</b> was black with blank white buttons in the light theme; it now follows the theme like the other File pages.</sub></p></td>
</tr>
</table>

## More of what was added

<table>
<tr>
<td width="62%" valign="top"><img src="docs/images/khiankrub-thai-word-breaking.png" alt="The full name of Bangkok, a long Thai word without spaces, wrapped onto a second line at a word boundary, and a long Thai paragraph wrapping between words" width="100%"><p align="center"><sub><b>Thai word breaking.</b> Lines wrap between Thai words, even in long text with no spaces at all.</sub></p>
<img src="docs/images/khiankrub-view-only.png" alt="The Editing menu open, offering Editing, Reviewing and View only" width="100%"><p align="center"><sub><b>View only</b> in the Editing menu: read a document without changing it by accident.</sub></p></td>
<td width="38%" valign="top"><img src="docs/images/khiankrub-shapes-menu.png" alt="Insert, Shapes menu with a small drawing beside each shape and connector name" width="100%"><p align="center"><sub><b>Insert › Shapes</b> shows a picture of each shape and connector.</sub></p></td>
</tr>
</table>

## Newest additions

### A real macOS menu bar

File, Edit, Insert, Format, View, Window and Help at the top of the screen, with the usual Mac shortcuts. Each item
runs the same command as its ribbon button or key.

<p align="center">
  <img src="docs/images/khiankrub-mac-menu-bar.png" alt="The macOS menu bar showing KhianKrub, File, Edit, Insert, Format, View, Window and Help, with the Format menu open: Font, Paragraph, Bullets and Numbering, Borders and Shading, Columns, Styles, Bold, Italic, Underline, font size, Clear Formatting and the alignments including Thai Distribution, each with its shortcut" width="60%">
</p>

### Page number at set distances from the paper's edges

Insert › Page Number › **Page Number Position…** places the number, for example, 1 inch from the top edge (to its
baseline) and 1 inch from the right edge, as theses often require. Top or bottom, left, centre or right, and no number
on the first page if wanted. Word opens the file with the number in the same place.

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/khiankrub-page-number-position.png" alt="The Page Number Position dialog: from the top edge, 1 inch to the baseline; right, 1 inch from the right edge; a check box for no number on the first page; and a small page preview with the number at the top right" width="100%"><p align="center"><sub>The dialog, with a preview of where the number goes.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/khiankrub-page-number-result.png" alt="The top right corner of a page at 150 percent zoom: the page number 1 with red arrows showing 1 inch from the top edge to its baseline and 1 inch from its right end to the right edge" width="100%"><p align="center"><sub>The result, measured: 1 in from the top edge and 1 in from the right edge.</sub></p></td>
</tr>
</table>

### Page breaks like Word's

Insert › Page Break (or ⌘Enter) moves the cursor to the next page, so what you type next goes there. With formatting
marks on, a page break shows as a dotted line with **Page Break** in the middle.

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/khiankrub-page-break.png" alt="Two pages side by side: page 1 has the word One and a dotted Page Break line; page 2 starts with Two, typed after the break, with the cursor" width="100%"><p align="center"><sub>"One", the break, then "Two" typed on page 2.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/khiankrub-break-label.png" alt="A Thai document with formatting marks on: a dotted line across the page with the words Page Break in the middle" width="100%"><p align="center"><sub>The "Page Break" mark, as in Word.</sub></p></td>
</tr>
</table>

### Hanging indent, as in Word

The Paragraph dialog has **Special: (none) / First line / Hanging** and **By**, instead of a negative first-line indent.

<p align="center">
  <img src="docs/images/khiankrub-hanging-indent.png" alt="The Paragraph dialog with Indentation Left 0.89 inches and Special set to Hanging, By 0.89 inches" width="50%">
</p>

### Pictures in the menus

The Layout tab's Text Direction, Orientation, Columns, Breaks and Line Numbers menus, and Insert's Table and Page
Number menus, show a small picture of each choice. The Insert Table grid is compact, as in Word.

<p align="center">
  <img src="docs/images/khiankrub-layout-menus.png" alt="Five Layout menus side by side, each choice with a small picture: Text Direction (Horizontal, Rotate 90 and 270 degrees), Orientation (Portrait, Landscape), Columns (One, Two, Three, Left, Right, More Columns), Breaks (Page, Column, Text Wrapping and the four section breaks) and Line Numbers (None, Continuous, Restart Each Page, Restart Each Section, options)" width="100%">
</p>
<p align="center">
  <img src="docs/images/khiankrub-insert-menus.png" alt="Insert, Table menu with a compact 10 by 8 grid and icons for Insert Table, Draw Table and Convert Text to Table; and Insert, Page Number menu with pictures for Top of Page, Bottom of Page, Page X of Y, Current Position, Format Page Numbers and Page Number Position" width="80%">
</p>

## Download and install

### Mac

1. Go to the [latest release](https://github.com/chanonss/khiankrub/releases/latest) and download
   **KhianKrub-…-macos-apple-silicon.zip** (under *Assets*).
2. Double-click the zip to unzip it, then drag **KhianKrub** into your **Applications** folder.
3. **First time only: right-click › Open.** The app is not signed by Apple, so macOS says it cannot verify the
   developer. Right-click (or Control-click) KhianKrub, choose **Open**, then click **Open** again. If the dialog
   only offers *Move to Trash*, open **System Settings › Privacy & Security** and click **Open Anyway** next to
   KhianKrub.

ดาวน์โหลดไฟล์ zip จากหน้า [Releases](https://github.com/chanonss/khiankrub/releases/latest) ดับเบิลคลิกเพื่อแตกไฟล์
แล้วลาก KhianKrub ไปไว้ในโฟลเดอร์ Applications ครั้งแรกที่เปิดให้คลิกขวาที่แอป เลือก Open แล้วกด Open อีกครั้ง

**Needs:** a Mac with Apple silicon (M1 or later) and macOS 11 or later. There is no Intel Mac build yet.

### Windows

1. Go to the [latest release](https://github.com/chanonss/khiankrub/releases/latest) and download
   **khiankrub-…-windows-x64.msi** (under *Assets*), then double-click it.
2. **First time only: More info › Run anyway.** The installer is not signed, so Windows says "Windows protected your
   PC". Click **More info**, then **Run anyway**, and click **Yes** when Windows asks for permission. KhianKrub is
   then in the Start menu.

To use it without installing, download **khiankrub-…-windows-x64-portable.zip** instead, unzip it and double-click
**KhianKrub.exe**.

ดาวน์โหลดไฟล์ .msi จากหน้า [Releases](https://github.com/chanonss/khiankrub/releases/latest) แล้วดับเบิลคลิกเพื่อติดตั้ง
ถ้าขึ้นหน้าต่าง "Windows protected your PC" ให้กด More info แล้วกด Run anyway

**Needs:** Windows 10 or 11, 64-bit (x64). There is no Windows on ARM or Linux build yet. The features are the same
as on the Mac, except the menu bar at the top of the screen, which is a macOS feature.

On both, Thai text looks best with a Thai font installed, such as TH Sarabun New.

## Build it yourself

KhianKrub is written in Rust. With [Rust](https://rustup.rs) installed:

```bash
git clone https://github.com/chanonss/khiankrub.git
cd khiankrub
cargo build --release -p wordcraft
packaging/macos/local-app.sh
```

The last step makes `~/Applications/KhianKrub.app`. On Windows (with the Visual Studio C++ Build Tools and
[WiX v5](https://wixtoolset.org)), `packaging\windows\package.ps1 -Arch x64` builds the installer and the portable zip
into `dist\release`. `cargo xtask ci` runs all the checks (it also needs
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
