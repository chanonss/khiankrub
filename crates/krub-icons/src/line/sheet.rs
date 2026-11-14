//! KhitKrub's icons in the line set: which of its names (`crate::sheet::Icon::name`) the
//! shared line drawings stand for, and KhitKrub's own drawings in the same style (24-unit
//! grid, thin outlines, pale gold surface) for the commands the shared set has no picture of.
//! Every `sheet::Icon` the app shows is drawn here; the older sheet drawing is only a fallback.
//! KhitKrub's work stays in this file, so work on one app never touches the other app's file.
//! Original artwork drawn in code, `MIT OR Apache-2.0` (AGENTS.md §1).

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::Pen;
use crate::sheet::{FnGlyph, Icon};

/// The line-set name drawn for `sheet` icon `name`, when one stands for the same command.
pub(crate) fn line_name(name: &str) -> Option<&'static str> {
    Some(match name {
        "paste" => "paste",
        "cut" => "cut",
        "copy" => "copy",
        "brush" => "painter",
        "bold" => "bold",
        "italic" => "italic",
        "underline" => "underline",
        "strike" => "strike",
        "borders" => "borders",
        "fill" => "shading",
        "font-color" => "fontcolor",
        "align-left" => "alignLeft",
        "align-center" => "alignCenter",
        "align-right" => "alignRight",
        "indent-dec" => "outdent",
        "indent-inc" => "indent",
        "merge" => "merge",
        "find" => "find",
        "search" => "find",
        "sort-asc" => "sort",
        "table" => "table",
        "chart" => "chart",
        "chart-bar" => "chart",
        "picture" => "picture",
        "shapes" => "shapes",
        "link" => "link",
        "comment" => "newComment",
        "text-box" => "textBox",
        "symbol" => "symbol",
        "sheet-equation" => "equation",
        "spell" => "spelling",
        "zoom" => "zoom",
        "gridlines" => "gridlines",
        "theme" => "themes",
        "margins" => "margins",
        "orient" => "orientation",
        "paper" => "size",
        "pen" => "pen",
        "eraser" => "eraser",
        "header" => "header",
        "footer" => "footer",
        "page-number" => "pageNumber",
        "calendar" => "dateTime",
        "macro" => "macros",
        "page-layout" => "printLayout",
        "appearance" => "darkMode",
        "save" => "optionsSave",
        "delete" => "del_cell",
        "format" => "autofit",
        "sheet-list" => "bullets",
        "insert-icons" => "icons",
        _ => return None,
    })
}

/// Every name [`paint`] draws itself (not through [`line_name`]), for the catalog test.
#[cfg(test)]
pub(crate) const OWN: &[&str] = &[
    "border-all",
    "border-none",
    "border-outside",
    "border-thick",
    "align-top",
    "align-middle",
    "align-bottom",
    "orientation",
    "wrap",
    "currency",
    "percent",
    "comma",
    "dec-inc",
    "dec-dec",
    "cond-format",
    "format-table",
    "cell-styles",
    "insert",
    "sum",
    "fill-down",
    "sort-filter",
    "undo",
    "redo",
    "home",
    "more",
    "chevron",
    "chevron-up",
    "close",
    "check",
    "fx",
    "plus",
    "left",
    "right",
    "normal",
    "page-break",
    "minus",
    "pivot-table",
    "chart-line",
    "chart-pie",
    "chart-scatter",
    "note",
    "sparkline",
    "checkbox",
    "form-button",
    "form-slider",
    "form-spin",
    "function",
    "name",
    "trace",
    "calc",
    "filter",
    "sort-desc",
    "duplicates",
    "text-columns",
    "validation",
    "group",
    "ungroup",
    "subtotal",
    "lock",
    "freeze",
    "sheet-rtl",
    "print-area",
    "script",
    "share",
    "settings",
    "sheet",
    "book",
    "folder",
    "fn-book-star",
    "fn-book-coins",
    "fn-book-question",
    "fn-book-letter",
    "fn-book-clock",
    "fn-book-magnifier",
    "fn-book-theta",
    "fn-book-dots",
    "page-count",
    "clock",
    "file-name",
    "back",
    "new-file",
    "open-folder",
    "info",
    "print",
    "record",
    "relative",
    "collapse",
    "expand",
    "play",
    "step",
    "continue",
    "run-to",
    "stop-square",
    "more-charts",
];

/// Draw `icon` in the line style: the shared drawing [`line_name`] names, else KhitKrub's own.
/// `false` when neither has it, so the caller keeps the older drawing.
pub(crate) fn paint(p: &Painter, r: Rect, icon: Icon, ink: Color32, flat: bool) -> bool {
    if let Some(name) = line_name(icon.name())
        && super::paint_ex(p, r, name, ink, false, flat)
    {
        return true;
    }
    let size = r.width().min(r.height());
    if !size.is_finite() || size <= 0.5 || !r.center().is_finite() {
        return false;
    }
    let scale = size / 24.0;
    let accent = if flat || ink == Color32::WHITE { ink.gamma_multiply(0.35) } else { crate::gold(ink, crate::GoldRole::Surface, 1.0) };
    let pen = Pen { painter: p, origin: r.center() - vec2(size, size) / 2.0, scale, ink, accent };
    draw(&pen, icon)
}

/// A closed shape filled with `color` (convex outlines only).
fn poly(pen: &Pen<'_>, points: &[(f32, f32)], color: Color32) {
    let path: Vec<Pos2> = points.iter().map(|&(x, y)| pen.point(x, y)).collect();
    pen.painter.add(Shape::convex_polygon(path, color, Stroke::NONE));
}

/// A line from `a` to `b` with an open arrowhead at `b`.
fn arrow(pen: &Pen<'_>, a: (f32, f32), b: (f32, f32)) {
    pen.line(&[a, b]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / len * 2.6, dy / len * 2.6);
    pen.line(&[(b.0 - ux - uy, b.1 - uy + ux), b, (b.0 - ux + uy, b.1 - uy - ux)]);
}

/// A dashed straight line.
fn dashed(pen: &Pen<'_>, a: (f32, f32), b: (f32, f32)) {
    let steps = 5;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = t0 + 0.5 / steps as f32;
        pen.line(&[(a.0 + (b.0 - a.0) * t0, a.1 + (b.1 - a.1) * t0), (a.0 + (b.0 - a.0) * t1, a.1 + (b.1 - a.1) * t1)]);
    }
}

/// A page with a folded corner, as the shared set draws one.
fn page(pen: &Pen<'_>) {
    pen.line(&[(5.0, 2.5), (15.0, 2.5), (19.0, 6.5), (19.0, 21.5), (5.0, 21.5), (5.0, 2.5)]);
    pen.line(&[(15.0, 2.5), (15.0, 6.5), (19.0, 6.5)]);
}

/// A 3×3 block of cells at (4, 4)–(19, 19), with the middle cell lit.
fn cells(pen: &Pen<'_>) {
    pen.fill(9.0, 9.0, 5.0, 5.0);
    pen.rect(4.0, 4.0, 15.0, 15.0);
    for v in [9.0, 14.0] {
        pen.line(&[(v, 4.0), (v, 19.0)]);
        pen.line(&[(4.0, v), (19.0, v)]);
    }
}

/// Chart axes.
fn axes(pen: &Pen<'_>) {
    pen.line(&[(4.0, 3.0), (4.0, 20.0), (21.0, 20.0)]);
}

/// A point on a circle round (`cx`, `cy`) at `turn` (0 = up, clockwise, 1 = a full turn).
fn round(cx: f32, cy: f32, radius: f32, turn: f32) -> (f32, f32) {
    let a = turn * std::f32::consts::TAU;
    (cx + radius * a.sin(), cy - radius * a.cos())
}

/// fx: an italic f and an x.
fn fx(pen: &Pen<'_>, dx: f32, dy: f32, k: f32) {
    let at = |x: f32, y: f32| (12.0 + (x - 12.0) * k + dx, 12.0 + (y - 12.0) * k + dy);
    pen.line(&[at(14.0, 4.5), at(12.0, 4.0), at(10.3, 5.0), at(9.3, 8.0), at(8.0, 18.0), at(7.0, 20.0), at(5.0, 20.5)]);
    pen.line(&[at(6.5, 10.0), at(12.5, 10.0)]);
    pen.line(&[at(13.0, 11.5), at(19.0, 19.0)]);
    pen.line(&[at(19.0, 11.5), at(13.0, 19.0)]);
}

// One flat table of drawings, one arm per icon: long, but each arm is simple.
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn draw(pen: &Pen<'_>, icon: Icon) -> bool {
    let accent = pen.accent;
    match icon {
        Icon::BorderAll | Icon::BorderNone | Icon::BorderOutside | Icon::BorderThick => {
            let inner_solid = icon == Icon::BorderAll;
            if icon == Icon::BorderAll {
                pen.fill(4.0, 4.0, 16.0, 16.0);
            }
            if inner_solid {
                pen.line(&[(12.0, 4.0), (12.0, 20.0)]);
                pen.line(&[(4.0, 12.0), (20.0, 12.0)]);
            } else {
                dashed(pen, (12.0, 4.5), (12.0, 20.5));
                dashed(pen, (4.5, 12.0), (20.5, 12.0));
            }
            match icon {
                Icon::BorderNone => {
                    for (a, b) in [((4.0, 4.0), (20.0, 4.0)), ((20.0, 4.0), (20.0, 20.0)), ((4.0, 20.0), (20.0, 20.0)), ((4.0, 4.0), (4.0, 20.0))] {
                        dashed(pen, a, (b.0 + (b.0 - a.0) / 9.0, b.1 + (b.1 - a.1) / 9.0));
                    }
                }
                Icon::BorderThick => {
                    pen.rect(4.0, 4.0, 16.0, 16.0);
                    pen.rect(5.2, 5.2, 13.6, 13.6);
                }
                _ => pen.rect(4.0, 4.0, 16.0, 16.0),
            }
        }
        Icon::AlignTop | Icon::AlignMiddle | Icon::AlignBottom => {
            let y = match icon {
                Icon::AlignTop => 4.0,
                Icon::AlignMiddle => 12.0,
                _ => 20.0,
            };
            pen.line(&[(4.0, y), (20.0, y)]);
            match icon {
                Icon::AlignTop => arrow(pen, (12.0, 20.0), (12.0, 7.0)),
                Icon::AlignBottom => arrow(pen, (12.0, 4.0), (12.0, 17.0)),
                _ => {
                    arrow(pen, (12.0, 2.5), (12.0, 9.0));
                    arrow(pen, (12.0, 21.5), (12.0, 15.0));
                }
            }
        }
        Icon::Orientation => {
            pen.text(8.0, 8.0, 9.0, "ab");
            arrow(pen, (6.0, 20.0), (19.5, 6.5));
        }
        Icon::Wrap => {
            pen.line(&[(4.0, 5.5), (20.0, 5.5)]);
            pen.line(&[(4.0, 11.5), (17.0, 11.5), (19.5, 12.5), (20.5, 14.5), (19.5, 16.5), (17.0, 17.5), (12.0, 17.5)]);
            pen.line(&[(14.5, 15.0), (12.0, 17.5), (14.5, 20.0)]);
            pen.line(&[(4.0, 17.5), (8.5, 17.5)]);
        }
        Icon::Currency => {
            pen.fill(3.0, 6.0, 18.0, 12.0);
            pen.rect(3.0, 6.0, 18.0, 12.0);
            pen.circle(12.0, 12.0, 3.2);
            pen.dot(6.0, 9.0, 0.9);
            pen.dot(18.0, 15.0, 0.9);
        }
        Icon::Percent => {
            pen.fill(5.0, 5.0, 5.0, 5.0);
            pen.circle(7.5, 7.5, 2.8);
            pen.circle(16.5, 16.5, 2.8);
            pen.line(&[(18.5, 4.5), (5.5, 19.5)]);
        }
        Icon::Comma => {
            pen.text(16.0, 10.5, 9.0, "000");
            pen.dot(6.5, 15.0, 1.7);
            pen.line(&[(7.8, 15.3), (7.2, 18.0), (5.2, 20.5)]);
        }
        Icon::DecInc | Icon::DecDec => {
            let (top, bottom) = if icon == Icon::DecInc { (".0", ".00") } else { (".00", ".0") };
            pen.text(14.0, 6.5, 9.0, top);
            pen.text(15.0, 17.0, 9.0, bottom);
            if icon == Icon::DecInc {
                arrow(pen, (8.0, 17.0), (2.5, 17.0));
            } else {
                arrow(pen, (2.5, 17.0), (8.0, 17.0));
            }
        }
        Icon::CondFormat => {
            pen.rect(3.0, 3.5, 18.0, 17.0);
            for y in [9.2, 14.8] {
                pen.line(&[(3.0, y), (21.0, y)]);
            }
            pen.fill(5.0, 5.2, 13.0, 2.4);
            pen.fill(5.0, 10.8, 7.0, 2.4);
            pen.fill(5.0, 16.4, 10.0, 2.4);
            pen.line(&[(5.0, 6.4), (18.0, 6.4)]);
            pen.line(&[(5.0, 12.0), (12.0, 12.0)]);
            pen.line(&[(5.0, 17.6), (15.0, 17.6)]);
        }
        Icon::FormatTable => {
            pen.fill(3.0, 4.0, 18.0, 4.0);
            pen.fill(3.0, 12.0, 18.0, 4.0);
            pen.rect(3.0, 4.0, 18.0, 16.0);
            for y in [8.0, 12.0, 16.0] {
                pen.line(&[(3.0, y), (21.0, y)]);
            }
            pen.line(&[(10.0, 4.0), (10.0, 20.0)]);
        }
        Icon::CellStyles => {
            pen.fill(3.0, 3.0, 8.0, 8.0);
            pen.fill(13.0, 13.0, 8.0, 8.0);
            for (x, y) in [(3.0, 3.0), (13.0, 3.0), (3.0, 13.0), (13.0, 13.0)] {
                pen.rect(x, y, 8.0, 8.0);
            }
            pen.text(17.0, 7.0, 7.0, "A");
            pen.line(&[(5.0, 17.0), (9.0, 17.0)]);
        }
        Icon::Insert => {
            cells(pen);
            pen.line(&[(15.0, 18.0), (21.0, 18.0)]);
            pen.line(&[(18.0, 15.0), (18.0, 21.0)]);
        }
        Icon::Sum => {
            pen.line(&[(18.0, 6.5), (18.0, 4.0), (6.0, 4.0), (13.0, 12.0), (6.0, 20.0), (18.0, 20.0), (18.0, 17.5)]);
        }
        Icon::FillDown => {
            pen.fill(4.0, 3.0, 10.0, 6.0);
            pen.rect(4.0, 3.0, 10.0, 18.0);
            pen.line(&[(4.0, 9.0), (14.0, 9.0)]);
            pen.line(&[(4.0, 15.0), (14.0, 15.0)]);
            arrow(pen, (18.5, 4.0), (18.5, 19.5));
        }
        Icon::SortFilter => {
            pen.text(6.0, 7.0, 8.0, "A");
            pen.text(6.0, 17.0, 8.0, "Z");
            pen.fill(13.5, 6.0, 6.0, 3.0);
            pen.line(&[(11.5, 5.0), (21.5, 5.0), (17.8, 10.5), (17.8, 18.0), (15.2, 20.0), (15.2, 10.5), (11.5, 5.0)]);
        }
        Icon::Filter => {
            pen.fill(7.0, 6.0, 10.0, 3.5);
            pen.line(&[(3.5, 4.5), (20.5, 4.5), (14.0, 12.0), (14.0, 19.0), (10.0, 21.0), (10.0, 12.0), (3.5, 4.5)]);
        }
        Icon::SortDesc => {
            pen.text(7.0, 7.0, 9.0, "Z");
            pen.text(7.0, 17.0, 9.0, "A");
            pen.line(&[(17.0, 4.0), (17.0, 19.0), (14.5, 16.5)]);
            pen.line(&[(17.0, 19.0), (19.5, 16.5)]);
        }
        Icon::Undo | Icon::Redo => {
            let m = |x: f32| if icon == Icon::Undo { x } else { 24.0 - x };
            pen.line(&[(m(5.0), 9.0), (m(15.0), 9.0), (m(18.0), 10.2), (m(19.5), 13.5), (m(18.0), 16.8), (m(15.0), 18.0), (m(8.0), 18.0)]);
            pen.line(&[(m(8.5), 5.5), (m(5.0), 9.0), (m(8.5), 12.5)]);
        }
        Icon::Home => {
            pen.line(&[(3.0, 11.5), (12.0, 3.5), (21.0, 11.5)]);
            pen.line(&[(5.5, 9.5), (5.5, 20.5), (18.5, 20.5), (18.5, 9.5)]);
            pen.fill(10.0, 14.0, 4.0, 6.5);
            pen.rect(10.0, 14.0, 4.0, 6.5);
        }
        Icon::More => {
            for x in [5.5, 12.0, 18.5] {
                pen.dot(x, 12.0, 1.6);
            }
        }
        Icon::Chevron => pen.line(&[(6.5, 9.0), (12.0, 14.5), (17.5, 9.0)]),
        Icon::ChevronUp => pen.line(&[(6.5, 15.0), (12.0, 9.5), (17.5, 15.0)]),
        Icon::Left => pen.line(&[(15.0, 6.5), (9.5, 12.0), (15.0, 17.5)]),
        Icon::Right => pen.line(&[(9.0, 6.5), (14.5, 12.0), (9.0, 17.5)]),
        Icon::Close => {
            pen.line(&[(6.0, 6.0), (18.0, 18.0)]);
            pen.line(&[(18.0, 6.0), (6.0, 18.0)]);
        }
        Icon::Check => pen.line(&[(4.5, 12.5), (9.5, 17.5), (19.5, 6.5)]),
        Icon::Plus => {
            pen.line(&[(12.0, 5.0), (12.0, 19.0)]);
            pen.line(&[(5.0, 12.0), (19.0, 12.0)]);
        }
        Icon::Minus => pen.line(&[(5.0, 12.0), (19.0, 12.0)]),
        Icon::Fx => fx(pen, 0.0, 0.0, 1.0),
        Icon::Function => {
            pen.fill(3.0, 3.0, 18.0, 4.0);
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(3.0, 7.0), (21.0, 7.0)]);
            fx(pen, 0.0, 2.2, 0.62);
        }
        Icon::Normal => {
            pen.fill(3.0, 3.0, 18.0, 4.0);
            pen.fill(3.0, 7.0, 4.0, 14.0);
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(3.0, 7.0), (21.0, 7.0)]);
            pen.line(&[(7.0, 3.0), (7.0, 21.0)]);
            pen.line(&[(14.0, 7.0), (14.0, 21.0)]);
            pen.line(&[(7.0, 14.0), (21.0, 14.0)]);
        }
        Icon::PageBreak => {
            pen.fill(5.0, 2.5, 14.0, 8.0);
            pen.rect(5.0, 2.5, 14.0, 8.0);
            pen.rect(5.0, 13.5, 14.0, 8.0);
            dashed(pen, (2.0, 12.0), (23.0, 12.0));
        }
        Icon::PivotTable => {
            pen.fill(9.0, 3.0, 12.0, 6.0);
            pen.fill(3.0, 9.0, 6.0, 12.0);
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(9.0, 3.0), (9.0, 21.0)]);
            pen.line(&[(3.0, 9.0), (21.0, 9.0)]);
            pen.line(&[(17.5, 12.5), (17.5, 17.5), (12.5, 17.5)]);
            pen.line(&[(15.5, 14.5), (17.5, 12.5), (19.5, 14.5)]);
            pen.line(&[(14.5, 15.5), (12.5, 17.5), (14.5, 19.5)]);
        }
        Icon::ChartLine => {
            axes(pen);
            let points = [(6.5, 16.0), (10.5, 10.5), (14.5, 13.5), (19.5, 6.0)];
            pen.line(&points);
            for (x, y) in points {
                pen.dot(x, y, 1.2);
            }
        }
        Icon::ChartPie => {
            let wedge: Vec<(f32, f32)> =
                std::iter::once((12.0, 12.0)).chain((0..=8).map(|i| round(12.0, 12.0, 8.5, i as f32 / 8.0 * 0.33))).collect();
            poly(pen, &wedge, accent);
            pen.circle(12.0, 12.0, 8.5);
            pen.line(&[(12.0, 3.5), (12.0, 12.0), round(12.0, 12.0, 8.5, 0.33)]);
        }
        Icon::ChartScatter => {
            axes(pen);
            for (x, y) in [(8.0, 15.5), (11.0, 11.0), (14.5, 13.5), (16.5, 7.5), (19.5, 10.0)] {
                pen.dot(x, y, 1.5);
            }
        }
        Icon::MoreCharts => {
            pen.fill(3.0, 3.0, 8.0, 8.0);
            pen.rect(3.0, 3.0, 8.0, 8.0);
            pen.line(&[(7.0, 3.0), (7.0, 11.0)]);
            pen.circle(17.0, 7.0, 4.0);
            pen.line(&[(17.0, 3.0), (17.0, 7.0), (21.0, 7.0)]);
            pen.line(&[(3.0, 20.0), (5.5, 15.0), (8.0, 17.5), (11.0, 13.0)]);
            pen.rect(14.0, 15.5, 2.5, 5.0);
            pen.rect(18.5, 13.0, 2.5, 7.5);
        }
        Icon::Note => {
            pen.fill(5.0, 5.0, 14.0, 3.5);
            pen.line(&[(4.0, 4.0), (20.0, 4.0), (20.0, 15.0), (15.0, 20.0), (4.0, 20.0), (4.0, 4.0)]);
            pen.line(&[(15.0, 20.0), (15.0, 15.0), (20.0, 15.0)]);
            pen.line(&[(7.0, 11.0), (17.0, 11.0)]);
            pen.line(&[(7.0, 14.5), (12.0, 14.5)]);
        }
        Icon::Sparkline => {
            pen.fill(3.0, 6.5, 18.0, 11.0);
            pen.rect(3.0, 6.5, 18.0, 11.0);
            pen.line(&[(5.0, 14.5), (8.0, 10.0), (11.0, 13.0), (14.0, 9.0), (16.5, 12.0), (19.0, 9.5)]);
        }
        Icon::Checkbox => {
            pen.fill(4.0, 4.0, 16.0, 16.0);
            pen.rect(4.0, 4.0, 16.0, 16.0);
            pen.line(&[(7.5, 12.0), (10.5, 15.5), (16.5, 8.5)]);
        }
        Icon::Button => {
            pen.fill(3.0, 5.0, 18.0, 10.0);
            pen.rect(3.0, 5.0, 18.0, 10.0);
            pen.line(&[(13.0, 11.0), (13.0, 21.5), (15.5, 19.2), (17.2, 22.5), (18.8, 21.8), (17.2, 18.5), (20.5, 18.5), (13.0, 11.0)]);
        }
        Icon::Slider => {
            pen.line(&[(3.0, 12.0), (21.0, 12.0)]);
            for x in [3.0, 21.0] {
                pen.line(&[(x, 10.0), (x, 14.0)]);
            }
            pen.fill(9.5, 7.0, 5.0, 10.0);
            pen.rect(9.5, 7.0, 5.0, 10.0);
        }
        Icon::Spin => {
            pen.fill(7.0, 3.0, 10.0, 9.0);
            pen.rect(7.0, 3.0, 10.0, 18.0);
            pen.line(&[(7.0, 12.0), (17.0, 12.0)]);
            pen.line(&[(9.5, 9.0), (12.0, 6.5), (14.5, 9.0)]);
            pen.line(&[(9.5, 15.0), (12.0, 17.5), (14.5, 15.0)]);
        }
        Icon::Name => {
            pen.fill(4.0, 9.0, 10.0, 6.0);
            pen.line(&[(3.0, 7.5), (15.5, 7.5), (21.0, 12.0), (15.5, 16.5), (3.0, 16.5), (3.0, 7.5)]);
            pen.dot(16.5, 12.0, 1.1);
            pen.line(&[(6.0, 12.0), (11.5, 12.0)]);
        }
        Icon::Trace => {
            pen.fill(3.0, 4.0, 6.0, 5.0);
            pen.rect(3.0, 4.0, 6.0, 5.0);
            pen.rect(15.0, 16.0, 6.0, 5.0);
            pen.dot(9.5, 6.5, 1.1);
            pen.line(&[(9.5, 6.5), (18.0, 6.5), (18.0, 13.0)]);
            pen.line(&[(16.0, 11.0), (18.0, 13.5), (20.0, 11.0)]);
        }
        Icon::Calc => {
            pen.rect(5.0, 2.5, 14.0, 19.0);
            pen.fill(7.5, 5.0, 9.0, 4.0);
            pen.rect(7.5, 5.0, 9.0, 4.0);
            for y in [12.5, 15.5, 18.5] {
                for x in [8.5, 12.0, 15.5] {
                    pen.dot(x, y, 0.95);
                }
            }
        }
        Icon::Duplicates => {
            pen.fill(3.0, 4.0, 11.0, 5.0);
            pen.rect(3.0, 4.0, 11.0, 5.0);
            pen.rect(3.0, 11.0, 11.0, 5.0);
            pen.line(&[(3.0, 18.5), (14.0, 18.5)]);
            pen.line(&[(16.0, 9.0), (21.0, 14.0)]);
            pen.line(&[(21.0, 9.0), (16.0, 14.0)]);
        }
        Icon::TextColumns => {
            pen.line(&[(2.5, 7.0), (9.0, 7.0)]);
            pen.line(&[(2.5, 11.0), (9.0, 11.0)]);
            pen.line(&[(2.5, 15.0), (7.5, 15.0)]);
            arrow(pen, (9.5, 18.5), (14.0, 18.5));
            pen.fill(15.0, 4.0, 3.0, 16.0);
            pen.rect(15.0, 4.0, 6.5, 16.0);
            pen.line(&[(18.2, 4.0), (18.2, 20.0)]);
        }
        Icon::Validation => {
            pen.fill(3.0, 4.0, 18.0, 7.0);
            pen.rect(3.0, 4.0, 18.0, 7.0);
            pen.line(&[(6.0, 7.5), (13.0, 7.5)]);
            pen.line(&[(7.0, 16.0), (10.5, 19.5), (17.5, 13.0)]);
        }
        Icon::Group | Icon::Ungroup => {
            if icon == Icon::Group {
                pen.line(&[(7.0, 4.0), (4.0, 4.0), (4.0, 20.0), (7.0, 20.0)]);
                pen.fill(9.0, 9.5, 12.0, 5.0);
            } else {
                pen.line(&[(7.0, 4.0), (4.0, 4.0), (4.0, 8.5)]);
                pen.line(&[(4.0, 15.5), (4.0, 20.0), (7.0, 20.0)]);
            }
            for y in [6.0, 12.0, 18.0] {
                pen.line(&[(9.0, y), (21.0, y)]);
            }
        }
        Icon::Subtotal => {
            pen.line(&[(3.0, 5.0), (13.0, 5.0)]);
            pen.line(&[(3.0, 9.0), (13.0, 9.0)]);
            pen.line(&[(3.0, 13.0), (21.0, 13.0)]);
            pen.fill(3.0, 15.0, 9.0, 5.0);
            pen.line(&[(20.5, 16.8), (20.5, 15.5), (15.0, 15.5), (18.0, 18.5), (15.0, 21.5), (20.5, 21.5), (20.5, 20.2)]);
        }
        Icon::Lock => {
            pen.line(&[(8.0, 11.0), (8.0, 7.5), (9.3, 5.0), (12.0, 3.8), (14.7, 5.0), (16.0, 7.5), (16.0, 11.0)]);
            pen.fill(5.0, 11.0, 14.0, 10.0);
            pen.rect(5.0, 11.0, 14.0, 10.0);
            pen.dot(12.0, 15.2, 1.2);
            pen.line(&[(12.0, 15.5), (12.0, 18.0)]);
        }
        Icon::Freeze => {
            pen.fill(3.0, 3.0, 18.0, 5.0);
            pen.fill(3.0, 8.0, 5.0, 13.0);
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(3.0, 8.0), (21.0, 8.0)]);
            pen.line(&[(8.0, 3.0), (8.0, 21.0)]);
            pen.line(&[(14.5, 8.0), (14.5, 21.0)]);
            pen.line(&[(8.0, 14.5), (21.0, 14.5)]);
        }
        Icon::SheetRtl => {
            pen.fill(15.0, 3.0, 6.0, 5.0);
            pen.rect(3.0, 3.0, 18.0, 11.0);
            pen.line(&[(3.0, 8.0), (21.0, 8.0)]);
            pen.line(&[(9.0, 3.0), (9.0, 14.0)]);
            pen.line(&[(15.0, 3.0), (15.0, 14.0)]);
            arrow(pen, (20.5, 19.0), (3.5, 19.0));
        }
        Icon::PrintArea => {
            page(pen);
            pen.fill(8.0, 9.0, 8.0, 8.0);
            dashed(pen, (8.0, 9.0), (17.5, 9.0));
            dashed(pen, (8.0, 17.0), (17.5, 17.0));
            dashed(pen, (8.0, 9.0), (8.0, 18.5));
            dashed(pen, (16.0, 9.0), (16.0, 18.5));
        }
        Icon::Script => {
            page(pen);
            pen.line(&[(10.0, 10.5), (7.5, 13.5), (10.0, 16.5)]);
            pen.line(&[(14.0, 10.5), (16.5, 13.5), (14.0, 16.5)]);
            pen.line(&[(13.0, 9.5), (11.0, 17.5)]);
        }
        Icon::Share => {
            for (x, y) in [(17.0, 5.5), (6.5, 12.0), (17.0, 18.5)] {
                poly(pen, &(0..12).map(|i| round(x, y, 2.6, i as f32 / 12.0)).collect::<Vec<_>>(), accent);
                pen.circle(x, y, 2.6);
            }
            pen.line(&[(8.8, 10.6), (14.7, 6.9)]);
            pen.line(&[(8.8, 13.4), (14.7, 17.1)]);
        }
        Icon::Settings => {
            let outline: Vec<(f32, f32)> = (0..=32)
                .map(|i| {
                    let k = i % 4;
                    let radius = if k == 1 || k == 2 { 9.0 } else { 6.8 };
                    round(12.0, 12.0, radius, (i as f32 - 0.5) / 32.0)
                })
                .collect();
            pen.line(&outline);
            pen.circle(12.0, 12.0, 3.0);
        }
        Icon::Sheet => {
            pen.rect(3.0, 3.0, 18.0, 14.0);
            pen.line(&[(3.0, 7.5), (21.0, 7.5)]);
            pen.line(&[(3.0, 12.0), (21.0, 12.0)]);
            pen.line(&[(9.0, 3.0), (9.0, 17.0)]);
            pen.line(&[(15.0, 3.0), (15.0, 17.0)]);
            pen.fill(5.0, 17.0, 6.5, 3.5);
            pen.line(&[(5.0, 17.0), (5.0, 20.5), (11.0, 20.5), (12.0, 17.0)]);
        }
        Icon::Book => {
            pen.fill(4.0, 6.5, 7.0, 3.0);
            pen.line(&[(12.0, 6.0), (9.0, 4.5), (3.0, 4.5), (3.0, 18.5), (9.0, 18.5), (12.0, 20.0)]);
            pen.line(&[(12.0, 6.0), (15.0, 4.5), (21.0, 4.5), (21.0, 18.5), (15.0, 18.5), (12.0, 20.0)]);
            pen.line(&[(12.0, 6.0), (12.0, 20.0)]);
            pen.line(&[(14.5, 9.0), (18.5, 9.0)]);
            pen.line(&[(14.5, 12.5), (18.5, 12.5)]);
        }
        Icon::FnBook(glyph) => {
            pen.fill(5.0, 3.0, 4.0, 18.0);
            pen.rect(5.0, 3.0, 14.0, 18.0);
            pen.line(&[(9.0, 3.0), (9.0, 21.0)]);
            let (cx, cy) = (14.0, 12.0);
            match glyph {
                FnGlyph::Star => {
                    let star: Vec<(f32, f32)> = (0..=10).map(|i| round(cx, cy, if i % 2 == 0 { 3.8 } else { 1.7 }, i as f32 / 10.0)).collect();
                    pen.line(&star);
                }
                FnGlyph::Coins => {
                    for y in [9.0, 12.0, 15.0] {
                        pen.rect(cx - 3.0, y - 1.0, 6.0, 2.4);
                    }
                }
                FnGlyph::Question => {
                    pen.line(&[
                        (cx - 2.3, cy - 2.5),
                        (cx - 1.5, cy - 4.0),
                        (cx, cy - 4.5),
                        (cx + 1.8, cy - 3.8),
                        (cx + 2.3, cy - 2.2),
                        (cx, cy),
                        (cx, cy + 1.5),
                    ]);
                    pen.dot(cx, cy + 4.0, 0.9);
                }
                FnGlyph::Letter => {
                    pen.line(&[(cx - 3.0, cy + 4.5), (cx, cy - 4.5), (cx + 3.0, cy + 4.5)]);
                    pen.line(&[(cx - 1.8, cy + 1.3), (cx + 1.8, cy + 1.3)]);
                }
                FnGlyph::Clock => {
                    pen.circle(cx, cy, 3.8);
                    pen.line(&[(cx, cy - 2.3), (cx, cy), (cx + 1.8, cy + 1.0)]);
                }
                FnGlyph::Magnifier => {
                    pen.circle(cx - 0.8, cy - 0.8, 2.8);
                    pen.line(&[(cx + 1.2, cy + 1.2), (cx + 3.5, cy + 3.5)]);
                }
                FnGlyph::Theta => {
                    pen.circle(cx, cy, 3.8);
                    pen.line(&[(cx - 3.8, cy), (cx + 3.8, cy)]);
                }
                FnGlyph::Dots => {
                    for x in [cx - 2.8, cx, cx + 2.8] {
                        pen.dot(x, cy, 0.95);
                    }
                }
            }
        }
        Icon::Folder => {
            pen.fill(4.0, 9.5, 16.0, 9.0);
            pen.line(&[(3.0, 5.5), (9.0, 5.5), (11.0, 8.0), (21.0, 8.0), (21.0, 19.0), (3.0, 19.0), (3.0, 5.5)]);
            pen.line(&[(3.0, 10.0), (21.0, 10.0)]);
        }
        Icon::OpenFolder => {
            pen.line(&[(3.0, 19.0), (3.0, 5.5), (9.0, 5.5), (11.0, 8.0), (19.0, 8.0), (19.0, 11.0)]);
            poly(pen, &[(6.0, 11.0), (22.0, 11.0), (19.0, 19.0), (3.0, 19.0)], accent);
            pen.line(&[(3.0, 19.0), (6.0, 11.0), (22.0, 11.0), (19.0, 19.0), (3.0, 19.0)]);
        }
        Icon::NewFile => {
            page(pen);
            pen.line(&[(12.0, 11.0), (12.0, 18.0)]);
            pen.line(&[(8.5, 14.5), (15.5, 14.5)]);
        }
        Icon::PageCount => {
            page(pen);
            pen.text(12.0, 14.0, 10.0, "#");
        }
        Icon::FileName => {
            page(pen);
            pen.fill(7.0, 9.0, 10.0, 4.0);
            pen.line(&[(8.5, 11.0), (15.5, 11.0)]);
            pen.line(&[(7.5, 16.0), (16.5, 16.0)]);
            pen.line(&[(7.5, 19.0), (13.5, 19.0)]);
        }
        Icon::Clock => {
            pen.circle(12.0, 12.0, 8.5);
            pen.line(&[(12.0, 6.5), (12.0, 12.0), (16.0, 14.5)]);
        }
        Icon::Back => {
            pen.circle(12.0, 12.0, 9.0);
            arrow(pen, (17.0, 12.0), (7.5, 12.0));
        }
        Icon::Info => {
            pen.circle(12.0, 12.0, 8.5);
            pen.dot(12.0, 7.8, 1.2);
            pen.line(&[(12.0, 11.0), (12.0, 17.0)]);
        }
        Icon::Print => {
            pen.rect(7.0, 3.0, 10.0, 6.0);
            pen.fill(5.0, 10.0, 14.0, 3.0);
            pen.line(&[(7.0, 17.0), (4.0, 17.0), (4.0, 9.0), (20.0, 9.0), (20.0, 17.0), (17.0, 17.0)]);
            pen.rect(7.0, 14.0, 10.0, 7.0);
            pen.line(&[(9.0, 17.5), (15.0, 17.5)]);
        }
        Icon::Record => {
            pen.circle(12.0, 12.0, 8.5);
            pen.dot(12.0, 12.0, 4.5);
        }
        Icon::Relative => {
            pen.fill(3.0, 3.0, 7.0, 6.0);
            pen.rect(3.0, 3.0, 7.0, 6.0);
            pen.rect(14.0, 15.0, 7.0, 6.0);
            pen.line(&[(6.5, 9.0), (6.5, 18.0), (11.5, 18.0)]);
            pen.line(&[(9.5, 15.5), (12.5, 18.0), (9.5, 20.5)]);
        }
        Icon::Collapse | Icon::Expand => {
            pen.fill(3.0, 14.5, 18.0, 6.0);
            pen.rect(3.0, 14.5, 18.0, 6.0);
            if icon == Icon::Collapse {
                arrow(pen, (12.0, 2.5), (12.0, 11.5));
            } else {
                arrow(pen, (12.0, 11.5), (12.0, 2.5));
            }
        }
        Icon::Play => {
            poly(pen, &[(7.0, 4.5), (19.0, 12.0), (7.0, 19.5)], accent);
            pen.line(&[(7.0, 4.5), (19.0, 12.0), (7.0, 19.5), (7.0, 4.5)]);
        }
        Icon::Step => {
            arrow(pen, (12.0, 3.0), (12.0, 15.0));
            pen.line(&[(5.0, 19.5), (19.0, 19.5)]);
        }
        Icon::Continue => {
            pen.line(&[(5.5, 5.0), (5.5, 19.0)]);
            poly(pen, &[(10.0, 5.0), (20.0, 12.0), (10.0, 19.0)], accent);
            pen.line(&[(10.0, 5.0), (20.0, 12.0), (10.0, 19.0), (10.0, 5.0)]);
        }
        Icon::RunTo => {
            pen.line(&[(10.0, 5.0), (20.5, 5.0)]);
            pen.fill(9.0, 10.0, 12.5, 4.0);
            pen.line(&[(10.0, 12.0), (20.5, 12.0)]);
            pen.line(&[(10.0, 19.0), (20.5, 19.0)]);
            arrow(pen, (2.5, 12.0), (7.5, 12.0));
        }
        Icon::StopSquare => {
            pen.fill(6.0, 6.0, 12.0, 12.0);
            pen.rect(6.0, 6.0, 12.0, 12.0);
        }
        _ => return false,
    }
    true
}
