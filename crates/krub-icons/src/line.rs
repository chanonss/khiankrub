//! The line set: KhianKrub's redrawn ribbon icons (original artwork on a 24-unit grid, thin
//! outlines with a pale gold surface), shared by the three KrubOffice apps. `paint` draws a
//! name it knows and returns `true`; an unknown name returns `false` so the caller can fall
//! back to its older drawing. Contributor-original, `MIT OR Apache-2.0` (AGENTS.md §1).

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, vec2};

struct Pen<'a> {
    painter: &'a Painter,
    origin: Pos2,
    scale: f32,
    ink: Color32,
    accent: Color32,
}

impl Pen<'_> {
    fn point(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.scale
    }
    fn stroke(&self) -> Stroke {
        Stroke::new((1.45 * self.scale).max(0.85), self.ink)
    }
    fn line(&self, points: &[(f32, f32)]) {
        let path: Vec<Pos2> = points.iter().map(|&(x, y)| self.point(x, y)).collect();
        self.painter.add(Shape::line(path, self.stroke()));
        for &(x, y) in points.first().into_iter().chain(points.last()) {
            self.painter.circle_filled(self.point(x, y), self.stroke().width / 2.0, self.ink);
        }
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32) {
        self.painter.rect_stroke(Rect::from_min_size(self.point(x, y), vec2(w, h) * self.scale), 1.2 * self.scale, self.stroke(), StrokeKind::Middle);
    }
    fn fill(&self, x: f32, y: f32, w: f32, h: f32) {
        self.painter.rect_filled(Rect::from_min_size(self.point(x, y), vec2(w, h) * self.scale), 1.0 * self.scale, self.accent);
    }
    fn text(&self, x: f32, y: f32, size: f32, text: &str) {
        self.painter.text(self.point(x, y), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(size * self.scale), self.ink);
    }
    fn dot(&self, x: f32, y: f32, radius: f32) {
        self.painter.circle_filled(self.point(x, y), radius * self.scale, self.ink);
    }
    fn circle(&self, x: f32, y: f32, radius: f32) {
        self.painter.circle_stroke(self.point(x, y), radius * self.scale, self.stroke());
    }
}

pub(crate) mod deck;
pub(crate) mod sheet;

/// Draw line-set icon `name` in `r`; `false` when the set has no such name.
pub fn paint(p: &Painter, r: Rect, name: &str, ink: Color32, disabled: bool) -> bool {
    paint_ex(p, r, name, ink, disabled, false)
}

/// [`paint`] with `flat`: the gold surface in ink too, for icons on a coloured band.
// One flat table of drawings, one arm per name: long, but each arm is simple.
#[allow(clippy::cognitive_complexity)]
pub fn paint_ex(p: &Painter, r: Rect, name: &str, ink: Color32, disabled: bool, flat: bool) -> bool {
    if !matches!(
        name,
        "paste"
            | "optionsGeneral"
            | "optionsDisplay"
            | "optionsProofing"
            | "optionsSave"
            | "optionsLanguage"
            | "optionsAccessibility"
            | "optionsAdvanced"
            | "picCorrections"
            | "picColor"
            | "picTransparency"
            | "picChange"
            | "picReset"
            | "picStyles"
            | "picBorder"
            | "crop"
            | "altText"
            | "selectTable"
            | "properties"
            | "deleteTable"
            | "insertAbove"
            | "insertBelow"
            | "insertLeft"
            | "insertRight"
            | "merge"
            | "splitCells"
            | "splitTable"
            | "autofit"
            | "distributeRows"
            | "distributeCols"
            | "cellMargins"
            | "repeatHeader"
            | "toText"
            | "formula"
            | "cellAlign_topLeft"
            | "cellAlign_topCenter"
            | "cellAlign_topRight"
            | "cellAlign_centerLeft"
            | "cellAlign_center"
            | "cellAlign_centerRight"
            | "cellAlign_bottomLeft"
            | "cellAlign_bottomCenter"
            | "cellAlign_bottomRight"
            | "sel_cell"
            | "sel_column"
            | "sel_row"
            | "sel_table"
            | "del_cell"
            | "del_column"
            | "del_row"
            | "del_table"
            | "fit_contents"
            | "fit_window"
            | "fit_fixed"
            | "spelling"
            | "thesaurus"
            | "wordCount"
            | "readAloud"
            | "accessibility"
            | "translate"
            | "language"
            | "deleteComment"
            | "prevComment"
            | "nextComment"
            | "showComments"
            | "trackChanges"
            | "markup"
            | "reviewingPane"
            | "accept"
            | "reject"
            | "prevChange"
            | "nextChange"
            | "compare"
            | "blockAuthors"
            | "restrict"
            | "hideInk"
            | "toc"
            | "addText"
            | "updateTable"
            | "footnote"
            | "endnote"
            | "nextFootnote"
            | "showNotes"
            | "researcher"
            | "citation"
            | "sources"
            | "styles"
            | "bibliography"
            | "caption"
            | "tableOfFigures"
            | "tableOfTables"
            | "update"
            | "markEntry"
            | "index"
            | "markCitation"
            | "tableOfAuthorities"
            | "themes"
            | "colors"
            | "fonts"
            | "paraSpacing"
            | "paraSpacing_default"
            | "paraSpacing_none"
            | "paraSpacing_compact"
            | "paraSpacing_tight"
            | "paraSpacing_open"
            | "paraSpacing_relaxed"
            | "paraSpacing_double"
            | "effectsDesign"
            | "setDefault"
            | "watermark"
            | "pageColor"
            | "pageBorders"
            | "textDirection"
            | "margins"
            | "orientation"
            | "size"
            | "columns"
            | "breaks"
            | "lineNumbers"
            | "hyphenation"
            | "position"
            | "wrapText"
            | "bringForward"
            | "sendBackward"
            | "selectionPane"
            | "align"
            | "group"
            | "rotate"
            | "dirHorizontal"
            | "dirDown"
            | "dirUp"
            | "portrait"
            | "landscape"
            | "cols_one"
            | "cols_two"
            | "cols_three"
            | "cols_left"
            | "cols_right"
            | "colsMore"
            | "mg_normal"
            | "mg_narrow"
            | "mg_moderate"
            | "mg_wide"
            | "mg_mirrored"
            | "mg_office2003"
            | "mg_custom"
            | "brk_page"
            | "brk_column"
            | "brk_textWrapping"
            | "brk_nextPage"
            | "brk_continuous"
            | "brk_evenPage"
            | "brk_oddPage"
            | "ln_none"
            | "ln_continuous"
            | "ln_restartPage"
            | "ln_restartSection"
            | "lnOptions"
            | "readMode"
            | "printLayout"
            | "outline"
            | "draft"
            | "webLayout"
            | "focus"
            | "immersive"
            | "vertical"
            | "sideToSide"
            | "ruler"
            | "hideWhiteSpace"
            | "gridlines"
            | "textBoundaries"
            | "navPane"
            | "printPreview"
            | "zoom"
            | "zoomOut"
            | "zoom100"
            | "zoomIn"
            | "onePage"
            | "multiplePages"
            | "pageWidth"
            | "darkMode"
            | "interfaceTheme"
            | "newWindow"
            | "arrangeAll"
            | "split"
            | "fullScreen"
            | "macros"
            | "documentTemplate"
            | "lasso"
            | "eraser"
            | "pen"
            | "pencil"
            | "addPen"
            | "thickness"
            | "inkToShape"
            | "inkToMath"
            | "canvas"
            | "replay"
            | "inkColor"
            | "coverPage"
            | "blankPage"
            | "pageBreak"
            | "table"
            | "picture"
            | "shapes"
            | "icons"
            | "smartArt"
            | "chart"
            | "link"
            | "crossRef"
            | "newComment"
            | "header"
            | "footer"
            | "pageNumber"
            | "quickParts"
            | "wordArt"
            | "dropCap"
            | "signature"
            | "dateTime"
            | "object"
            | "equation"
            | "symbol"
            | "textToTable"
            | "pn_top"
            | "pn_bottom"
            | "pn_xofy"
            | "pn_current"
            | "pn_format"
            | "pn_position"
            | "textBox"
            | "bookmark"
            | "drawTable"
            | "cut"
            | "copy"
            | "painter"
            | "grow"
            | "shrink"
            | "case"
            | "clear"
            | "phonetic"
            | "bold"
            | "italic"
            | "underline"
            | "strike"
            | "subscript"
            | "superscript"
            | "charborder"
            | "enclose"
            | "effects"
            | "highlight"
            | "fontcolor"
            | "bullets"
            | "numbering"
            | "multilevel"
            | "outdent"
            | "indent"
            | "alignLeft"
            | "alignCenter"
            | "alignRight"
            | "justify"
            | "textLtr"
            | "textRtl"
            | "sort"
            | "pilcrow"
            | "lineSpacing"
            | "shading"
            | "borders"
            | "find"
            | "replace"
            | "select"
            | "dictate"
            | "editor"
    ) {
        return false;
    }
    let size = r.width().min(r.height());
    if !size.is_finite() || size <= 0.5 || !r.center().is_finite() {
        return true;
    }
    let scale = size / 24.0;
    let accent = if disabled || flat || ink == Color32::WHITE { ink.gamma_multiply(0.35) } else { crate::gold(ink, crate::GoldRole::Surface, 1.0) };
    let pen = Pen { painter: p, origin: r.center() - vec2(size, size) / 2.0, scale, ink, accent };
    match name {
        "paste" => {
            pen.line(&[(7.0, 5.0), (4.5, 5.0), (4.5, 21.0), (18.5, 21.0), (18.5, 18.0)]);
            pen.fill(8.0, 2.5, 7.0, 4.0);
            pen.rect(8.0, 2.5, 7.0, 4.0);
            pen.rect(10.0, 9.0, 10.0, 8.0);
            pen.line(&[(12.0, 11.5), (18.0, 11.5)]);
            pen.line(&[(12.0, 14.0), (16.5, 14.0)]);
        }
        "cut" => {
            pen.circle(6.7, 17.2, 2.6);
            pen.circle(17.3, 17.2, 2.6);
            pen.line(&[(8.5, 15.4), (18.6, 4.0)]);
            pen.line(&[(15.5, 15.4), (5.4, 4.0)]);
            pen.painter.circle_filled(pen.point(12.0, 11.5), 1.05 * scale, ink);
        }
        "copy" => {
            pen.line(&[(6.5, 16.5), (4.0, 16.5), (4.0, 3.5), (14.5, 3.5), (14.5, 6.0)]);
            pen.fill(9.0, 8.0, 10.5, 12.5);
            pen.rect(9.0, 8.0, 10.5, 12.5);
            pen.line(&[(11.0, 11.0), (17.5, 11.0)]);
            pen.line(&[(11.0, 14.0), (17.5, 14.0)]);
            pen.line(&[(11.0, 17.0), (15.5, 17.0)]);
        }
        "painter" => {
            pen.fill(4.5, 3.5, 14.0, 5.0);
            pen.rect(4.5, 3.5, 14.0, 5.0);
            pen.line(&[(18.5, 6.0), (20.0, 6.0), (20.0, 11.0), (11.5, 11.0), (11.5, 13.0)]);
            pen.rect(9.7, 13.0, 3.6, 7.5);
        }
        "grow" | "shrink" => {
            pen.line(&[(3.0, 19.0), (8.0, 5.0), (13.0, 19.0)]);
            pen.line(&[(4.8, 14.0), (11.2, 14.0)]);
            pen.line(&[(15.0, 8.0), (21.0, 8.0)]);
            if name == "grow" {
                pen.line(&[(18.0, 5.0), (18.0, 11.0)]);
            }
        }
        "case" => {
            pen.line(&[(2.0, 18.5), (7.0, 5.0), (12.0, 18.5)]);
            pen.line(&[(4.0, 13.5), (10.0, 13.5)]);
            pen.circle(17.0, 15.0, 3.5);
            pen.line(&[(20.5, 11.5), (20.5, 18.5)]);
        }
        "clear" => {
            pen.line(&[(2.5, 15.0), (7.0, 4.0), (11.0, 14.0)]);
            pen.line(&[(4.0, 11.0), (9.5, 11.0)]);
            pen.fill(12.5, 12.0, 7.0, 5.0);
            pen.line(&[(10.5, 16.0), (15.0, 10.5), (21.0, 15.0), (16.5, 20.0), (13.0, 20.0), (10.5, 17.5), (10.5, 16.0)]);
            pen.line(&[(15.5, 20.0), (21.0, 20.0)]);
        }
        "phonetic" => {
            pen.line(&[(5.0, 20.0), (10.0, 9.0), (15.0, 20.0)]);
            pen.line(&[(7.0, 16.0), (13.0, 16.0)]);
            pen.text(17.0, 6.0, 8.0, "a");
            pen.line(&[(4.0, 5.0), (10.0, 5.0)]);
        }
        "bold" => {
            pen.line(&[(7.0, 4.0), (7.0, 20.0), (13.0, 20.0), (16.0, 19.0), (17.0, 17.0), (17.0, 15.0), (16.0, 13.0), (13.0, 12.0), (7.0, 12.0)]);
            pen.line(&[(7.0, 4.0), (13.0, 4.0), (15.5, 5.0), (16.5, 7.0), (16.0, 10.0), (13.0, 12.0)]);
            pen.line(&[(8.7, 5.0), (8.7, 19.0)]);
        }
        "italic" => {
            pen.line(&[(11.0, 4.0), (19.0, 4.0)]);
            pen.line(&[(15.0, 4.0), (9.0, 20.0)]);
            pen.line(&[(5.0, 20.0), (13.0, 20.0)]);
        }
        "underline" => {
            pen.line(&[(6.0, 4.0), (6.0, 13.0), (7.0, 16.0), (9.0, 17.5), (13.0, 17.5), (15.0, 16.0), (16.0, 13.0), (16.0, 4.0)]);
            pen.line(&[(4.0, 21.0), (19.0, 21.0)]);
        }
        "strike" => {
            pen.line(&[
                (16.5, 6.0),
                (14.5, 4.5),
                (9.0, 4.5),
                (7.0, 6.5),
                (7.0, 9.0),
                (9.0, 11.0),
                (14.0, 13.0),
                (16.0, 15.0),
                (16.0, 17.0),
                (14.0, 19.5),
                (8.0, 19.5),
                (6.0, 18.0),
            ]);
            pen.line(&[(3.0, 12.0), (21.0, 12.0)]);
        }
        "subscript" | "superscript" => {
            pen.line(&[(3.0, 8.0), (12.0, 18.0)]);
            pen.line(&[(12.0, 8.0), (3.0, 18.0)]);
            let y = if name == "subscript" { 19.0 } else { 8.0 };
            pen.text(17.0, y, 9.0, "2");
        }
        "charborder" | "enclose" | "effects" | "fontcolor" => {
            if name == "charborder" {
                pen.rect(3.0, 3.0, 18.0, 18.0);
            }
            if name == "enclose" {
                pen.circle(12.0, 12.0, 9.0);
            }
            if name == "effects" {
                pen.fill(6.0, 14.0, 12.0, 4.0);
            }
            let (top, bottom) = if name == "fontcolor" { (4.0, 17.0) } else { (5.5, 18.0) };
            pen.line(&[(6.0, bottom), (12.0, top), (18.0, bottom)]);
            pen.line(&[(8.5, 13.5), (15.5, 13.5)]);
            if name == "fontcolor" {
                pen.fill(4.0, 19.0, 16.0, 2.0);
            }
        }
        "highlight" => {
            pen.fill(3.0, 19.0, 18.0, 2.0);
            pen.line(&[(5.0, 16.0), (14.0, 6.0), (18.0, 10.0), (9.0, 19.0), (5.0, 19.0), (5.0, 16.0)]);
            pen.line(&[(12.5, 7.5), (16.5, 11.5)]);
        }
        "bullets" | "numbering" | "multilevel" => {
            for (i, y) in [6.0, 12.0, 18.0].into_iter().enumerate() {
                if name == "bullets" {
                    pen.dot(4.0, y, 1.0);
                } else if name == "numbering" {
                    pen.text(4.0, y, 6.0, ["1", "2", "3"][i]);
                } else {
                    pen.dot(if i == 0 { 4.0 } else { 7.0 }, y, 1.0);
                }
                let start = if name == "multilevel" && i > 0 { 11.0 } else { 9.0 };
                pen.line(&[(start, y), (21.0, y)]);
            }
        }
        "indent" | "outdent" => {
            for y in [5.0, 9.0, 15.0, 19.0] {
                pen.line(&[(9.0, y), (21.0, y)]);
            }
            // A larger solid arrowhead keeps the opposing directions readable at 20 px.
            let (tail, tip, direction) = if name == "indent" { (2.0, 9.0, 1.0) } else { (9.0, 2.0, -1.0) };
            pen.line(&[(tail, 12.0), (tip - direction * 2.5, 12.0)]);
            pen.painter.add(Shape::convex_polygon(
                vec![pen.point(tip, 12.0), pen.point(tip - direction * 4.0, 8.8), pen.point(tip - direction * 4.0, 15.2)],
                pen.ink,
                Stroke::NONE,
            ));
        }
        "alignLeft" | "alignCenter" | "alignRight" | "justify" => {
            for (i, y) in [5.0, 9.5, 14.0, 18.5].into_iter().enumerate() {
                let short = i % 2 == 1 && name != "justify";
                let (a, b) = if !short {
                    (3.5, 20.5)
                } else if name == "alignRight" {
                    (8.0, 20.5)
                } else if name == "alignCenter" {
                    (5.8, 18.2)
                } else {
                    (3.5, 16.0)
                };
                pen.line(&[(a, y), (b, y)]);
            }
        }
        "textLtr" | "textRtl" => {
            let rtl = name == "textRtl";
            let (a, b) = if rtl { (20.0, 13.0) } else { (4.0, 11.0) };
            pen.line(&[(a, 6.0), (b, 6.0)]);
            pen.line(&[(b - (b - a).signum() * 2.0, 4.0), (b, 6.0), (b - (b - a).signum() * 2.0, 8.0)]);
            pen.text(12.0, 15.0, 14.0, "¶");
        }
        "sort" => {
            pen.text(7.0, 7.0, 9.0, "A");
            pen.text(7.0, 17.0, 9.0, "Z");
            pen.line(&[(17.0, 4.0), (17.0, 19.0), (14.5, 16.5)]);
            pen.line(&[(17.0, 19.0), (19.5, 16.5)]);
        }
        "pilcrow" => {
            pen.fill(6.0, 4.0, 6.0, 8.0);
            pen.line(&[(12.0, 4.0), (12.0, 20.0)]);
            pen.line(&[(16.0, 4.0), (16.0, 20.0)]);
            pen.line(&[(10.0, 4.0), (18.0, 4.0)]);
            pen.line(&[(10.0, 4.0), (7.0, 5.0), (6.0, 8.0), (7.0, 11.0), (10.0, 12.0), (12.0, 12.0)]);
        }
        "lineSpacing" => {
            for y in [5.0, 9.5, 14.0, 18.5] {
                pen.line(&[(10.0, y), (21.0, y)]);
            }
            pen.line(&[(5.0, 4.0), (5.0, 20.0)]);
            pen.line(&[(3.0, 6.0), (5.0, 4.0), (7.0, 6.0)]);
            pen.line(&[(3.0, 18.0), (5.0, 20.0), (7.0, 18.0)]);
        }
        "shading" => {
            pen.fill(3.0, 19.0, 18.0, 2.0);
            pen.line(&[(5.0, 11.0), (11.0, 5.0), (18.0, 12.0), (12.0, 18.0), (5.0, 11.0)]);
            pen.line(&[(7.0, 4.0), (12.0, 9.0)]);
            pen.circle(19.0, 17.0, 1.2);
        }
        "borders" => {
            pen.rect(4.0, 4.0, 16.0, 16.0);
            pen.line(&[(12.0, 4.0), (12.0, 20.0)]);
            pen.line(&[(4.0, 12.0), (20.0, 12.0)]);
            pen.fill(4.0, 18.0, 16.0, 2.0);
        }
        "find" | "replace" => {
            pen.circle(10.0, 10.0, 5.5);
            pen.line(&[(14.0, 14.0), (19.5, 19.5)]);
            if name == "replace" {
                pen.line(&[(15.0, 4.0), (20.0, 4.0), (20.0, 8.0)]);
                pen.line(&[(18.0, 6.0), (20.0, 8.0), (22.0, 6.0)]);
            }
        }
        "select" => {
            pen.fill(5.0, 3.5, 8.0, 12.0);
            pen.line(&[(5.0, 3.0), (5.0, 18.0), (8.5, 14.5), (11.5, 21.0), (14.0, 20.0), (11.0, 13.5), (16.0, 13.5), (5.0, 3.0)]);
        }
        "pen" | "pencil" | "addPen" => {
            pen.fill(9.0, 7.0, 8.0, 6.0);
            pen.line(&[(4.0, 20.0), (5.5, 14.5), (15.5, 4.5), (19.5, 8.5), (9.5, 18.5), (4.0, 20.0)]);
            pen.line(&[(5.5, 14.5), (9.5, 18.5)]);
            if name == "pencil" {
                pen.line(&[(13.5, 6.5), (17.5, 10.5)]);
            }
            if name == "pen" {
                pen.line(&[(7.0, 15.0), (15.0, 7.0)]);
            }
            if name == "addPen" {
                pen.line(&[(16.0, 17.5), (22.0, 17.5)]);
                pen.line(&[(19.0, 14.5), (19.0, 20.5)]);
            }
        }
        "optionsGeneral" => {
            for (y, x) in [(6.0, 9.0), (12.0, 15.0), (18.0, 8.0)] {
                pen.line(&[(4.0, y), (20.0, y)]);
                pen.fill(x - 1.5, y - 2.0, 3.0, 4.0);
                pen.rect(x - 1.5, y - 2.0, 3.0, 4.0);
            }
        }
        "optionsDisplay" => {
            pen.rect(3.0, 4.0, 18.0, 13.0);
            pen.fill(5.0, 6.0, 14.0, 9.0);
            pen.line(&[(12.0, 17.0), (12.0, 20.0)]);
            pen.line(&[(8.0, 20.0), (16.0, 20.0)]);
        }
        "optionsProofing" => {
            pen.text(10.0, 8.0, 11.0, "abc");
            pen.line(&[(6.0, 17.0), (9.0, 20.0), (18.0, 11.0)]);
        }
        "optionsSave" => {
            pen.line(&[(4.0, 3.0), (17.0, 3.0), (20.0, 6.0), (20.0, 21.0), (4.0, 21.0), (4.0, 3.0)]);
            pen.fill(7.0, 4.0, 9.0, 5.0);
            pen.rect(7.0, 4.0, 9.0, 5.0);
            pen.rect(7.0, 13.0, 10.0, 7.0);
        }
        "optionsLanguage" => {
            pen.circle(12.0, 12.0, 8.5);
            pen.line(&[(3.5, 12.0), (20.5, 12.0)]);
            pen.line(&[(12.0, 3.5), (12.0, 20.5)]);
            pen.line(&[(6.0, 7.0), (18.0, 7.0)]);
            pen.line(&[(6.0, 17.0), (18.0, 17.0)]);
        }
        "optionsAccessibility" => {
            pen.circle(12.0, 4.5, 2.0);
            pen.line(&[(4.0, 9.0), (12.0, 8.0), (20.0, 9.0)]);
            pen.line(&[(12.0, 8.0), (12.0, 15.0)]);
            pen.line(&[(8.0, 21.0), (12.0, 15.0), (16.0, 21.0)]);
        }
        "optionsAdvanced" => {
            pen.rect(4.0, 4.0, 16.0, 16.0);
            pen.fill(6.0, 6.0, 12.0, 4.0);
            pen.line(&[(7.0, 13.0), (17.0, 13.0)]);
            pen.circle(10.0, 13.0, 1.5);
            pen.line(&[(7.0, 17.0), (17.0, 17.0)]);
            pen.circle(14.0, 17.0, 1.5);
        }
        "picCorrections" | "picColor" | "picTransparency" | "picChange" | "picReset" | "picStyles" | "picBorder" => {
            pen.rect(3.0, 4.0, 15.0, 14.0);
            pen.circle(13.0, 8.0, 1.2);
            pen.line(&[(4.0, 16.0), (8.0, 11.0), (11.0, 14.0), (13.0, 12.0), (17.0, 16.0)]);
            match name {
                "picCorrections" => {
                    pen.circle(19.0, 5.0, 2.0);
                    pen.line(&[(19.0, 1.5), (19.0, 2.0)]);
                    pen.line(&[(22.0, 5.0), (22.5, 5.0)]);
                }
                "picColor" => {
                    pen.fill(5.0, 14.0, 11.0, 2.0);
                    pen.dot(19.0, 8.0, 1.4);
                }
                "picTransparency" => {
                    pen.line(&[(6.0, 7.0), (8.0, 7.0)]);
                    pen.line(&[(6.0, 9.0), (8.0, 9.0)]);
                    pen.line(&[(10.0, 9.0), (12.0, 9.0)]);
                }
                "picChange" => {
                    pen.line(&[(13.0, 20.0), (21.0, 20.0)]);
                    pen.line(&[(19.0, 18.0), (21.0, 20.0), (19.0, 22.0)]);
                }
                "picReset" => {
                    pen.line(&[(14.0, 21.0), (19.0, 21.0), (21.0, 19.0), (21.0, 16.0), (19.0, 14.0)]);
                    pen.line(&[(19.0, 17.0), (19.0, 14.0), (22.0, 14.0)]);
                }
                "picStyles" => {
                    pen.fill(5.0, 18.5, 13.0, 2.0);
                    pen.line(&[(6.0, 21.0), (19.0, 21.0)]);
                }
                "picBorder" => {
                    pen.rect(1.5, 2.5, 18.0, 17.0);
                }
                _ => {}
            }
        }
        "crop" => {
            pen.line(&[(6.0, 3.0), (6.0, 18.0), (21.0, 18.0)]);
            pen.line(&[(3.0, 6.0), (18.0, 6.0), (18.0, 21.0)]);
        }
        "altText" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.text(12.0, 10.0, 10.0, "A");
            pen.line(&[(6.0, 16.0), (18.0, 16.0)]);
            pen.line(&[(6.0, 19.0), (15.0, 19.0)]);
        }
        "selectTable" | "properties" | "deleteTable" | "insertAbove" | "insertBelow" | "insertLeft" | "insertRight" | "merge" | "splitCells"
        | "splitTable" | "autofit" | "distributeRows" | "distributeCols" | "cellMargins" | "repeatHeader" | "toText" | "formula" => {
            pen.rect(3.0, 4.0, 15.0, 15.0);
            pen.line(&[(3.0, 9.0), (18.0, 9.0)]);
            pen.line(&[(3.0, 14.0), (18.0, 14.0)]);
            pen.line(&[(8.0, 4.0), (8.0, 19.0)]);
            pen.line(&[(13.0, 4.0), (13.0, 19.0)]);
            match name {
                "selectTable" | "repeatHeader" => pen.fill(4.0, 5.0, 13.0, 3.0),
                "deleteTable" => {
                    pen.line(&[(14.0, 15.0), (21.0, 22.0)]);
                    pen.line(&[(21.0, 15.0), (14.0, 22.0)]);
                }
                "insertAbove" | "insertBelow" => {
                    let y = if name == "insertAbove" { 2.0 } else { 22.0 };
                    pen.line(&[(9.0, y), (15.0, y)]);
                    pen.line(&[(12.0, y - 1.5), (12.0, y + 1.5)]);
                }
                "insertLeft" | "insertRight" => {
                    let x = if name == "insertLeft" { 1.5 } else { 21.0 };
                    pen.line(&[(x, 9.0), (x, 15.0)]);
                    pen.line(&[(x - 1.5, 12.0), (x + 1.5, 12.0)]);
                }
                "merge" => {
                    pen.fill(9.0, 10.0, 8.0, 3.0);
                    pen.line(&[(9.0, 11.5), (16.0, 11.5)]);
                }
                "splitCells" => pen.line(&[(15.5, 10.0), (15.5, 13.0)]),
                "splitTable" => pen.line(&[(4.0, 11.5), (17.0, 11.5)]),
                "autofit" | "distributeCols" => {
                    pen.line(&[(5.0, 21.0), (16.0, 21.0)]);
                    pen.line(&[(6.5, 19.5), (5.0, 21.0), (6.5, 22.5)]);
                    pen.line(&[(14.5, 19.5), (16.0, 21.0), (14.5, 22.5)]);
                }
                "distributeRows" => {
                    pen.line(&[(21.0, 6.0), (21.0, 17.0)]);
                    pen.line(&[(19.5, 7.5), (21.0, 6.0), (22.5, 7.5)]);
                    pen.line(&[(19.5, 15.5), (21.0, 17.0), (22.5, 15.5)]);
                }
                "cellMargins" => pen.rect(9.5, 10.0, 2.0, 3.0),
                "properties" => {
                    pen.circle(18.0, 18.0, 3.0);
                    pen.dot(18.0, 18.0, 0.8);
                }
                "toText" => {
                    pen.line(&[(19.0, 7.0), (22.0, 7.0)]);
                    pen.line(&[(19.0, 10.0), (22.0, 10.0)]);
                }
                "formula" => pen.text(19.0, 18.0, 8.0, "∑"),
                _ => {}
            }
        }
        "cellAlign_topLeft"
        | "cellAlign_topCenter"
        | "cellAlign_topRight"
        | "cellAlign_centerLeft"
        | "cellAlign_center"
        | "cellAlign_centerRight"
        | "cellAlign_bottomLeft"
        | "cellAlign_bottomCenter"
        | "cellAlign_bottomRight" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            let y = if name.contains("top") {
                6.5
            } else if name.contains("bottom") {
                15.0
            } else {
                10.5
            };
            let x = if name.ends_with("Left") {
                5.5
            } else if name.ends_with("Right") {
                10.5
            } else {
                8.0
            };
            pen.line(&[(x, y), (x + 8.0, y)]);
            pen.line(&[(x, y + 3.0), (x + 6.0, y + 3.0)]);
        }
        "sel_cell" | "sel_column" | "sel_row" | "sel_table" | "del_cell" | "del_column" | "del_row" | "del_table" => {
            let area = name.split_once('_').map(|(_, v)| v).unwrap_or("");
            let (x, y, w, h) = match area {
                "cell" => (9.0, 9.0, 5.0, 5.0),
                "column" => (9.0, 4.0, 5.0, 15.0),
                "row" => (4.0, 9.0, 15.0, 5.0),
                _ => (4.0, 4.0, 15.0, 15.0),
            };
            pen.fill(x, y, w, h);
            pen.rect(4.0, 4.0, 15.0, 15.0);
            for v in [9.0, 14.0] {
                pen.line(&[(v, 4.0), (v, 19.0)]);
                pen.line(&[(4.0, v), (19.0, v)]);
            }
            if name.starts_with("del_") {
                pen.line(&[(15.0, 15.0), (21.0, 21.0)]);
                pen.line(&[(21.0, 15.0), (15.0, 21.0)]);
            }
        }
        "fit_contents" | "fit_window" | "fit_fixed" => {
            pen.rect(3.0, 4.0, 18.0, 15.0);
            pen.line(&[(9.0, 4.0), (9.0, 19.0)]);
            pen.line(&[(15.0, 4.0), (15.0, 19.0)]);
            pen.line(&[(3.0, 11.5), (21.0, 11.5)]);
            if name == "fit_fixed" {
                pen.line(&[(9.5, 21.0), (14.5, 21.0)]);
            } else {
                let (a, b) = if name == "fit_window" { (4.0, 20.0) } else { (9.0, 15.0) };
                pen.line(&[(a, 21.0), (b, 21.0)]);
                pen.line(&[(a + 1.5, 19.5), (a, 21.0), (a + 1.5, 22.5)]);
                pen.line(&[(b - 1.5, 19.5), (b, 21.0), (b - 1.5, 22.5)]);
            }
        }
        "spelling" => {
            pen.text(10.0, 8.0, 11.0, "abc");
            pen.line(&[(6.0, 17.0), (9.0, 20.0), (18.0, 11.0)]);
        }
        "thesaurus" => {
            pen.fill(5.0, 4.0, 7.0, 14.0);
            pen.line(&[
                (12.0, 7.0),
                (9.0, 5.0),
                (4.0, 5.0),
                (4.0, 18.0),
                (9.0, 18.0),
                (12.0, 20.0),
                (15.0, 18.0),
                (20.0, 18.0),
                (20.0, 5.0),
                (15.0, 5.0),
                (12.0, 7.0),
                (12.0, 20.0),
            ]);
            pen.text(8.0, 11.0, 7.0, "A");
            pen.text(16.0, 11.0, 7.0, "B");
        }
        "wordCount" => {
            pen.line(&[(3.0, 5.0), (13.0, 5.0)]);
            pen.line(&[(3.0, 9.0), (13.0, 9.0)]);
            pen.line(&[(3.0, 13.0), (11.0, 13.0)]);
            pen.text(16.0, 17.0, 8.0, "123");
        }
        "readAloud" => {
            pen.fill(3.0, 9.0, 5.0, 6.0);
            pen.line(&[(3.0, 9.0), (7.0, 9.0), (12.0, 5.0), (12.0, 19.0), (7.0, 15.0), (3.0, 15.0), (3.0, 9.0)]);
            pen.line(&[(15.0, 9.0), (17.0, 12.0), (15.0, 15.0)]);
            pen.line(&[(18.0, 6.0), (21.0, 9.0), (21.0, 15.0), (18.0, 18.0)]);
        }
        "accessibility" => {
            pen.circle(12.0, 4.5, 2.0);
            pen.line(&[(4.0, 9.0), (12.0, 8.0), (20.0, 9.0)]);
            pen.line(&[(12.0, 8.0), (12.0, 15.0)]);
            pen.line(&[(8.0, 21.0), (12.0, 15.0), (16.0, 21.0)]);
        }
        "translate" | "language" => {
            pen.circle(12.0, 12.0, 8.5);
            pen.line(&[(3.5, 12.0), (20.5, 12.0)]);
            pen.line(&[(12.0, 3.5), (12.0, 20.5)]);
            pen.line(&[(6.0, 7.0), (18.0, 7.0)]);
            pen.line(&[(6.0, 17.0), (18.0, 17.0)]);
            if name == "translate" {
                pen.fill(8.0, 9.0, 8.0, 6.0);
                pen.text(12.0, 12.0, 7.0, "A");
            }
        }
        "deleteComment" | "prevComment" | "nextComment" | "showComments" => {
            pen.line(&[(3.0, 4.0), (19.0, 4.0), (19.0, 15.0), (10.0, 15.0), (6.0, 19.0), (6.0, 15.0), (3.0, 15.0), (3.0, 4.0)]);
            if name == "deleteComment" {
                pen.line(&[(8.0, 7.0), (14.0, 12.0)]);
                pen.line(&[(14.0, 7.0), (8.0, 12.0)]);
            } else if name == "showComments" {
                pen.fill(6.0, 6.0, 10.0, 6.0);
                pen.line(&[(7.0, 8.0), (15.0, 8.0)]);
                pen.line(&[(7.0, 11.0), (13.0, 11.0)]);
            } else {
                let (a, b) = if name == "prevComment" { (16.0, 7.0) } else { (7.0, 16.0) };
                pen.line(&[(a, 9.5), (b, 9.5)]);
                let d = (b - a).signum();
                pen.line(&[(b - 2.0 * d, 7.5), (b, 9.5), (b - 2.0 * d, 11.5)]);
            }
        }
        "trackChanges" | "markup" | "reviewingPane" => {
            pen.rect(3.0, 3.0, 17.0, 18.0);
            pen.line(&[(7.0, 7.0), (16.0, 7.0)]);
            pen.line(&[(7.0, 10.0), (16.0, 10.0)]);
            pen.line(&[(7.0, 13.0), (14.0, 13.0)]);
            if name == "reviewingPane" {
                pen.fill(4.0, 4.0, 3.0, 16.0);
                pen.line(&[(8.0, 3.0), (8.0, 21.0)]);
            } else {
                pen.fill(5.0, 16.0, 12.0, 2.0);
                pen.line(&[(5.0, 16.5), (17.0, 16.5)]);
            }
        }
        "accept" | "reject" => {
            pen.rect(4.0, 3.0, 15.0, 18.0);
            pen.line(&[(7.0, 7.0), (16.0, 7.0)]);
            pen.line(&[(7.0, 10.0), (14.0, 10.0)]);
            if name == "accept" {
                pen.line(&[(8.0, 16.0), (11.0, 19.0), (19.0, 12.0)]);
            } else {
                pen.line(&[(9.0, 13.0), (17.0, 20.0)]);
                pen.line(&[(17.0, 13.0), (9.0, 20.0)]);
            }
        }
        "prevChange" | "nextChange" => {
            pen.rect(4.0, 3.0, 13.0, 18.0);
            pen.line(&[(7.0, 7.0), (14.0, 7.0)]);
            pen.line(&[(7.0, 10.0), (14.0, 10.0)]);
            let (a, b) = if name == "prevChange" { (21.0, 15.0) } else { (15.0, 21.0) };
            pen.line(&[(a, 16.0), (b, 16.0)]);
            let d = (b - a).signum();
            pen.line(&[(b - 2.0 * d, 14.0), (b, 16.0), (b - 2.0 * d, 18.0)]);
        }
        "compare" => {
            pen.rect(3.0, 4.0, 8.0, 14.0);
            pen.fill(4.0, 5.0, 6.0, 3.0);
            pen.rect(13.0, 6.0, 8.0, 14.0);
            pen.line(&[(5.0, 11.0), (9.0, 11.0)]);
            pen.line(&[(15.0, 13.0), (19.0, 13.0)]);
            pen.line(&[(15.0, 16.0), (19.0, 16.0)]);
        }
        "blockAuthors" | "restrict" => {
            pen.rect(3.0, 3.0, 13.0, 17.0);
            pen.line(&[(6.0, 7.0), (13.0, 7.0)]);
            pen.line(&[(6.0, 10.0), (12.0, 10.0)]);
            pen.fill(13.0, 14.0, 8.0, 6.0);
            pen.rect(13.0, 14.0, 8.0, 6.0);
            pen.line(&[(15.0, 14.0), (15.0, 11.0), (17.0, 9.0), (19.0, 11.0), (19.0, 14.0)]);
            if name == "blockAuthors" {
                pen.dot(17.0, 17.0, 0.9);
            }
        }
        "hideInk" => {
            pen.line(&[(4.0, 18.0), (7.0, 13.0), (10.0, 16.0), (14.0, 9.0), (18.0, 12.0)]);
            pen.line(&[(4.0, 4.0), (20.0, 20.0)]);
        }
        "toc" | "index" | "bibliography" | "tableOfAuthorities" | "sources" => {
            pen.rect(4.0, 2.5, 16.0, 19.0);
            if name == "sources" {
                pen.fill(6.0, 4.0, 12.0, 4.0);
            }
            for (i, y) in [7.0, 10.5, 14.0, 17.5].into_iter().enumerate() {
                let left = if name == "toc" && i == 1 { 9.0 } else { 7.0 };
                pen.line(&[(left, y), (if name == "bibliography" { 17.0 } else { 14.5 }, y)]);
                if name != "bibliography" {
                    pen.dot(17.0, y, 0.6);
                }
            }
            if name == "tableOfAuthorities" {
                pen.line(&[(6.5, 5.0), (17.5, 5.0)]);
            }
        }
        "addText" => {
            pen.line(&[(4.0, 5.0), (14.0, 5.0)]);
            pen.line(&[(4.0, 9.0), (14.0, 9.0)]);
            pen.line(&[(4.0, 13.0), (12.0, 13.0)]);
            pen.line(&[(15.0, 16.0), (22.0, 16.0)]);
            pen.line(&[(18.5, 12.5), (18.5, 19.5)]);
        }
        "updateTable" | "update" => {
            pen.rect(3.0, 3.0, 13.0, 15.0);
            pen.line(&[(6.0, 7.0), (13.0, 7.0)]);
            pen.line(&[(6.0, 10.0), (12.0, 10.0)]);
            pen.line(&[(13.0, 16.0), (15.0, 13.0), (19.0, 13.0), (21.0, 15.0), (21.0, 18.0), (19.0, 20.0), (15.0, 20.0)]);
            pen.line(&[(18.5, 15.0), (21.0, 15.0), (21.0, 12.5)]);
        }
        "footnote" | "endnote" | "nextFootnote" | "showNotes" => {
            pen.rect(4.0, 2.5, 16.0, 19.0);
            pen.line(&[(7.0, 6.0), (16.0, 6.0)]);
            pen.line(&[(7.0, 9.0), (14.0, 9.0)]);
            pen.fill(6.0, 16.0, 12.0, 3.0);
            pen.line(&[(7.0, 14.0), (17.0, 14.0)]);
            if name == "endnote" {
                pen.text(8.0, 17.5, 6.0, "i");
            } else {
                pen.text(8.0, 17.5, 6.0, "1");
            }
            if name == "nextFootnote" {
                pen.line(&[(15.0, 11.0), (20.0, 11.0)]);
                pen.line(&[(18.0, 9.0), (20.0, 11.0), (18.0, 13.0)]);
            }
            if name == "showNotes" {
                pen.circle(15.0, 17.0, 2.5);
            }
        }
        "researcher" => {
            pen.rect(4.0, 3.0, 12.0, 16.0);
            pen.line(&[(6.5, 7.0), (13.0, 7.0)]);
            pen.line(&[(6.5, 10.0), (12.0, 10.0)]);
            pen.circle(15.5, 15.0, 4.0);
            pen.line(&[(18.5, 18.0), (21.0, 20.5)]);
        }
        "citation" | "markCitation" => {
            pen.fill(4.0, 4.0, 16.0, 15.0);
            pen.rect(4.0, 3.0, 16.0, 18.0);
            pen.text(12.0, 11.0, 13.0, "“ ”");
            if name == "markCitation" {
                pen.line(&[(8.0, 18.0), (16.0, 18.0)]);
            }
        }
        "styles" => {
            pen.line(&[(3.5, 19.0), (8.5, 4.0), (13.5, 19.0)]);
            pen.line(&[(5.5, 14.0), (11.5, 14.0)]);
            pen.line(&[(15.0, 8.0), (21.0, 8.0)]);
            pen.line(&[(15.0, 12.0), (21.0, 12.0)]);
            pen.line(&[(15.0, 16.0), (19.0, 16.0)]);
        }
        "caption" => {
            pen.rect(3.0, 3.0, 18.0, 12.0);
            pen.fill(5.0, 11.0, 14.0, 2.0);
            pen.line(&[(5.0, 12.0), (9.0, 7.0), (12.0, 10.0), (14.0, 8.0), (19.0, 13.0)]);
            pen.line(&[(4.0, 19.0), (20.0, 19.0)]);
        }
        "tableOfFigures" | "tableOfTables" => {
            pen.rect(4.0, 2.5, 16.0, 19.0);
            if name == "tableOfTables" {
                pen.fill(6.5, 5.0, 5.0, 4.0);
                pen.rect(6.5, 5.0, 5.0, 4.0);
                pen.line(&[(9.0, 5.0), (9.0, 9.0)]);
                pen.line(&[(6.5, 7.0), (11.5, 7.0)]);
            } else {
                pen.rect(6.5, 5.0, 5.0, 4.0);
                pen.line(&[(7.0, 8.5), (8.5, 6.5), (11.0, 8.5)]);
            }
            pen.line(&[(13.0, 6.5), (17.5, 6.5)]);
            pen.line(&[(6.5, 12.0), (17.5, 12.0)]);
            pen.line(&[(6.5, 15.0), (17.5, 15.0)]);
            pen.line(&[(6.5, 18.0), (15.0, 18.0)]);
        }
        "markEntry" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            pen.line(&[(7.0, 7.0), (17.0, 7.0)]);
            pen.fill(7.0, 10.0, 8.0, 3.0);
            pen.line(&[(7.0, 11.5), (15.0, 11.5)]);
            pen.line(&[(7.0, 16.0), (15.0, 16.0)]);
        }
        "themes" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.fill(5.0, 5.0, 14.0, 5.0);
            pen.line(&[(5.0, 10.0), (19.0, 10.0)]);
            pen.line(&[(5.5, 14.0), (18.5, 14.0)]);
            pen.line(&[(5.5, 17.0), (15.5, 17.0)]);
        }
        "colors" => {
            pen.circle(11.0, 11.0, 8.0);
            pen.fill(8.0, 8.0, 6.0, 6.0);
            pen.dot(8.0, 6.5, 1.0);
            pen.dot(13.5, 6.5, 1.0);
            pen.dot(6.5, 11.5, 1.0);
            pen.dot(11.0, 15.5, 1.0);
            pen.line(&[(16.0, 16.0), (20.0, 20.0)]);
        }
        "fonts" => {
            pen.line(&[(3.0, 19.0), (8.0, 5.0), (13.0, 19.0)]);
            pen.line(&[(5.0, 14.0), (11.0, 14.0)]);
            pen.circle(17.0, 15.0, 3.0);
            pen.line(&[(20.0, 12.0), (20.0, 18.0)]);
        }
        "paraSpacing_default"
        | "paraSpacing_none"
        | "paraSpacing_compact"
        | "paraSpacing_tight"
        | "paraSpacing_open"
        | "paraSpacing_relaxed"
        | "paraSpacing_double" => {
            // Two paragraphs, each with two lines. Both the line and paragraph gaps
            // reflect the preset; the pair of shorter lines marks paragraph endings.
            let rows = match name {
                "paraSpacing_none" => [5.0, 8.0, 11.0, 14.0],
                "paraSpacing_compact" => [4.0, 7.0, 12.0, 15.0],
                "paraSpacing_tight" => [3.5, 7.0, 12.5, 16.0],
                "paraSpacing_open" => [2.0, 5.5, 15.5, 19.0],
                "paraSpacing_relaxed" => [2.5, 7.0, 13.0, 17.5],
                "paraSpacing_double" => [2.0, 7.0, 14.0, 19.0],
                _ => [3.0, 6.5, 13.5, 17.0],
            };
            for (i, y) in rows.into_iter().enumerate() {
                pen.line(&[(3.0, y), (if i % 2 == 0 { 21.0 } else { 16.0 }, y)]);
            }
        }
        "paraSpacing" => {
            for y in [5.0, 8.5, 15.5, 19.0] {
                pen.line(&[(10.0, y), (21.0, y)]);
            }
            pen.line(&[(5.0, 4.0), (5.0, 20.0)]);
            pen.line(&[(3.0, 6.0), (5.0, 4.0), (7.0, 6.0)]);
            pen.line(&[(3.0, 18.0), (5.0, 20.0), (7.0, 18.0)]);
        }
        "effectsDesign" => {
            pen.fill(5.0, 7.0, 10.0, 10.0);
            pen.rect(5.0, 7.0, 10.0, 10.0);
            pen.line(&[(18.0, 3.0), (18.0, 8.0)]);
            pen.line(&[(15.5, 5.5), (20.5, 5.5)]);
            pen.line(&[(18.0, 13.0), (18.0, 16.0)]);
            pen.line(&[(16.5, 14.5), (19.5, 14.5)]);
        }
        "setDefault" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            pen.fill(6.0, 5.0, 12.0, 4.0);
            pen.line(&[(7.0, 13.0), (10.0, 16.0), (17.0, 9.0)]);
        }
        "watermark" => {
            pen.rect(4.0, 2.5, 16.0, 19.0);
            for y in [7.0, 10.0, 13.0, 16.0] {
                pen.line(&[(6.5, y), (17.5, y)]);
            }
            pen.fill(6.0, 10.0, 12.0, 4.0);
            pen.line(&[(5.0, 18.0), (19.0, 5.0)]);
        }
        "pageColor" => {
            pen.fill(6.0, 4.0, 12.0, 15.0);
            pen.rect(4.0, 2.5, 16.0, 19.0);
            pen.line(&[(7.0, 7.0), (17.0, 7.0)]);
            pen.line(&[(7.0, 10.0), (17.0, 10.0)]);
        }
        "pageBorders" => {
            pen.rect(3.0, 2.5, 18.0, 19.0);
            pen.rect(6.0, 5.5, 12.0, 13.0);
            pen.line(&[(8.0, 9.0), (16.0, 9.0)]);
            pen.line(&[(8.0, 12.0), (16.0, 12.0)]);
            pen.line(&[(8.0, 15.0), (14.0, 15.0)]);
        }
        "textDirection" | "dirHorizontal" | "dirDown" | "dirUp" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            if name == "dirDown" || name == "dirUp" {
                for x in [8.0, 11.0, 14.0] {
                    pen.line(&[(x, 6.0), (x, 16.0)]);
                }
                let (a, b) = if name == "dirDown" { (6.0, 17.0) } else { (17.0, 6.0) };
                pen.line(&[(17.0, a), (17.0, b)]);
                let d = (b - a).signum();
                pen.line(&[(15.0, b - 2.0 * d), (17.0, b), (19.0, b - 2.0 * d)]);
            } else {
                for y in [7.0, 10.0, 13.0] {
                    pen.line(&[(7.0, y), (17.0, y)]);
                }
                pen.line(&[(7.0, 17.0), (17.0, 17.0)]);
                pen.line(&[(15.0, 15.0), (17.0, 17.0), (15.0, 19.0)]);
            }
        }
        "margins" | "mg_normal" | "mg_narrow" | "mg_moderate" | "mg_wide" | "mg_mirrored" | "mg_office2003" | "mg_custom" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            let (l, r, t, b) = match name {
                "mg_narrow" => (5.5, 18.5, 5.0, 19.0),
                "mg_moderate" => (6.0, 18.0, 6.5, 17.5),
                "mg_wide" => (8.0, 16.0, 6.5, 17.5),
                "mg_mirrored" => (8.0, 18.0, 6.0, 18.0),
                "mg_office2003" => (7.5, 16.5, 6.0, 18.0),
                _ => (7.0, 17.0, 6.0, 18.0),
            };
            pen.fill(l, t, r - l, b - t);
            pen.line(&[(l, t), (r, t)]);
            pen.line(&[(l, b), (r, b)]);
            pen.line(&[(l, t), (l, b)]);
            pen.line(&[(r, t), (r, b)]);
            if name == "mg_custom" {
                pen.dot(9.0, 12.0, 0.7);
                pen.dot(12.0, 12.0, 0.7);
                pen.dot(15.0, 12.0, 0.7);
            }
        }
        "orientation" | "portrait" | "landscape" => {
            if name == "landscape" {
                pen.rect(2.5, 6.0, 19.0, 12.0);
                pen.fill(4.0, 7.5, 16.0, 2.0);
                for y in [12.0, 15.0] {
                    pen.line(&[(5.0, y), (19.0, y)]);
                }
            } else {
                pen.rect(6.0, 2.5, 12.0, 19.0);
                pen.fill(7.5, 4.0, 9.0, 2.0);
                for y in [9.0, 12.0, 15.0, 18.0] {
                    pen.line(&[(8.0, y), (16.0, y)]);
                }
            }
        }
        "size" => {
            pen.rect(5.0, 3.0, 14.0, 18.0);
            pen.line(&[(2.0, 4.0), (2.0, 20.0)]);
            pen.line(&[(1.0, 6.0), (2.0, 4.0), (3.0, 6.0)]);
            pen.line(&[(1.0, 18.0), (2.0, 20.0), (3.0, 18.0)]);
            pen.line(&[(7.0, 11.0), (17.0, 11.0)]);
            pen.line(&[(9.0, 9.0), (7.0, 11.0), (9.0, 13.0)]);
            pen.line(&[(15.0, 9.0), (17.0, 11.0), (15.0, 13.0)]);
        }
        "columns" | "cols_one" | "cols_two" | "cols_three" | "cols_left" | "cols_right" | "colsMore" => {
            pen.rect(3.5, 3.0, 17.0, 18.0);
            let spans: &[(f32, f32)] = match name {
                "cols_one" => &[(6.0, 18.0)],
                "cols_three" => &[(5.5, 8.5), (10.5, 13.5), (15.5, 18.5)],
                "cols_left" => &[(5.5, 8.5), (11.0, 18.5)],
                "cols_right" => &[(5.5, 13.0), (15.5, 18.5)],
                _ => &[(5.5, 11.0), (13.0, 18.5)],
            };
            for &(a, b) in spans {
                for y in [6.5, 9.5, 12.5, 15.5, 18.5] {
                    pen.line(&[(a, y), (b, y)]);
                }
            }
            if name == "colsMore" {
                pen.dot(10.0, 19.0, 0.7);
                pen.dot(12.0, 19.0, 0.7);
                pen.dot(14.0, 19.0, 0.7);
            }
        }
        "breaks" | "brk_page" | "brk_column" | "brk_textWrapping" | "brk_nextPage" | "brk_continuous" | "brk_evenPage" | "brk_oddPage" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            for y in [6.0, 9.0, 15.0, 18.0] {
                pen.line(&[(7.0, y), (17.0, y)]);
            }
            pen.line(&[(5.0, 12.0), (19.0, 12.0)]);
            if matches!(name, "brk_nextPage" | "brk_continuous" | "brk_evenPage" | "brk_oddPage") {
                pen.line(&[(5.0, 13.5), (19.0, 13.5)]);
            }
            if name == "brk_evenPage" {
                pen.text(18.0, 17.0, 6.5, "2");
            } else if name == "brk_oddPage" {
                pen.text(18.0, 17.0, 6.5, "1");
            }
            if name == "brk_column" {
                pen.line(&[(12.0, 4.0), (12.0, 20.0)]);
            }
            if name == "brk_textWrapping" {
                pen.line(&[(16.0, 8.0), (16.0, 10.0), (12.0, 10.0)]);
            }
            if name == "brk_nextPage" {
                pen.line(&[(21.5, 15.0), (21.5, 20.0)]);
            }
        }
        "lineNumbers" | "ln_none" | "ln_continuous" | "ln_restartPage" | "ln_restartSection" | "lnOptions" => {
            pen.rect(4.0, 3.0, 16.0, 18.0);
            for (i, y) in [6.0, 9.0, 12.0, 15.0, 18.0].into_iter().enumerate() {
                pen.line(&[(10.0, y), (18.0, y)]);
                if name != "ln_none" {
                    let n = if matches!(name, "ln_restartPage" | "ln_restartSection") && i >= 3 { i - 2 } else { i + 1 };
                    pen.text(7.0, y, 5.5, &n.to_string());
                }
            }
            if name == "ln_restartPage" || name == "ln_restartSection" {
                pen.line(&[(5.0, 13.5), (19.0, 13.5)]);
            }
        }
        "hyphenation" => {
            pen.text(8.0, 8.0, 9.0, "ab");
            pen.line(&[(13.0, 9.0), (18.0, 9.0)]);
            pen.text(13.0, 16.0, 9.0, "cd");
        }
        "position" | "wrapText" => {
            pen.rect(3.5, 3.0, 17.0, 18.0);
            for y in [6.0, 9.0, 12.0, 15.0, 18.0] {
                pen.line(&[(6.0, y), (18.0, y)]);
            }
            pen.fill(10.0, 8.0, 7.0, 7.0);
            pen.rect(10.0, 8.0, 7.0, 7.0);
        }
        "bringForward" | "sendBackward" => {
            pen.fill(4.0, 4.0, 9.0, 9.0);
            pen.rect(4.0, 4.0, 9.0, 9.0);
            pen.rect(9.0, 9.0, 9.0, 9.0);
            let (a, b) = if name == "bringForward" { (18.0, 5.0) } else { (5.0, 18.0) };
            pen.line(&[(21.0, a), (21.0, b)]);
            let d = (b - a).signum();
            pen.line(&[(19.0, b - 2.0 * d), (21.0, b), (23.0, b - 2.0 * d)]);
        }
        "selectionPane" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(10.0, 3.0), (10.0, 21.0)]);
            pen.fill(4.0, 4.0, 5.0, 16.0);
            for y in [7.0, 11.0, 15.0] {
                pen.line(&[(13.0, y), (18.0, y)]);
            }
        }
        "align" => {
            pen.line(&[(4.0, 4.0), (4.0, 20.0)]);
            pen.fill(7.0, 6.0, 11.0, 4.0);
            pen.rect(7.0, 6.0, 11.0, 4.0);
            pen.rect(7.0, 14.0, 8.0, 4.0);
        }
        "group" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.fill(6.0, 6.0, 6.0, 6.0);
            pen.rect(6.0, 6.0, 6.0, 6.0);
            pen.circle(16.0, 16.0, 3.0);
        }
        "rotate" => {
            pen.line(&[(6.0, 8.0), (8.0, 5.0), (12.0, 4.0), (16.0, 5.0), (19.0, 8.0)]);
            pen.line(&[(16.0, 8.0), (19.0, 8.0), (19.0, 5.0)]);
            pen.fill(8.0, 11.0, 8.0, 7.0);
            pen.rect(8.0, 11.0, 8.0, 7.0);
        }
        "readMode" | "immersive" => {
            pen.fill(5.0, 5.0, 6.0, 12.0);
            pen.line(&[
                (12.0, 7.0),
                (9.0, 5.0),
                (4.0, 5.0),
                (4.0, 18.0),
                (9.0, 18.0),
                (12.0, 20.0),
                (15.0, 18.0),
                (20.0, 18.0),
                (20.0, 5.0),
                (15.0, 5.0),
                (12.0, 7.0),
                (12.0, 20.0),
            ]);
            if name == "immersive" {
                pen.line(&[(6.0, 9.0), (9.0, 9.0)]);
                pen.line(&[(15.0, 9.0), (18.0, 9.0)]);
            }
        }
        "printLayout" | "outline" | "draft" | "webLayout" | "onePage" | "pageWidth" | "documentTemplate" | "newWindow" => {
            pen.line(&[(5.0, 2.5), (15.0, 2.5), (19.0, 6.5), (19.0, 21.5), (5.0, 21.5), (5.0, 2.5)]);
            pen.line(&[(15.0, 2.5), (15.0, 6.5), (19.0, 6.5)]);
            match name {
                "printLayout" => {
                    pen.fill(7.0, 7.5, 10.0, 2.5);
                    pen.line(&[(7.0, 12.0), (17.0, 12.0)]);
                    pen.line(&[(7.0, 15.0), (17.0, 15.0)]);
                }
                "outline" => {
                    for y in [8.0, 12.0, 16.0] {
                        pen.dot(7.5, y, 0.7);
                        pen.line(&[(10.0, y), (16.5, y)]);
                    }
                }
                "draft" => {
                    for y in [8.0, 11.0, 14.0, 17.0] {
                        pen.line(&[(7.0, y), (16.5, y)]);
                    }
                }
                "webLayout" => {
                    pen.circle(12.0, 13.0, 4.5);
                    pen.line(&[(7.5, 13.0), (16.5, 13.0)]);
                    pen.line(&[(12.0, 8.5), (12.0, 17.5)]);
                }
                "pageWidth" => {
                    pen.line(&[(3.0, 13.0), (21.0, 13.0)]);
                    pen.line(&[(5.0, 11.0), (3.0, 13.0), (5.0, 15.0)]);
                    pen.line(&[(19.0, 11.0), (21.0, 13.0), (19.0, 15.0)]);
                }
                "documentTemplate" => {
                    pen.fill(7.0, 8.0, 10.0, 4.0);
                    pen.line(&[(7.0, 15.0), (17.0, 15.0)]);
                    pen.line(&[(7.0, 18.0), (15.0, 18.0)]);
                }
                "newWindow" => {
                    pen.line(&[(10.0, 13.0), (18.0, 13.0)]);
                    pen.line(&[(14.0, 9.0), (14.0, 17.0)]);
                }
                _ => {}
            }
        }
        "focus" => {
            for (x, y, dx, dy) in [(3.0, 3.0, 1.0, 1.0), (21.0, 3.0, -1.0, 1.0), (3.0, 21.0, 1.0, -1.0), (21.0, 21.0, -1.0, -1.0)] {
                pen.line(&[(x + 5.0 * dx, y), (x, y), (x, y + 5.0 * dy)]);
            }
            pen.fill(8.0, 8.0, 8.0, 8.0);
        }
        "vertical" | "sideToSide" => {
            pen.rect(7.0, 6.0, 10.0, 12.0);
            if name == "vertical" {
                pen.line(&[(12.0, 2.0), (12.0, 5.0)]);
                pen.line(&[(10.0, 4.0), (12.0, 2.0), (14.0, 4.0)]);
                pen.line(&[(12.0, 19.0), (12.0, 22.0)]);
                pen.line(&[(10.0, 20.0), (12.0, 22.0), (14.0, 20.0)]);
            } else {
                pen.line(&[(2.0, 12.0), (6.0, 12.0)]);
                pen.line(&[(4.0, 10.0), (2.0, 12.0), (4.0, 14.0)]);
                pen.line(&[(18.0, 12.0), (22.0, 12.0)]);
                pen.line(&[(20.0, 10.0), (22.0, 12.0), (20.0, 14.0)]);
            }
        }
        "ruler" => {
            pen.fill(3.0, 8.0, 18.0, 8.0);
            pen.rect(3.0, 8.0, 18.0, 8.0);
            for x in [6.0, 9.0, 12.0, 15.0, 18.0] {
                pen.line(&[(x, 8.0), (x, if x == 12.0 { 12.0 } else { 10.5 })]);
            }
        }
        "hideWhiteSpace" => {
            pen.line(&[(5.0, 3.0), (19.0, 3.0), (19.0, 9.0), (5.0, 9.0), (5.0, 3.0)]);
            pen.line(&[(5.0, 15.0), (19.0, 15.0), (19.0, 21.0), (5.0, 21.0), (5.0, 15.0)]);
            pen.line(&[(3.0, 10.0), (3.0, 14.0)]);
            pen.line(&[(1.5, 11.5), (3.0, 10.0), (4.5, 11.5)]);
            pen.line(&[(1.5, 12.5), (3.0, 14.0), (4.5, 12.5)]);
        }
        "gridlines" => {
            pen.rect(3.5, 3.5, 17.0, 17.0);
            for n in [9.2, 14.8] {
                pen.line(&[(n, 3.5), (n, 20.5)]);
                pen.line(&[(3.5, n), (20.5, n)]);
            }
        }
        "textBoundaries" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(6.0, 8.0), (18.0, 8.0)]);
            pen.line(&[(6.0, 12.0), (18.0, 12.0)]);
            pen.line(&[(6.0, 16.0), (14.0, 16.0)]);
        }
        "navPane" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.fill(4.0, 4.0, 5.0, 16.0);
            pen.line(&[(9.0, 3.0), (9.0, 21.0)]);
            pen.line(&[(12.0, 8.0), (18.0, 8.0)]);
            pen.line(&[(12.0, 12.0), (18.0, 12.0)]);
            pen.line(&[(12.0, 16.0), (16.0, 16.0)]);
        }
        "printPreview" => {
            pen.line(&[(4.0, 3.0), (13.0, 3.0), (16.0, 6.0), (16.0, 9.0)]);
            pen.line(&[(4.0, 3.0), (4.0, 20.0), (12.0, 20.0)]);
            pen.circle(14.5, 14.0, 4.0);
            pen.line(&[(17.5, 17.0), (21.0, 20.5)]);
        }
        "zoom" | "zoomOut" | "zoomIn" | "zoom100" => {
            pen.circle(10.0, 10.0, 6.0);
            pen.line(&[(14.5, 14.5), (20.0, 20.0)]);
            if name == "zoom100" {
                pen.text(10.0, 10.0, 6.0, "1:1");
            } else if name != "zoom" {
                pen.line(&[(7.0, 10.0), (13.0, 10.0)]);
                if name == "zoomIn" {
                    pen.line(&[(10.0, 7.0), (10.0, 13.0)]);
                }
            } else {
                pen.fill(7.0, 7.0, 6.0, 6.0);
            }
        }
        "multiplePages" | "arrangeAll" => {
            pen.fill(3.0, 4.0, 8.0, 11.0);
            pen.rect(3.0, 4.0, 8.0, 11.0);
            pen.rect(13.0, 7.0, 8.0, 11.0);
            if name == "arrangeAll" {
                pen.line(&[(5.0, 17.0), (11.0, 17.0)]);
                pen.line(&[(13.0, 4.0), (19.0, 4.0)]);
            }
        }
        "darkMode" | "interfaceTheme" => {
            pen.circle(12.0, 12.0, 8.5);
            pen.fill(5.0, 6.0, 6.0, 12.0);
            pen.line(&[(12.0, 3.5), (12.0, 20.5)]);
            if name == "interfaceTheme" {
                pen.circle(12.0, 12.0, 3.0);
            }
        }
        "split" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(3.0, 12.0), (21.0, 12.0)]);
            pen.fill(5.0, 5.0, 14.0, 5.0);
        }
        "fullScreen" => {
            for (x, y, dx, dy) in [(3.0, 3.0, 1.0, 1.0), (21.0, 3.0, -1.0, 1.0), (3.0, 21.0, 1.0, -1.0), (21.0, 21.0, -1.0, -1.0)] {
                pen.line(&[(x + 5.0 * dx, y), (x, y), (x, y + 5.0 * dy)]);
            }
        }
        "macros" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            pen.line(&[(3.0, 8.0), (21.0, 8.0)]);
            pen.line(&[(7.0, 12.0), (9.0, 14.0), (7.0, 16.0)]);
            pen.line(&[(12.0, 16.0), (17.0, 16.0)]);
        }
        "lasso" => {
            pen.line(&[
                (16.0, 15.0),
                (19.0, 13.0),
                (20.0, 9.0),
                (18.0, 5.5),
                (14.0, 4.0),
                (9.0, 4.0),
                (5.0, 6.0),
                (3.5, 9.5),
                (4.5, 13.0),
                (8.0, 15.0),
                (12.0, 15.0),
                (15.0, 17.0),
                (13.0, 20.0),
                (9.0, 21.0),
            ]);
        }
        "eraser" => {
            pen.fill(10.0, 7.0, 8.0, 6.0);
            pen.line(&[(3.5, 14.0), (12.5, 5.0), (20.5, 13.0), (13.5, 20.0), (8.0, 20.0), (3.5, 15.5), (3.5, 14.0)]);
            pen.line(&[(8.0, 9.5), (16.0, 17.5)]);
            pen.line(&[(13.5, 20.0), (21.0, 20.0)]);
        }
        "thickness" => {
            pen.line(&[(4.0, 6.0), (20.0, 6.0)]);
            pen.line(&[(4.0, 11.0), (20.0, 11.0)]);
            pen.fill(4.0, 15.0, 16.0, 3.0);
            pen.line(&[(4.0, 19.5), (20.0, 19.5)]);
        }
        "inkColor" => {
            pen.circle(10.0, 10.0, 7.0);
            pen.fill(7.0, 7.0, 6.0, 6.0);
            pen.circle(8.0, 7.0, 0.8);
            pen.circle(13.0, 7.0, 0.8);
            pen.circle(6.5, 11.5, 0.8);
            pen.line(&[(14.0, 15.0), (20.0, 21.0)]);
            pen.line(&[(17.0, 14.0), (22.0, 19.0)]);
        }
        "inkToShape" | "inkToMath" => {
            pen.line(&[(3.0, 17.0), (5.0, 13.0), (7.0, 16.0), (9.0, 11.0)]);
            pen.line(&[(10.0, 12.0), (14.0, 12.0)]);
            pen.line(&[(12.5, 10.5), (14.0, 12.0), (12.5, 13.5)]);
            if name == "inkToShape" {
                pen.fill(16.0, 6.0, 5.0, 11.0);
                pen.rect(16.0, 6.0, 5.0, 11.0);
            } else {
                pen.line(&[(21.0, 6.0), (16.0, 6.0), (19.0, 12.0), (16.0, 18.0), (21.0, 18.0)]);
            }
        }
        "canvas" => {
            pen.fill(5.0, 5.0, 14.0, 14.0);
            pen.rect(4.0, 4.0, 16.0, 16.0);
            for (x, y) in [(4.0, 4.0), (20.0, 4.0), (4.0, 20.0), (20.0, 20.0)] {
                pen.circle(x, y, 1.1);
            }
        }
        "replay" => {
            pen.line(&[
                (18.0, 7.0),
                (16.0, 4.5),
                (12.0, 3.5),
                (7.5, 5.0),
                (5.0, 8.0),
                (4.0, 12.0),
                (5.5, 16.5),
                (9.0, 19.0),
                (13.0, 20.0),
                (17.0, 18.0),
                (19.0, 15.0),
            ]);
            pen.line(&[(15.0, 7.0), (18.0, 7.0), (18.0, 4.0)]);
            pen.fill(10.0, 8.0, 5.0, 8.0);
            pen.line(&[(10.0, 8.0), (16.0, 12.0), (10.0, 16.0), (10.0, 8.0)]);
        }
        "coverPage" | "blankPage" | "pageBreak" | "header" | "footer" | "pageNumber" | "pn_top" | "pn_bottom" | "pn_xofy" | "pn_current"
        | "pn_format" | "pn_position" => {
            pen.line(&[(5.0, 2.5), (15.0, 2.5), (19.0, 6.5), (19.0, 21.5), (5.0, 21.5), (5.0, 2.5)]);
            pen.line(&[(15.0, 2.5), (15.0, 6.5), (19.0, 6.5)]);
            match name {
                "coverPage" => {
                    pen.fill(7.0, 7.0, 10.0, 4.0);
                    pen.line(&[(7.5, 14.0), (16.5, 14.0)]);
                    pen.line(&[(7.5, 17.0), (14.5, 17.0)]);
                }
                "pageBreak" => {
                    pen.line(&[(7.0, 8.0), (17.0, 8.0)]);
                    for x in [7.0, 10.5, 14.0] {
                        pen.line(&[(x, 12.0), (x + 1.5, 12.0)]);
                    }
                    pen.line(&[(7.0, 16.0), (17.0, 16.0)]);
                }
                "header" | "pn_top" => {
                    pen.fill(7.0, 5.0, 10.0, 3.0);
                    pen.line(&[(7.0, 10.5), (17.0, 10.5)]);
                    pen.line(&[(7.0, 14.0), (15.0, 14.0)]);
                }
                "footer" | "pn_bottom" | "pn_xofy" => {
                    pen.fill(7.0, 17.0, 10.0, 2.5);
                    pen.line(&[(7.0, 9.0), (17.0, 9.0)]);
                    pen.line(&[(7.0, 12.0), (15.0, 12.0)]);
                    if name == "pn_xofy" {
                        pen.text(12.0, 18.0, 5.5, "1/2");
                    }
                }
                "pageNumber" | "pn_format" => {
                    pen.text(12.0, 13.0, 11.0, "1");
                }
                "pn_current" => {
                    pen.line(&[(8.0, 10.0), (16.0, 10.0)]);
                    pen.line(&[(8.0, 14.0), (11.0, 14.0)]);
                    pen.line(&[(13.0, 12.0), (13.0, 16.0)]);
                }
                "pn_position" => {
                    pen.line(&[(8.0, 10.0), (16.0, 10.0)]);
                    pen.line(&[(12.0, 13.0), (12.0, 19.0)]);
                    pen.line(&[(10.0, 15.0), (12.0, 13.0), (14.0, 15.0)]);
                    pen.line(&[(10.0, 17.0), (12.0, 19.0), (14.0, 17.0)]);
                }
                _ => {}
            }
        }
        "table" | "textToTable" => {
            let x = if name == "table" { 3.0 } else { 10.0 };
            pen.fill(x, 4.0, 20.0 - x, 4.0);
            pen.rect(x, 4.0, 20.0 - x, 15.0);
            pen.line(&[(x, 9.0), (20.0, 9.0)]);
            pen.line(&[(x, 14.0), (20.0, 14.0)]);
            pen.line(&[((x + 20.0) / 2.0, 4.0), ((x + 20.0) / 2.0, 19.0)]);
            if name == "textToTable" {
                pen.line(&[(2.0, 8.0), (7.0, 8.0)]);
                pen.line(&[(2.0, 12.0), (7.0, 12.0)]);
                pen.line(&[(2.0, 16.0), (7.0, 16.0)]);
            }
        }
        "picture" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            pen.fill(5.0, 15.0, 14.0, 3.0);
            pen.circle(15.5, 8.5, 1.5);
            pen.line(&[(4.5, 17.5), (9.0, 12.0), (12.0, 15.0), (14.0, 13.0), (19.5, 18.0)]);
        }
        "shapes" => {
            pen.fill(4.0, 4.0, 9.0, 9.0);
            pen.rect(4.0, 4.0, 9.0, 9.0);
            pen.circle(15.0, 15.0, 5.0);
        }
        "icons" => {
            pen.fill(4.0, 4.0, 6.0, 6.0);
            pen.rect(4.0, 4.0, 6.0, 6.0);
            pen.circle(16.5, 7.0, 3.0);
            pen.line(&[(4.0, 20.0), (7.5, 13.0), (11.0, 20.0), (4.0, 20.0)]);
            pen.line(&[(16.5, 13.0), (20.0, 16.5), (16.5, 20.0), (13.0, 16.5), (16.5, 13.0)]);
        }
        "smartArt" => {
            pen.fill(3.0, 4.0, 7.0, 6.0);
            pen.rect(3.0, 4.0, 7.0, 6.0);
            pen.rect(14.0, 4.0, 7.0, 6.0);
            pen.rect(8.5, 15.0, 7.0, 5.0);
            pen.line(&[(10.0, 7.0), (14.0, 7.0)]);
            pen.line(&[(12.0, 10.0), (12.0, 15.0)]);
        }
        "chart" => {
            pen.line(&[(4.0, 3.0), (4.0, 20.0), (21.0, 20.0)]);
            pen.fill(7.0, 12.0, 3.0, 7.0);
            pen.rect(7.0, 12.0, 3.0, 7.0);
            pen.rect(12.0, 7.0, 3.0, 12.0);
            pen.rect(17.0, 10.0, 3.0, 9.0);
        }
        "link" => {
            pen.line(&[(10.0, 14.0), (14.0, 10.0)]);
            pen.line(&[(9.0, 9.0), (7.0, 9.0), (4.5, 11.5), (4.5, 15.0), (7.0, 17.5), (10.0, 17.5), (12.0, 15.5)]);
            pen.line(&[(12.0, 8.5), (14.0, 6.5), (17.0, 6.5), (19.5, 9.0), (19.5, 12.5), (17.0, 15.0), (15.0, 15.0)]);
        }
        "crossRef" => {
            pen.line(&[(4.0, 3.0), (14.0, 3.0), (14.0, 8.0)]);
            pen.line(&[(4.0, 3.0), (4.0, 18.0), (9.0, 18.0)]);
            pen.line(&[(7.0, 7.0), (11.0, 7.0)]);
            pen.line(&[(7.0, 10.0), (11.0, 10.0)]);
            pen.line(&[(10.0, 14.0), (20.0, 14.0), (20.0, 20.0), (14.0, 20.0)]);
            pen.line(&[(16.0, 18.0), (14.0, 20.0), (16.0, 22.0)]);
        }
        "newComment" => {
            pen.fill(5.0, 5.0, 14.0, 9.0);
            pen.line(&[(4.0, 4.0), (20.0, 4.0), (20.0, 16.0), (11.0, 16.0), (7.0, 20.0), (7.0, 16.0), (4.0, 16.0), (4.0, 4.0)]);
            pen.line(&[(8.0, 9.0), (16.0, 9.0)]);
            pen.line(&[(8.0, 12.0), (14.0, 12.0)]);
        }
        "quickParts" => {
            pen.rect(4.0, 4.0, 16.0, 16.0);
            pen.fill(6.0, 6.0, 5.0, 5.0);
            pen.rect(6.0, 6.0, 5.0, 5.0);
            pen.line(&[(13.0, 7.0), (18.0, 7.0)]);
            pen.line(&[(13.0, 10.0), (18.0, 10.0)]);
            pen.line(&[(6.0, 14.0), (18.0, 14.0)]);
            pen.line(&[(6.0, 17.0), (16.0, 17.0)]);
        }
        "wordArt" | "dropCap" => {
            if name == "dropCap" {
                pen.rect(3.0, 4.0, 10.0, 13.0);
                pen.line(&[(15.0, 6.0), (21.0, 6.0)]);
                pen.line(&[(15.0, 10.0), (21.0, 10.0)]);
                pen.line(&[(15.0, 14.0), (21.0, 14.0)]);
                pen.line(&[(3.0, 20.0), (21.0, 20.0)]);
                pen.text(8.0, 10.5, 12.0, "A");
            } else {
                pen.fill(6.0, 14.0, 12.0, 4.0);
                pen.line(&[(4.0, 20.0), (11.0, 3.0), (19.0, 20.0)]);
                pen.line(&[(7.0, 14.0), (16.0, 14.0)]);
            }
        }
        "signature" => {
            pen.line(&[(3.0, 19.0), (21.0, 19.0)]);
            pen.line(&[(4.0, 15.0), (7.0, 10.0), (8.5, 14.0), (11.0, 7.0), (12.0, 15.0), (15.0, 12.0), (17.0, 14.0), (20.0, 12.0)]);
        }
        "dateTime" => {
            pen.fill(4.0, 5.0, 16.0, 4.0);
            pen.rect(4.0, 5.0, 16.0, 15.0);
            pen.line(&[(4.0, 9.0), (20.0, 9.0)]);
            pen.line(&[(8.0, 3.0), (8.0, 7.0)]);
            pen.line(&[(16.0, 3.0), (16.0, 7.0)]);
            pen.circle(15.5, 15.5, 3.0);
            pen.line(&[(15.5, 13.5), (15.5, 15.5), (17.0, 16.3)]);
        }
        "object" => {
            pen.rect(3.5, 3.5, 17.0, 17.0);
            pen.fill(7.0, 7.0, 10.0, 10.0);
            pen.rect(7.0, 7.0, 10.0, 10.0);
        }
        "equation" => {
            pen.line(&[(18.0, 4.0), (6.0, 4.0), (13.0, 12.0), (6.0, 20.0), (18.0, 20.0)]);
        }
        "symbol" => {
            pen.text(12.0, 12.0, 20.0, "Ω");
        }
        "textBox" => {
            pen.rect(3.5, 4.0, 17.0, 16.0);
            pen.line(&[(7.5, 8.0), (16.5, 8.0)]);
            pen.line(&[(12.0, 8.0), (12.0, 16.0)]);
            pen.line(&[(9.5, 16.0), (14.5, 16.0)]);
        }
        "bookmark" => {
            pen.fill(7.0, 3.0, 10.0, 9.0);
            pen.line(&[(7.0, 3.0), (17.0, 3.0), (17.0, 21.0), (12.0, 17.0), (7.0, 21.0), (7.0, 3.0)]);
        }
        "drawTable" => {
            pen.rect(3.0, 4.0, 14.0, 14.0);
            pen.line(&[(3.0, 9.0), (17.0, 9.0)]);
            pen.line(&[(9.5, 4.0), (9.5, 18.0)]);
            pen.fill(14.0, 13.0, 5.0, 4.0);
            pen.line(&[(11.0, 21.0), (12.0, 17.0), (19.0, 10.0), (21.0, 12.0), (14.0, 19.0), (11.0, 21.0)]);
        }
        "dictate" => {
            pen.fill(9.0, 3.0, 6.0, 11.0);
            pen.rect(9.0, 3.0, 6.0, 11.0);
            pen.line(&[(6.0, 11.0), (6.0, 13.0), (7.5, 16.0), (10.0, 17.5), (14.0, 17.5), (16.5, 16.0), (18.0, 13.0), (18.0, 11.0)]);
            pen.line(&[(12.0, 18.0), (12.0, 21.0)]);
            pen.line(&[(8.5, 21.0), (15.5, 21.0)]);
        }
        "editor" => {
            pen.line(&[(4.0, 4.0), (16.0, 4.0)]);
            pen.line(&[(4.0, 8.0), (13.0, 8.0)]);
            pen.line(&[(4.0, 12.0), (10.0, 12.0)]);
            pen.fill(12.0, 12.0, 8.0, 5.0);
            pen.line(&[(11.0, 19.0), (12.0, 15.0), (18.5, 8.5), (21.0, 11.0), (14.5, 17.5), (11.0, 19.0)]);
            pen.line(&[(4.0, 19.0), (8.0, 19.0)]);
        }
        _ => return false,
    }
    true
}
