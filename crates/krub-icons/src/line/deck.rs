//! PhutKrub's icons in the line set: which of its names (`crate::deck::Icon::name`) the
//! shared line drawings stand for, and PhutKrub's own drawings in the same style (24-unit
//! grid, 1.45 stroke, pale gold surface) for the commands KhianKrub has no icon for:
//! slides, transitions, animations, slide show, masters, arrange, media, title bar and File.
//! Everything PhutKrub adds lives in this file, so work on one app never touches the other
//! app's file. Contributor-original artwork drawn in code, `MIT OR Apache-2.0` (AGENTS.md §1).

use egui::epaint::Mesh;
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

use super::Pen;

/// The line-set name drawn for `deck` icon `name`, when one stands for the same command.
pub(crate) fn line_name(name: &str) -> Option<&'static str> {
    Some(match name {
        "paste" => "paste",
        "cut" => "cut",
        "copy" => "copy",
        "format-painter" => "painter",
        "font-grow" => "grow",
        "font-shrink" => "shrink",
        "clear-formatting" => "clear",
        "bold" => "bold",
        "italic" => "italic",
        "underline" => "underline",
        "strikethrough" => "strike",
        "superscript" => "superscript",
        "subscript" => "subscript",
        "change-case" => "case",
        "highlight" => "highlight",
        "font-color" => "fontcolor",
        "bullets" => "bullets",
        "numbering" => "numbering",
        "decrease-indent" => "outdent",
        "increase-indent" => "indent",
        "line-spacing" => "lineSpacing",
        "columns" => "columns",
        "align-left" => "alignLeft",
        "align-center" => "alignCenter",
        "align-right" => "alignRight",
        "justify" => "justify",
        "text-direction" => "textDirection",
        "shapes" => "shapes",
        "table" => "table",
        "picture" => "picture",
        "icons" => "icons",
        "smart-art" => "smartArt",
        "chart" => "chart",
        "link" => "link",
        "text-box" => "textBox",
        "word-art" => "wordArt",
        "date-time" => "dateTime",
        "object" => "object",
        "equation" => "equation",
        "symbol" => "symbol",
        "bookmark" => "bookmark",
        "pen" => "pen",
        "pencil" => "pencil",
        "eraser" => "eraser",
        "lasso-select" => "lasso",
        "ink-to-shape" => "inkToShape",
        "ink-to-math" => "inkToMath",
        "ruler" => "ruler",
        "themes" => "themes",
        "colors" => "colors",
        "fonts" => "fonts",
        "spelling" => "spelling",
        "thesaurus" => "thesaurus",
        "accessibility" => "accessibility",
        "translate" => "translate",
        "language" => "language",
        "new-comment" => "newComment",
        "delete-comment" => "deleteComment",
        "prev-comment" => "prevComment",
        "next-comment" => "nextComment",
        "show-comments" => "showComments",
        "compare" => "compare",
        "view-outline" => "outline",
        "view-reading" => "readMode",
        "gridlines" => "gridlines",
        "zoom-glass" => "zoom",
        "zoom-in" => "zoomIn",
        "zoom-out" => "zoomOut",
        "new-window" => "newWindow",
        "macros" => "macros",
        "bring-forward" => "bringForward",
        "send-backward" => "sendBackward",
        "group" => "group",
        "align-objects" => "align",
        "selection-pane" => "selectionPane",
        "rotate" => "rotate",
        "crop" => "crop",
        "corrections" => "picCorrections",
        "color-adjust" => "picColor",
        "transparency" => "picTransparency",
        "change-picture" => "picChange",
        "reset-picture" => "picReset",
        "alt-text" => "altText",
        "insert-above" => "insertAbove",
        "insert-below" => "insertBelow",
        "insert-left" => "insertLeft",
        "insert-right" => "insertRight",
        "delete-table" => "deleteTable",
        "merge-cells" => "merge",
        "split-cells" => "splitCells",
        "borders" => "borders",
        "shading" => "shading",
        "search" => "find",
        // Same command (or the same idea) under another app's name.
        "comment" => "newComment",
        "comments-bubble" | "comments-small" => "showComments",
        "header-footer" => "header",
        "palette" => "colors",
        "save" => "optionsSave",
        "settings" => "optionsGeneral",
        "text-fill" => "fontcolor",
        "text-effects" => "effects",
        "shape-fill" => "shading",
        "shape-effects" => "effectsDesign",
        "highlighter-pen" => "highlight",
        "view-reading-small" => "readMode",
        _ => return None,
    })
}

/// Draw deck icon `name` in the line style: the shared drawing it is routed to, else
/// PhutKrub's own. `false` when neither knows the name (the caller keeps its older drawing).
pub(crate) fn paint(p: &Painter, r: Rect, name: &str, ink: Color32, disabled: bool, flat: bool) -> bool {
    let ink = if disabled { ink.gamma_multiply(0.35) } else { ink };
    if let Some(line) = line_name(name) {
        return super::paint_ex(p, r, line, ink, disabled, flat);
    }
    let size = r.width().min(r.height());
    if !size.is_finite() || size <= 0.5 || !r.center().is_finite() {
        // Nothing to draw, but still say whether the name is ours.
        let mut quiet = p.clone();
        quiet.set_invisible();
        return draw(&Pen { painter: &quiet, origin: Pos2::ZERO, scale: 1.0, ink, accent: ink }, name);
    }
    let accent = if disabled || flat || ink == Color32::WHITE { ink.gamma_multiply(0.35) } else { crate::gold(ink, crate::GoldRole::Surface, 1.0) };
    let pen = Pen { painter: p, origin: r.center() - vec2(size, size) / 2.0, scale: size / 24.0, ink, accent };
    draw(&pen, name)
}

// ----- small drawing helpers (all in grid units) -----

/// A closed outline with a gold surface (the polygon must be convex for the surface).
fn shape(pen: &Pen, points: &[(f32, f32)]) {
    let fill = points.iter().map(|&(x, y)| pen.point(x, y)).collect();
    pen.painter.add(Shape::convex_polygon(fill, pen.accent, Stroke::NONE));
    outline(pen, points);
}

/// A closed outline.
fn outline(pen: &Pen, points: &[(f32, f32)]) {
    let mut closed = points.to_vec();
    closed.extend(points.first().copied());
    pen.line(&closed);
}

/// A rectangle with a gold surface.
fn card(pen: &Pen, x: f32, y: f32, w: f32, h: f32) {
    pen.fill(x, y, w, h);
    pen.rect(x, y, w, h);
}

/// Points on an arc around (`cx`, `cy`); angles in degrees, clockwise on screen from 3 o'clock.
fn arc_points(cx: f32, cy: f32, radius: f32, from: f32, to: f32) -> Vec<(f32, f32)> {
    let steps = (((to - from).abs() / 15.0).ceil() as usize).max(2);
    (0..=steps)
        .map(|i| {
            let a = (from + (to - from) * i as f32 / steps as f32).to_radians();
            (cx + radius * a.cos(), cy + radius * a.sin())
        })
        .collect()
}

fn arc(pen: &Pen, cx: f32, cy: f32, radius: f32, from: f32, to: f32) {
    pen.line(&arc_points(cx, cy, radius, from, to));
}

/// An open arrowhead at `tip`, pointing away from `from`.
fn head(pen: &Pen, from: (f32, f32), tip: (f32, f32)) {
    let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / len, dy / len);
    let (l, w) = (2.6, 2.3);
    pen.line(&[(tip.0 - ux * l + uy * w, tip.1 - uy * l - ux * w), tip, (tip.0 - ux * l - uy * w, tip.1 - uy * l + ux * w)]);
}

/// A straight arrow from `a` to `b`.
fn arrow(pen: &Pen, a: (f32, f32), b: (f32, f32)) {
    pen.line(&[a, b]);
    head(pen, a, b);
}

/// An arc ending in an arrowhead.
fn arc_arrow(pen: &Pen, cx: f32, cy: f32, radius: f32, from: f32, to: f32) {
    let points = arc_points(cx, cy, radius, from, to);
    pen.line(&points);
    if let [.., before, tip] = points[..] {
        head(pen, before, tip);
    }
}

/// A short dashed line.
fn dashed(pen: &Pen, a: (f32, f32), b: (f32, f32)) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = (((dx * dx + dy * dy).sqrt() / 3.0).round() as usize).max(1);
    for i in (0..n).step_by(2) {
        let (t0, t1) = (i as f32 / n as f32, ((i + 1) as f32 / n as f32).min(1.0));
        pen.line(&[(a.0 + dx * t0, a.1 + dy * t0), (a.0 + dx * t1, a.1 + dy * t1)]);
    }
}

/// A four-point sparkle with a gold centre.
fn sparkle(pen: &Pen, cx: f32, cy: f32, outer: f32, inner: f32) {
    let points = [
        (cx, cy - outer),
        (cx + inner, cy - inner),
        (cx + outer, cy),
        (cx + inner, cy + inner),
        (cx, cy + outer),
        (cx - inner, cy + inner),
        (cx - outer, cy),
        (cx - inner, cy - inner),
    ];
    let mut mesh = Mesh::default();
    mesh.colored_vertex(pen.point(cx, cy), pen.accent);
    for &(x, y) in &points {
        mesh.colored_vertex(pen.point(x, y), pen.accent);
    }
    for i in 0..points.len() as u32 {
        mesh.add_triangle(0, i + 1, (i + 1) % points.len() as u32 + 1);
    }
    pen.painter.add(Shape::mesh(mesh));
    outline(pen, &points);
}

/// An almond eye with a gold iris.
fn eye(pen: &Pen, cx: f32, cy: f32, half: f32) {
    let h = half * 0.6;
    let top: Vec<(f32, f32)> =
        (0..=10).map(|i| (cx - half + 2.0 * half * i as f32 / 10.0, cy - h * (std::f32::consts::PI * i as f32 / 10.0).sin())).collect();
    let bottom: Vec<(f32, f32)> = top.iter().map(|&(x, y)| (x, 2.0 * cy - y)).collect();
    pen.line(&top);
    pen.line(&bottom);
    pen.painter.circle_filled(pen.point(cx, cy), h * 0.62 * pen.scale, pen.accent);
    pen.circle(cx, cy, h * 0.62);
}

/// A sheet of paper with a folded corner, from (`x0`, `y0`) to (`x1`, `y1`).
fn page(pen: &Pen, x0: f32, y0: f32, x1: f32, y1: f32) {
    let fold = 4.5;
    pen.line(&[(x1 - fold, y0), (x0, y0), (x0, y1), (x1, y1), (x1, y0 + fold), (x1 - fold, y0)]);
    pen.line(&[(x1 - fold, y0), (x1 - fold, y0 + fold), (x1, y0 + fold)]);
}

/// A small plus sign centred on (`x`, `y`).
fn plus(pen: &Pen, x: f32, y: f32, half: f32) {
    pen.line(&[(x - half, y), (x + half, y)]);
    pen.line(&[(x, y - half), (x, y + half)]);
}

/// A small cross centred on (`x`, `y`).
fn cross(pen: &Pen, x: f32, y: f32, half: f32) {
    pen.line(&[(x - half, y - half), (x + half, y + half)]);
    pen.line(&[(x + half, y - half), (x - half, y + half)]);
}

/// A speaker with `waves` sound arcs.
fn speaker(pen: &Pen, waves: usize) {
    shape(pen, &[(3.0, 9.0), (7.0, 9.0), (12.0, 4.5), (12.0, 19.5), (7.0, 15.0), (3.0, 15.0)]);
    for radius in [4.5, 8.0].into_iter().take(waves) {
        arc(pen, 12.0, 12.0, radius, -45.0, 45.0);
    }
}

/// A padlock; `open` lifts the shackle off its right post.
fn padlock(pen: &Pen, open: bool) {
    card(pen, 5.0, 11.0, 14.0, 10.0);
    if open {
        arc(pen, 12.0, 7.0, 4.5, 180.0, 360.0);
        pen.line(&[(7.5, 7.0), (7.5, 11.0)]);
    } else {
        arc(pen, 12.0, 8.0, 4.5, 180.0, 360.0);
        pen.line(&[(7.5, 8.0), (7.5, 11.0)]);
        pen.line(&[(16.5, 8.0), (16.5, 11.0)]);
    }
    pen.dot(12.0, 15.5, 1.2);
}

/// A projection screen on a stand.
fn screen(pen: &Pen) {
    pen.line(&[(2.5, 4.0), (21.5, 4.0)]);
    pen.rect(4.0, 4.0, 16.0, 11.0);
    pen.line(&[(12.0, 15.0), (12.0, 18.0)]);
    pen.line(&[(8.5, 21.0), (12.0, 18.0), (15.5, 21.0)]);
}

// One flat table of drawings, one arm per name: long, but each arm is simple.
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn draw(pen: &Pen, name: &str) -> bool {
    match name {
        // ----- Slides -----
        "new-slide" => {
            card(pen, 2.5, 7.0, 15.0, 11.0);
            pen.line(&[(5.5, 11.0), (14.5, 11.0)]);
            pen.line(&[(5.5, 14.5), (11.5, 14.5)]);
            plus(pen, 19.0, 5.0, 3.0);
        }
        "duplicate-slide" => {
            pen.line(&[(6.0, 14.0), (3.0, 14.0), (3.0, 4.0), (16.0, 4.0), (16.0, 7.0)]);
            card(pen, 8.0, 9.0, 13.0, 10.0);
            plus(pen, 14.5, 14.0, 2.5);
        }
        "slide" => {
            pen.rect(3.0, 5.0, 18.0, 14.0);
            pen.fill(5.0, 7.0, 14.0, 3.5);
            pen.line(&[(6.0, 14.0), (18.0, 14.0)]);
            pen.line(&[(6.0, 16.5), (14.0, 16.5)]);
        }
        "layout" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            card(pen, 5.0, 6.0, 14.0, 3.5);
            pen.rect(5.0, 12.0, 6.0, 6.0);
            pen.rect(13.0, 12.0, 6.0, 6.0);
        }
        "reset-slide" => {
            card(pen, 2.5, 9.0, 13.0, 10.0);
            arc_arrow(pen, 17.0, 7.5, 4.0, 160.0, 430.0);
        }
        "section" => {
            card(pen, 3.0, 3.0, 9.0, 4.0);
            pen.line(&[(5.0, 7.0), (5.0, 18.5), (8.0, 18.5)]);
            pen.line(&[(5.0, 12.0), (8.0, 12.0)]);
            pen.rect(8.0, 9.5, 13.0, 5.0);
            pen.rect(8.0, 16.0, 13.0, 5.0);
        }
        "delete-slide" => {
            card(pen, 2.5, 4.0, 15.0, 11.0);
            cross(pen, 18.0, 18.0, 3.0);
        }
        "hide-slide" => {
            card(pen, 3.0, 5.0, 18.0, 13.0);
            pen.line(&[(3.5, 21.0), (20.5, 2.5)]);
        }
        "slide-size" => {
            card(pen, 6.5, 8.0, 11.0, 8.0);
            for (x, y, dx, dy) in [(2.5, 4.0, 1.0, 1.0), (21.5, 4.0, -1.0, 1.0), (2.5, 20.0, 1.0, -1.0), (21.5, 20.0, -1.0, -1.0)] {
                pen.line(&[(x + 4.0 * dx, y), (x, y), (x, y + 4.0 * dy)]);
            }
        }
        "slide-number" => {
            pen.rect(3.0, 4.0, 18.0, 15.0);
            pen.fill(5.0, 6.0, 14.0, 3.5);
            pen.text(16.0, 14.5, 9.0, "#");
            pen.line(&[(6.0, 13.0), (11.0, 13.0)]);
            pen.line(&[(6.0, 16.0), (10.0, 16.0)]);
        }
        "variants" => {
            card(pen, 3.0, 3.0, 13.0, 9.0);
            pen.rect(8.0, 12.0, 13.0, 9.0);
            for x in [11.5, 14.5, 17.5] {
                pen.dot(x, 16.5, 1.1);
            }
        }
        "format-background" => {
            card(pen, 3.0, 4.0, 18.0, 14.0);
            pen.line(&[(6.0, 21.0), (18.0, 21.0)]);
            pen.line(&[(7.0, 14.5), (11.0, 9.5), (14.0, 12.5), (15.5, 11.0), (18.0, 14.5)]);
        }
        "background-styles" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            pen.fill(5.0, 6.0, 14.0, 3.0);
            pen.fill(5.0, 10.5, 10.0, 3.0);
            pen.fill(5.0, 15.0, 6.0, 3.0);
        }
        "design-ideas" => {
            pen.painter.circle_filled(pen.point(12.0, 9.5), 4.0 * pen.scale, pen.accent);
            arc(pen, 12.0, 9.5, 6.5, 135.0, 405.0);
            pen.line(&[(7.4, 14.1), (9.0, 17.0), (15.0, 17.0), (16.6, 14.1)]);
            pen.line(&[(9.5, 20.0), (14.5, 20.0)]);
            pen.line(&[(12.0, 9.5), (12.0, 13.5)]);
        }
        // ----- Transitions and animations -----
        "preview" => {
            pen.rect(2.5, 4.0, 19.0, 16.0);
            eye(pen, 12.0, 12.0, 6.5);
        }
        "effect-options" => {
            card(pen, 3.0, 3.0, 10.0, 10.0);
            arrow(pen, (14.0, 14.0), (20.5, 20.5));
            arrow(pen, (14.0, 8.0), (21.0, 8.0));
            arrow(pen, (8.0, 14.0), (8.0, 21.0));
        }
        "apply-to-all" => {
            pen.line(&[(8.5, 7.0), (8.5, 3.5), (21.5, 3.5), (21.5, 12.5), (18.5, 12.5)]);
            pen.line(&[(5.5, 10.0), (5.5, 6.5), (18.5, 6.5), (18.5, 15.5), (15.5, 15.5)]);
            card(pen, 2.5, 10.0, 13.0, 9.5);
        }
        "sound" | "audio" | "volume" => speaker(pen, 2),
        "duration" => {
            pen.circle(12.0, 13.0, 8.0);
            pen.line(&[(12.0, 8.5), (12.0, 13.0), (15.5, 15.0)]);
            pen.line(&[(9.5, 2.5), (14.5, 2.5)]);
        }
        "anim-entrance" => {
            card(pen, 11.0, 4.0, 10.0, 16.0);
            arrow(pen, (2.5, 12.0), (15.5, 12.0));
        }
        "anim-exit" => {
            card(pen, 3.0, 4.0, 10.0, 16.0);
            arrow(pen, (8.5, 12.0), (21.5, 12.0));
        }
        "anim-emphasis" => {
            sparkle(pen, 12.0, 12.0, 9.0, 2.6);
        }
        "anim-path" => {
            pen.painter.circle_filled(pen.point(4.5, 19.0), 2.2 * pen.scale, pen.accent);
            pen.circle(4.5, 19.0, 2.2);
            let curve: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    let u = 1.0 - t;
                    (u * u * 6.5 + 2.0 * u * t * 8.0 + t * t * 19.5, u * u * 17.0 + 2.0 * u * t * 4.0 + t * t * 5.0)
                })
                .collect();
            for pair in curve.chunks(2) {
                if let [a, b] = pair {
                    pen.line(&[*a, *b]);
                }
            }
            head(pen, (16.0, 4.6), (20.5, 5.2));
        }
        "animation-pane" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.line(&[(13.0, 3.0), (13.0, 21.0)]);
            sparkle(pen, 8.0, 12.0, 3.8, 1.1);
            for y in [7.5, 12.0, 16.5] {
                pen.fill(15.0, y - 1.25, 4.5, 2.5);
            }
        }
        "trigger" => {
            pen.fill(5.0, 5.0, 6.0, 10.0);
            pen.line(&[(5.0, 3.0), (5.0, 18.0), (8.5, 14.5), (11.5, 21.0), (14.0, 20.0), (11.0, 13.5), (16.0, 13.5), (5.0, 3.0)]);
            sparkle(pen, 18.5, 6.0, 3.5, 1.0);
        }
        "animation-painter" => {
            card(pen, 3.5, 3.0, 12.5, 4.5);
            pen.line(&[(16.0, 5.0), (17.5, 5.0), (17.5, 9.5), (10.0, 9.5), (10.0, 11.5)]);
            pen.rect(8.3, 11.5, 3.4, 8.0);
            sparkle(pen, 18.0, 17.0, 4.0, 1.2);
        }
        "add-animation" => {
            sparkle(pen, 10.0, 13.0, 7.5, 2.2);
            plus(pen, 19.0, 5.0, 3.0);
        }
        // ----- Slide show and record -----
        "play" => {
            shape(pen, &[(7.0, 4.0), (19.5, 12.0), (7.0, 20.0)]);
        }
        "pause" => {
            card(pen, 6.0, 5.0, 4.0, 14.0);
            card(pen, 14.0, 5.0, 4.0, 14.0);
        }
        "stop" => card(pen, 6.0, 6.0, 12.0, 12.0),
        "play-from-start" => {
            pen.line(&[(5.0, 5.0), (5.0, 19.0)]);
            shape(pen, &[(9.0, 5.0), (19.5, 12.0), (9.0, 19.0)]);
        }
        "play-from-current" => {
            pen.rect(2.5, 4.0, 15.0, 11.0);
            pen.line(&[(5.0, 8.0), (12.0, 8.0)]);
            shape(pen, &[(12.5, 11.5), (21.0, 16.5), (12.5, 21.5)]);
        }
        "presenter-view" => {
            pen.rect(2.0, 4.0, 13.0, 9.5);
            pen.fill(4.0, 6.0, 9.0, 3.0);
            pen.line(&[(8.5, 13.5), (8.5, 17.0)]);
            pen.line(&[(5.5, 17.0), (11.5, 17.0)]);
            pen.circle(18.5, 8.5, 2.5);
            pen.line(&[(14.5, 20.5), (15.0, 16.0), (17.0, 14.0), (20.0, 14.0), (22.0, 16.0), (22.5, 20.5)]);
        }
        "custom-show" => {
            card(pen, 3.0, 4.0, 8.0, 6.0);
            pen.rect(3.0, 14.0, 8.0, 6.0);
            pen.line(&[(14.0, 6.0), (21.0, 6.0)]);
            pen.line(&[(14.0, 8.5), (19.0, 8.5)]);
            pen.line(&[(14.0, 16.0), (21.0, 16.0)]);
            pen.line(&[(14.0, 18.5), (19.0, 18.5)]);
        }
        "set-up-show" => {
            screen(pen);
            pen.fill(6.0, 6.0, 12.0, 3.0);
            pen.line(&[(6.5, 11.5), (17.5, 11.5)]);
        }
        "view-show-small" => {
            screen(pen);
            shape(pen, &[(10.0, 6.5), (15.0, 9.5), (10.0, 12.5)]);
        }
        "rehearse-timings" => {
            pen.painter.circle_filled(pen.point(12.0, 13.5), 4.0 * pen.scale, pen.accent);
            pen.circle(12.0, 13.5, 7.5);
            pen.line(&[(12.0, 13.5), (12.0, 9.5)]);
            pen.line(&[(9.5, 3.0), (14.5, 3.0)]);
            pen.line(&[(12.0, 3.0), (12.0, 6.0)]);
            pen.line(&[(18.0, 6.5), (19.5, 5.0)]);
        }
        "record" | "screen-recording" => {
            pen.circle(12.0, 12.0, 8.5);
            pen.painter.circle_filled(pen.point(12.0, 12.0), 4.5 * pen.scale, pen.accent);
            pen.circle(12.0, 12.0, 4.5);
        }
        "cameo" => {
            pen.circle(12.0, 12.0, 9.0);
            pen.painter.circle_filled(pen.point(12.0, 10.0), 3.0 * pen.scale, pen.accent);
            pen.circle(12.0, 10.0, 3.0);
            pen.line(&[(6.5, 18.5), (8.5, 15.5), (15.5, 15.5), (17.5, 18.5)]);
        }
        "subtitles" => {
            pen.rect(3.0, 5.0, 18.0, 14.0);
            pen.fill(5.0, 13.0, 14.0, 4.0);
            pen.line(&[(6.5, 15.0), (11.0, 15.0)]);
            pen.line(&[(13.0, 15.0), (17.5, 15.0)]);
        }
        "coach" => {
            pen.circle(9.0, 7.0, 3.0);
            pen.line(&[(3.0, 20.0), (4.0, 14.0), (6.5, 11.5), (11.5, 11.5), (14.0, 14.0), (15.0, 20.0)]);
            pen.fill(16.0, 4.0, 6.0, 5.0);
            pen.line(&[(15.5, 3.5), (22.0, 3.5), (22.0, 9.5), (18.0, 9.5), (16.5, 11.5), (16.5, 9.5), (15.5, 9.5), (15.5, 3.5)]);
        }
        // ----- Insert -----
        "screenshot" => {
            pen.rect(3.0, 4.0, 18.0, 15.0);
            pen.line(&[(3.0, 8.0), (21.0, 8.0)]);
            pen.dot(5.5, 6.0, 0.8);
            pen.dot(8.0, 6.0, 0.8);
            card(pen, 7.0, 11.0, 9.0, 5.5);
        }
        "video" => {
            card(pen, 2.5, 6.5, 13.0, 11.0);
            pen.line(&[(15.5, 10.5), (21.0, 7.0), (21.0, 17.0), (15.5, 13.5)]);
        }
        "zoom" => {
            card(pen, 3.0, 3.0, 13.0, 9.0);
            pen.circle(15.0, 15.5, 4.5);
            pen.line(&[(18.3, 18.8), (21.5, 22.0)]);
        }
        "action" => {
            card(pen, 2.5, 4.0, 15.0, 9.0);
            pen.line(&[(12.0, 9.5), (12.0, 21.0), (14.5, 18.5), (16.5, 22.5), (18.5, 21.5), (16.5, 17.5), (20.0, 17.5), (12.0, 9.5)]);
        }
        "models-3d" => {
            shape(pen, &[(12.0, 3.0), (20.0, 7.5), (20.0, 16.5), (12.0, 21.0), (4.0, 16.5), (4.0, 7.5)]);
            pen.line(&[(4.0, 7.5), (12.0, 12.0), (20.0, 7.5)]);
            pen.line(&[(12.0, 12.0), (12.0, 21.0)]);
        }
        "char-spacing" => {
            pen.line(&[(3.0, 14.0), (6.5, 4.0), (10.0, 14.0)]);
            pen.line(&[(4.3, 10.5), (8.7, 10.5)]);
            pen.line(&[(13.0, 4.0), (16.5, 14.0), (20.0, 4.0)]);
            arrow(pen, (12.0, 19.0), (3.0, 19.0));
            arrow(pen, (12.0, 19.0), (21.0, 19.0));
        }
        "text-shadow" => {
            let font = egui::FontId::proportional(17.0 * pen.scale);
            pen.painter.text(pen.point(13.5, 13.5), egui::Align2::CENTER_CENTER, "S", font, pen.accent);
            pen.text(11.5, 11.5, 17.0, "S");
        }
        "align-text" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            pen.fill(5.0, 9.5, 14.0, 5.0);
            pen.line(&[(7.0, 10.5), (17.0, 10.5)]);
            pen.line(&[(8.5, 13.5), (15.5, 13.5)]);
            pen.line(&[(10.0, 5.5), (12.0, 7.5), (14.0, 5.5)]);
            pen.line(&[(10.0, 18.5), (12.0, 16.5), (14.0, 18.5)]);
        }
        "convert-smart-art" => {
            pen.line(&[(4.0, 5.0), (11.0, 5.0)]);
            pen.line(&[(4.0, 8.5), (9.0, 8.5)]);
            arrow(pen, (6.0, 12.0), (11.0, 17.0));
            card(pen, 13.0, 4.0, 8.0, 5.0);
            pen.rect(13.0, 15.0, 8.0, 5.0);
            pen.line(&[(17.0, 9.0), (17.0, 15.0)]);
        }
        // ----- Drawing and shapes -----
        "arrange" => {
            card(pen, 3.0, 3.0, 8.0, 8.0);
            pen.rect(8.0, 8.0, 8.0, 8.0);
            pen.rect(13.0, 13.0, 8.0, 8.0);
        }
        "quick-styles" => {
            card(pen, 3.0, 4.0, 18.0, 13.0);
            pen.text(12.0, 10.5, 10.0, "Aa");
            pen.line(&[(5.0, 20.5), (19.0, 20.5)]);
        }
        "shape-outline" => {
            pen.rect(4.5, 3.5, 15.0, 12.0);
            pen.line(&[(7.5, 12.5), (16.5, 6.5)]);
            pen.fill(3.0, 19.0, 18.0, 2.5);
        }
        "text-outline" => {
            outline(pen, &[(4.0, 17.0), (10.5, 3.0), (13.5, 3.0), (20.0, 17.0), (16.8, 17.0), (15.3, 13.5), (8.7, 13.5), (7.2, 17.0)]);
            outline(pen, &[(9.9, 10.5), (14.1, 10.5), (12.0, 5.8)]);
            pen.fill(3.0, 19.5, 18.0, 2.0);
        }
        "edit-shape" => {
            let points = [(4.0, 18.0), (4.0, 8.0), (12.0, 3.5), (20.0, 8.0), (17.0, 19.5)];
            outline(pen, &points);
            for (x, y) in points {
                card(pen, x - 1.6, y - 1.6, 3.2, 3.2);
            }
        }
        "merge-shapes" => {
            pen.painter.circle_filled(pen.point(9.0, 12.0), 6.5 * pen.scale, pen.accent);
            pen.circle(9.0, 12.0, 6.5);
            pen.circle(15.0, 12.0, 6.5);
        }
        "bring-to-front" | "send-to-back" => {
            let front = name == "bring-to-front";
            for (x, y) in [(3.0, 3.0), (13.0, 13.0)] {
                if !front {
                    pen.fill(x, y, 8.0, 8.0);
                }
                pen.rect(x, y, 8.0, 8.0);
            }
            if front {
                card(pen, 7.0, 7.0, 10.0, 10.0);
            } else {
                pen.rect(7.0, 7.0, 10.0, 10.0);
            }
        }
        "ungroup" => {
            card(pen, 5.0, 5.0, 7.0, 7.0);
            pen.circle(16.0, 16.0, 3.5);
            pen.line(&[(2.5, 7.0), (2.5, 2.5), (7.0, 2.5)]);
            pen.line(&[(21.5, 17.0), (21.5, 21.5), (17.0, 21.5)]);
        }
        "align-obj-left" | "align-obj-center" | "align-obj-right" => {
            let (x, a, b) = match name {
                "align-obj-left" => (4.0, 6.0, 6.0),
                "align-obj-center" => (12.0, 5.0, 8.0),
                _ => (20.0, 6.0, 11.0),
            };
            pen.line(&[(x, 3.0), (x, 21.0)]);
            card(pen, a, 5.0, if name == "align-obj-center" { 14.0 } else { 12.0 }, 5.0);
            pen.rect(b, 14.0, if name == "align-obj-center" { 8.0 } else { 7.0 }, 5.0);
        }
        "align-obj-top" | "align-obj-middle" | "align-obj-bottom" => {
            let (y, a, b) = match name {
                "align-obj-top" => (4.0, 6.0, 6.0),
                "align-obj-middle" => (12.0, 5.0, 8.0),
                _ => (20.0, 6.0, 11.0),
            };
            pen.line(&[(3.0, y), (21.0, y)]);
            card(pen, 5.0, a, 5.0, if name == "align-obj-middle" { 14.0 } else { 12.0 });
            pen.rect(14.0, b, 5.0, if name == "align-obj-middle" { 8.0 } else { 7.0 });
        }
        "distribute-h" => {
            pen.line(&[(3.0, 4.0), (3.0, 20.0)]);
            pen.line(&[(21.0, 4.0), (21.0, 20.0)]);
            card(pen, 6.5, 6.0, 4.0, 12.0);
            pen.rect(13.5, 8.0, 4.0, 8.0);
        }
        "distribute-v" => {
            pen.line(&[(4.0, 3.0), (20.0, 3.0)]);
            pen.line(&[(4.0, 21.0), (20.0, 21.0)]);
            card(pen, 6.0, 6.5, 12.0, 4.0);
            pen.rect(8.0, 13.5, 8.0, 4.0);
        }
        "rotate-right" => {
            card(pen, 4.0, 11.0, 10.0, 9.0);
            arc_arrow(pen, 12.0, 11.0, 7.0, 200.0, 335.0);
        }
        "rotate-left" => {
            card(pen, 10.0, 11.0, 10.0, 9.0);
            arc_arrow(pen, 12.0, 11.0, 7.0, 340.0, 205.0);
        }
        "flip-h" => {
            dashed(pen, (12.0, 2.5), (12.0, 21.5));
            shape(pen, &[(9.5, 6.0), (9.5, 18.0), (3.0, 18.0)]);
            outline(pen, &[(14.5, 6.0), (14.5, 18.0), (21.0, 18.0)]);
        }
        "flip-v" => {
            dashed(pen, (2.5, 12.0), (21.5, 12.0));
            shape(pen, &[(6.0, 9.5), (18.0, 9.5), (18.0, 3.0)]);
            outline(pen, &[(6.0, 14.5), (18.0, 14.5), (18.0, 21.0)]);
        }
        // ----- Picture and media -----
        "artistic-effects" => {
            pen.rect(3.0, 3.0, 15.0, 13.0);
            pen.line(&[(4.0, 14.0), (8.0, 9.0), (11.0, 12.0), (13.0, 10.0), (17.0, 14.0)]);
            pen.circle(13.5, 6.5, 1.2);
            pen.line(&[(15.5, 18.0), (21.0, 12.5)]);
            shape(pen, &[(15.5, 18.0), (13.0, 21.5), (11.5, 20.0), (13.5, 16.5)]);
        }
        "compress-pictures" => {
            card(pen, 7.5, 7.5, 9.0, 9.0);
            arrow(pen, (2.5, 2.5), (6.0, 6.0));
            arrow(pen, (21.5, 21.5), (18.0, 18.0));
            arrow(pen, (21.5, 2.5), (18.0, 6.0));
            arrow(pen, (2.5, 21.5), (6.0, 18.0));
        }
        "remove-background" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            for (x, y) in [(4.5, 4.5), (8.5, 4.5), (16.0, 4.5), (4.5, 8.5), (16.0, 8.5)] {
                pen.fill(x, y, 3.5, 3.5);
            }
            pen.circle(12.0, 10.0, 3.0);
            pen.line(&[(6.0, 21.0), (7.0, 16.5), (9.5, 14.5), (14.5, 14.5), (17.0, 16.5), (18.0, 21.0)]);
        }
        "trim-media" => {
            pen.line(&[(2.5, 12.0), (21.5, 12.0)]);
            card(pen, 7.0, 8.5, 10.0, 7.0);
            pen.line(&[(7.0, 5.0), (7.0, 19.0)]);
            pen.line(&[(17.0, 5.0), (17.0, 19.0)]);
        }
        "loop" => {
            arc_arrow(pen, 12.0, 12.0, 7.5, 200.0, 520.0);
        }
        "image" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            pen.fill(4.5, 13.0, 15.0, 5.5);
            pen.line(&[(4.0, 18.0), (9.0, 12.0), (13.0, 16.0), (15.5, 13.5), (20.0, 18.0)]);
            pen.circle(15.5, 8.5, 1.5);
        }
        // ----- Chart -----
        "change-chart-type" => {
            card(pen, 4.0, 13.0, 3.0, 7.0);
            pen.rect(9.0, 8.0, 3.0, 12.0);
            pen.line(&[(2.5, 20.0), (14.0, 20.0)]);
            arc_arrow(pen, 18.0, 9.0, 3.5, 120.0, 400.0);
        }
        "edit-data" => {
            pen.rect(3.0, 3.0, 14.0, 14.0);
            pen.fill(4.0, 4.0, 12.0, 3.5);
            pen.line(&[(3.0, 8.0), (17.0, 8.0)]);
            pen.line(&[(3.0, 12.5), (17.0, 12.5)]);
            pen.line(&[(10.0, 3.0), (10.0, 17.0)]);
            shape(pen, &[(12.0, 21.0), (13.0, 17.0), (19.0, 11.0), (21.0, 13.0), (15.0, 19.0)]);
        }
        "chart-elements" => {
            pen.line(&[(3.0, 3.0), (3.0, 20.0), (15.0, 20.0)]);
            card(pen, 6.0, 12.0, 3.0, 8.0);
            pen.rect(11.0, 8.0, 3.0, 12.0);
            plus(pen, 19.0, 6.0, 3.0);
        }
        "chart-styles" => {
            pen.line(&[(3.0, 3.0), (3.0, 20.0), (21.0, 20.0)]);
            card(pen, 6.0, 11.0, 3.5, 9.0);
            card(pen, 11.5, 6.0, 3.5, 14.0);
            pen.rect(17.0, 13.0, 3.5, 7.0);
        }
        "switch-row-col" => {
            pen.rect(3.0, 3.0, 11.0, 11.0);
            pen.fill(4.0, 4.0, 9.0, 3.0);
            pen.line(&[(3.0, 7.0), (14.0, 7.0)]);
            arc_arrow(pen, 14.0, 14.0, 6.5, 0.0, 90.0);
            pen.line(&[(20.5, 14.0), (20.5, 17.0)]);
        }
        // ----- Draw -----
        "ink-to-text" => {
            pen.line(&[(3.0, 9.0), (5.0, 4.5), (7.0, 9.0), (9.0, 4.5), (11.0, 9.0)]);
            arrow(pen, (6.0, 12.5), (10.5, 17.0));
            pen.fill(12.5, 12.0, 9.0, 9.0);
            pen.text(17.0, 16.5, 10.0, "A");
        }
        // ----- View -----
        "view-normal" | "view-normal-small" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            pen.line(&[(8.0, 4.0), (8.0, 20.0)]);
            card(pen, 10.0, 6.0, 9.0, 8.0);
            pen.line(&[(10.0, 17.0), (19.0, 17.0)]);
        }
        "view-sorter" | "view-sorter-small" | "grid" => {
            card(pen, 3.0, 4.0, 8.0, 6.5);
            pen.rect(13.0, 4.0, 8.0, 6.5);
            pen.rect(3.0, 13.5, 8.0, 6.5);
            pen.rect(13.0, 13.5, 8.0, 6.5);
        }
        "view-notes-page" | "notes-master" => {
            pen.rect(5.0, 2.5, 14.0, 19.0);
            card(pen, 7.5, 5.0, 9.0, 6.5);
            pen.line(&[(7.5, 14.5), (16.5, 14.5)]);
            pen.line(&[(7.5, 17.5), (14.0, 17.5)]);
            if name == "notes-master" {
                pen.line(&[(3.0, 5.0), (3.0, 21.0)]);
            }
        }
        "slide-master" => {
            pen.rect(3.0, 3.0, 18.0, 9.0);
            pen.fill(5.0, 5.0, 14.0, 2.5);
            pen.line(&[(6.0, 9.5), (15.0, 9.5)]);
            pen.line(&[(7.0, 12.0), (7.0, 15.0)]);
            pen.line(&[(17.0, 12.0), (17.0, 15.0)]);
            card(pen, 3.0, 15.0, 8.0, 6.0);
            pen.rect(13.0, 15.0, 8.0, 6.0);
        }
        "handout-master" => {
            pen.rect(4.0, 2.5, 16.0, 19.0);
            for (i, y) in [5.0, 10.0, 15.0].into_iter().enumerate() {
                if i == 0 {
                    card(pen, 6.5, y, 5.0, 3.5);
                } else {
                    pen.rect(6.5, y, 5.0, 3.5);
                }
                pen.line(&[(13.5, y + 1.0), (17.5, y + 1.0)]);
                pen.line(&[(13.5, y + 3.0), (16.0, y + 3.0)]);
            }
        }
        "notes" | "notes-small" => {
            card(pen, 3.0, 3.0, 18.0, 11.0);
            pen.line(&[(3.0, 17.5), (21.0, 17.5)]);
            pen.line(&[(3.0, 21.0), (15.0, 21.0)]);
        }
        "guides" => {
            pen.rect(3.0, 3.0, 18.0, 18.0);
            dashed(pen, (12.0, 4.5), (12.0, 19.5));
            dashed(pen, (4.5, 12.0), (19.5, 12.0));
        }
        "grayscale" => {
            let half: Vec<Pos2> = arc_points(12.0, 12.0, 8.5, 90.0, 270.0).into_iter().map(|(x, y)| pen.point(x, y)).collect();
            pen.painter.add(Shape::convex_polygon(half, pen.accent, Stroke::NONE));
            pen.circle(12.0, 12.0, 8.5);
            pen.line(&[(12.0, 3.5), (12.0, 20.5)]);
        }
        "fit-to-window" | "fit-small" => {
            pen.rect(3.0, 4.0, 18.0, 16.0);
            card(pen, 8.0, 9.0, 8.0, 6.0);
            for (x, y, dx, dy) in [(5.5, 6.5, 1.0, 1.0), (18.5, 6.5, -1.0, 1.0), (5.5, 17.5, 1.0, -1.0), (18.5, 17.5, -1.0, -1.0)] {
                pen.line(&[(x, y), (x + 1.5 * dx, y + 1.5 * dy)]);
            }
        }
        "sparkle" => {
            sparkle(pen, 10.5, 13.5, 8.0, 2.4);
            sparkle(pen, 19.0, 5.0, 3.2, 1.0);
        }
        "star" => {
            let star: Vec<(f32, f32)> = (0..10)
                .map(|i| {
                    let radius = if i % 2 == 0 { 9.0 } else { 3.8 };
                    let a = (-90.0 + 36.0 * i as f32).to_radians();
                    (12.0 + radius * a.cos(), 12.5 + radius * a.sin())
                })
                .collect();
            outline(pen, &star);
        }
        // ----- Title bar, panes and File -----
        "home" => {
            pen.line(&[(3.0, 11.5), (12.0, 3.5), (21.0, 11.5)]);
            pen.line(&[(5.5, 9.5), (5.5, 20.5), (18.5, 20.5), (18.5, 9.5)]);
            card(pen, 10.0, 14.0, 4.0, 6.5);
        }
        "file" => {
            pen.fill(8.0, 11.0, 8.0, 6.0);
            page(pen, 5.0, 2.5, 19.0, 21.5);
            pen.line(&[(8.0, 12.0), (16.0, 12.0)]);
            pen.line(&[(8.0, 15.5), (14.0, 15.5)]);
        }
        "new-file" => {
            page(pen, 4.0, 2.5, 17.0, 21.5);
            pen.line(&[(7.0, 11.0), (14.0, 11.0)]);
            pen.line(&[(7.0, 14.5), (12.0, 14.5)]);
            pen.painter.circle_filled(pen.point(18.0, 18.0), 4.2 * pen.scale, pen.accent);
            plus(pen, 18.0, 18.0, 2.5);
        }
        "folder" => {
            pen.fill(4.0, 9.0, 16.0, 9.5);
            pen.line(&[(3.0, 5.0), (9.0, 5.0), (11.0, 7.5), (21.0, 7.5), (21.0, 19.5), (3.0, 19.5), (3.0, 5.0)]);
        }
        "open-folder" => {
            pen.line(&[(3.0, 19.0), (3.0, 5.0), (9.0, 5.0), (11.0, 7.5), (19.0, 7.5), (19.0, 10.0)]);
            shape(pen, &[(3.0, 19.0), (6.5, 10.0), (22.0, 10.0), (18.5, 19.0)]);
        }
        "save-as" => {
            pen.line(&[(11.0, 20.0), (3.0, 20.0), (3.0, 3.0), (14.0, 3.0), (17.0, 6.0), (17.0, 10.0)]);
            card(pen, 6.0, 3.5, 7.0, 4.0);
            pen.line(&[(6.0, 14.0), (6.0, 11.0), (12.0, 11.0)]);
            shape(pen, &[(12.0, 21.5), (13.0, 17.5), (19.0, 11.5), (21.0, 13.5), (15.0, 19.5)]);
        }
        "recent" => {
            arc_arrow(pen, 12.0, 12.0, 8.5, 150.0, -180.0);
            pen.line(&[(12.0, 7.5), (12.0, 12.0), (15.5, 14.0)]);
        }
        "back-circle" => {
            pen.circle(12.0, 12.0, 9.0);
            arrow(pen, (16.5, 12.0), (7.5, 12.0));
        }
        "export" | "share" => {
            pen.fill(5.0, 15.0, 14.0, 4.5);
            pen.line(&[(4.0, 12.0), (4.0, 20.5), (20.0, 20.5), (20.0, 12.0)]);
            arrow(pen, (12.0, 16.0), (12.0, 3.5));
        }
        "print" => {
            card(pen, 7.0, 3.0, 10.0, 5.5);
            pen.line(&[(7.0, 17.0), (3.5, 17.0), (3.5, 8.5), (20.5, 8.5), (20.5, 17.0), (17.0, 17.0)]);
            pen.rect(7.0, 13.5, 10.0, 7.5);
            pen.line(&[(9.0, 17.0), (15.0, 17.0)]);
            pen.dot(17.5, 11.5, 0.9);
        }
        "undo" | "redo" => {
            let m = |x: f32| if name == "undo" { x } else { 24.0 - x };
            let curve = [(19.5, 19.0), (19.5, 14.0), (18.0, 11.0), (15.0, 9.3), (5.0, 9.3)];
            let curve: Vec<(f32, f32)> = curve.iter().map(|&(x, y)| (m(x), y)).collect();
            pen.line(&curve);
            pen.line(&[(m(9.0), 5.3), (m(5.0), 9.3), (m(9.0), 13.3)]);
        }
        "more" => {
            for x in [6.0, 12.0, 18.0] {
                pen.dot(x, 12.0, 1.5);
            }
        }
        "drag-handle" => {
            for (x, y) in [(9.0, 6.0), (15.0, 6.0), (9.0, 12.0), (15.0, 12.0), (9.0, 18.0), (15.0, 18.0)] {
                pen.dot(x, y, 1.3);
            }
        }
        "info" | "help" | "warning" => {
            if name == "warning" {
                shape(pen, &[(12.0, 3.0), (21.5, 20.0), (2.5, 20.0)]);
                pen.line(&[(12.0, 9.0), (12.0, 14.0)]);
                pen.dot(12.0, 17.0, 1.1);
            } else {
                pen.painter.circle_filled(pen.point(12.0, 12.0), 9.0 * pen.scale, pen.accent);
                pen.circle(12.0, 12.0, 9.0);
                if name == "info" {
                    pen.dot(12.0, 7.5, 1.2);
                    pen.line(&[(12.0, 10.5), (12.0, 17.0)]);
                } else {
                    pen.line(&[(9.0, 9.5), (9.5, 7.3), (12.0, 6.3), (14.5, 7.3), (15.0, 9.5), (13.8, 11.2), (12.0, 12.3), (12.0, 14.0)]);
                    pen.dot(12.0, 17.2, 1.2);
                }
            }
        }
        "lock" | "read-only" => padlock(pen, false),
        "unlock" => padlock(pen, true),
        "eye" => eye(pen, 12.0, 12.0, 9.5),
        "eye-off" => {
            eye(pen, 12.0, 12.0, 9.5);
            pen.line(&[(4.0, 20.0), (20.0, 4.0)]);
        }
        "check" => pen.line(&[(4.0, 12.5), (9.5, 18.0), (20.0, 6.0)]),
        "close" => cross(pen, 12.0, 12.0, 6.5),
        "plus" => plus(pen, 12.0, 12.0, 7.0),
        "minus" => pen.line(&[(5.0, 12.0), (19.0, 12.0)]),
        "chevron-down" => pen.line(&[(6.0, 9.0), (12.0, 15.0), (18.0, 9.0)]),
        "chevron-up" => pen.line(&[(6.0, 15.0), (12.0, 9.0), (18.0, 15.0)]),
        "chevron-left" => pen.line(&[(15.0, 6.0), (9.0, 12.0), (15.0, 18.0)]),
        "chevron-right" => pen.line(&[(9.0, 6.0), (15.0, 12.0), (9.0, 18.0)]),
        "collapse" => {
            pen.line(&[(6.0, 4.0), (12.0, 9.0), (18.0, 4.0)]);
            pen.line(&[(6.0, 20.0), (12.0, 15.0), (18.0, 20.0)]);
        }
        "expand" => {
            pen.line(&[(6.0, 9.0), (12.0, 4.0), (18.0, 9.0)]);
            pen.line(&[(6.0, 15.0), (12.0, 20.0), (18.0, 15.0)]);
        }
        "list" => {
            for y in [6.0, 12.0, 18.0] {
                pen.dot(4.5, y, 1.1);
                pen.line(&[(8.0, y), (20.0, y)]);
            }
        }
        "pin" => {
            shape(pen, &[(9.0, 3.0), (15.0, 3.0), (15.0, 10.0), (18.0, 14.0), (6.0, 14.0), (9.0, 10.0)]);
            pen.line(&[(12.0, 14.0), (12.0, 21.0)]);
        }
        "account" => {
            pen.painter.circle_filled(pen.point(12.0, 8.5), 4.0 * pen.scale, pen.accent);
            pen.circle(12.0, 8.5, 4.0);
            pen.line(&[(4.0, 21.0), (5.0, 16.5), (8.0, 14.0), (16.0, 14.0), (19.0, 16.5), (20.0, 21.0)]);
        }
        _ => return false,
    }
    true
}
