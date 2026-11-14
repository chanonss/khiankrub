use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

pub fn paint(p: &Painter, r: Rect, name: &str, c: Color32, _accent: Color32) {
    if clipboard_pilot::paint(p, r, name, c) {
        return;
    }
    if hf::paint(p, r, name, c, c == Color32::WHITE) {
        return;
    }
    krub_icons::paint_named(p, r, name, c, c == Color32::WHITE, false);
}

/// Original Home ribbon artwork on a 24-unit grid, drawn at the requested display size.
mod clipboard_pilot {
    use egui::{Color32, Painter, Rect};

    /// The shared line set (`krub_icons::line`); disabled when drawn in the disabled text colour.
    pub fn paint(p: &Painter, r: Rect, name: &str, ink: Color32) -> bool {
        let disabled = ink == crate::theme::Tokens::get(p.ctx()).text_disabled;
        krub_icons::line::paint(p, r, name, ink, disabled)
    }
}
/// Diagrams used by the Home Borders menu. Selected edges are solid; other edges
/// remain light solid guides so every border option is distinguishable.
pub(crate) mod home_menu {
    use egui::{Color32, Painter, Rect, Stroke, vec2};

    pub fn paint(p: &Painter, r: Rect, name: &str, ink: Color32) -> bool {
        if matches!(
            name,
            "borders" | "drawTable" | "table" | "textToTable" | "pn_top" | "pn_bottom" | "pn_xofy" | "pn_current" | "pn_format" | "pn_position"
        ) {
            return super::clipboard_pilot::paint(p, r, name, ink);
        }
        let Some(kind) = name.strip_prefix("tb_").or(match name {
            "hline" | "gridlines" => Some(name),
            _ => None,
        }) else {
            return false;
        };
        if !matches!(
            kind,
            "bottom"
                | "top"
                | "left"
                | "right"
                | "none"
                | "all"
                | "outside"
                | "inside"
                | "insideH"
                | "insideV"
                | "diagDown"
                | "diagUp"
                | "hline"
                | "gridlines"
        ) {
            return false;
        }
        let size = r.width().min(r.height());
        if !size.is_finite() || size <= 0.5 || !r.center().is_finite() {
            return true;
        }
        let scale = size / 24.0;
        let origin = r.center() - vec2(size, size) * 0.5;
        let pt = |x: f32, y: f32| origin + vec2(x, y) * scale;
        let solid = Stroke::new((1.45 * scale).max(0.85), ink);
        let guide = Stroke::new((1.0 * scale).max(0.8), ink.gamma_multiply(0.42));
        let edge = |a: (f32, f32), b: (f32, f32), selected: bool| {
            let points = [pt(a.0, a.1), pt(b.0, b.1)];
            if selected {
                p.line_segment(points, solid);
            } else {
                p.line_segment(points, guide);
            }
        };
        if kind == "hline" {
            for y in [5.0, 8.0, 16.0, 19.0] {
                p.line_segment([pt(5.0, y), pt(19.0, y)], guide);
            }
            p.line_segment([pt(3.0, 12.0), pt(21.0, 12.0)], solid);
            return true;
        }
        let outer = matches!(kind, "all" | "outside");
        let inner = matches!(kind, "all" | "inside");
        edge((4.0, 4.0), (20.0, 4.0), outer || kind == "top");
        edge((4.0, 20.0), (20.0, 20.0), outer || kind == "bottom");
        edge((4.0, 4.0), (4.0, 20.0), outer || kind == "left");
        edge((20.0, 4.0), (20.0, 20.0), outer || kind == "right");
        edge((4.0, 12.0), (20.0, 12.0), inner || kind == "insideH");
        edge((12.0, 4.0), (12.0, 20.0), inner || kind == "insideV");
        if kind == "diagDown" {
            edge((4.0, 4.0), (20.0, 20.0), true);
        }
        if kind == "diagUp" {
            edge((4.0, 20.0), (20.0, 4.0), true);
        }
        true
    }
}
/// Layout submenu diagrams share the same vector set as the ribbon.
pub(crate) mod layout_menu {
    use egui::{Color32, Painter, Rect};
    pub fn paint(p: &Painter, r: Rect, name: &str, ink: Color32) -> bool {
        if name.starts_with("mg_")
            || name.starts_with("cols_")
            || name.starts_with("brk_")
            || name.starts_with("ln_")
            || matches!(name, "colsMore" | "lnOptions" | "dirHorizontal" | "dirDown" | "dirUp" | "portrait" | "landscape")
        {
            return super::clipboard_pilot::paint(p, r, name, ink);
        }
        false
    }
}
/// Table Layout selection, deletion and autofit menu diagrams.
pub(crate) mod table_menu {
    use egui::{Color32, Painter, Rect};
    pub fn paint(p: &Painter, r: Rect, name: &str, ink: Color32) -> bool {
        if name.starts_with("sel_") || name.starts_with("del_") || name.starts_with("fit_") {
            return super::clipboard_pilot::paint(p, r, name, ink);
        }
        false
    }
}
/// Draw an Insert › Icons library icon into `r` in `c`, from the same path data the inserted
/// picture is rendered from (`wordcraft_render::icon_lib`).
pub fn library(p: &Painter, r: Rect, icon: &wordcraft_render::icon_lib::Icon, c: Color32) {
    use wordcraft_render::icon_lib::{GRID, STROKE};
    let s = r.width().min(r.height()) / GRID;
    let o = r.center() - vec2(GRID / 2.0 * s, GRID / 2.0 * s);
    let stroke = Stroke::new(STROKE * s, c);
    for piece in icon.pieces() {
        let pts: Vec<Pos2> = piece.points.iter().map(|q| pos2(o.x + q[0] * s, o.y + q[1] * s)).collect();
        if piece.fill {
            p.add(Shape::convex_polygon(pts, c, Stroke::NONE));
        } else if piece.closed {
            p.add(Shape::closed_line(pts, stroke));
        } else {
            // Round caps, as on the inserted picture.
            for end in [pts.first(), pts.last()].into_iter().flatten() {
                p.circle_filled(*end, stroke.width / 2.0, c);
            }
            p.add(Shape::line(pts, stroke));
        }
    }
}

/// The Header & Footer tab's own symbols, drawn here in code (original artwork, in the shared
/// Quiet Glass style: ink strokes on a 20-unit grid with gold surfaces; flat on the band).
pub mod hf {
    use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};

    /// Every name this module draws.
    pub const ALL: [&str; 10] =
        ["hfGoHeader", "hfGoFooter", "hfPrevious", "hfNext", "hfLink", "hfFromTop", "hfFromBottom", "hfClose", "hfField", "hfPictureFile"];

    struct Pen<'a> {
        p: &'a Painter,
        r: Rect,
        c: Color32,
        flat: bool,
    }

    impl Pen<'_> {
        fn s(&self) -> f32 {
            self.r.width().min(self.r.height()) / 24.0
        }
        fn pt(&self, x: f32, y: f32) -> Pos2 {
            let s = self.s();
            let o = self.r.center() - vec2(12.0 * s, 12.0 * s);
            pos2(o.x + x * 1.2 * s, o.y + y * 1.2 * s)
        }
        fn stroke(&self) -> Stroke {
            Stroke::new((1.35 * self.s()).max(0.85), self.c)
        }
        fn gold(&self) -> Color32 {
            if self.flat { self.c.gamma_multiply(0.16) } else { krub_icons::gold(self.c, krub_icons::GoldRole::Primary, 1.0) }
        }
        fn line(&self, pts: &[(f32, f32)]) {
            let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
            let st = self.stroke();
            self.p.add(Shape::line(v, st));
            for &(x, y) in pts.first().into_iter().chain(pts.last()) {
                self.p.circle_filled(self.pt(x, y), st.width * 0.5, self.c);
            }
        }
        fn dashed(&self, x0: f32, x1: f32, y: f32) {
            let mut x = x0;
            while x < x1 {
                self.line(&[(x, y), ((x + 1.6).min(x1), y)]);
                x += 3.4;
            }
        }
        fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
            self.p.rect_stroke(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), 1.0, self.stroke(), StrokeKind::Middle);
        }
        fn frect(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
            self.p.rect_filled(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), 1.0, self.gold());
        }
        fn fill(&self, pts: &[(f32, f32)], c: Color32) {
            let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
            self.p.add(Shape::convex_polygon(v, c, Stroke::NONE));
        }
        /// A page with a folded corner.
        fn page(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
            let v: Vec<Pos2> = [(x0, y0), (x1 - 4.0, y0), (x1, y0 + 4.0), (x1, y1), (x0, y1)].iter().map(|&(x, y)| self.pt(x, y)).collect();
            self.p.add(Shape::closed_line(v, self.stroke()));
            self.line(&[(x1 - 4.0, y0), (x1 - 4.0, y0 + 4.0), (x1, y0 + 4.0)]);
        }
        /// An arrow head pointing `dir` (unit vector) with its tip at (x, y).
        fn head(&self, x: f32, y: f32, dx: f32, dy: f32) {
            let (px, py) = (-dy, dx);
            self.fill(&[(x, y), (x - dx * 3.0 + px * 2.4, y - dy * 3.0 + py * 2.4), (x - dx * 3.0 - px * 2.4, y - dy * 3.0 - py * 2.4)], self.c);
        }
    }

    /// Draw `name` if it is one of this tab's symbols.
    pub fn paint(p: &Painter, r: Rect, name: &str, c: Color32, flat: bool) -> bool {
        let size = r.width().min(r.height());
        if !size.is_finite() || size <= 0.5 || !r.center().is_finite() || !ALL.contains(&name) {
            return false;
        }
        let pen = Pen { p, r, c, flat };
        match name {
            // A page with its header band and an arrow into it.
            "hfGoHeader" => {
                pen.page(5.0, 2.0, 17.0, 18.0);
                pen.frect(6.5, 4.5, 13.0, 7.5);
                pen.dashed(6.5, 15.5, 9.5);
                pen.line(&[(11.0, 16.0), (11.0, 12.0)]);
                pen.head(11.0, 10.8, 0.0, -1.0);
            }
            "hfGoFooter" => {
                pen.page(5.0, 2.0, 17.0, 18.0);
                pen.frect(6.5, 13.0, 15.5, 16.0);
                pen.dashed(6.5, 15.5, 10.5);
                pen.line(&[(11.0, 4.5), (11.0, 8.0)]);
                pen.head(11.0, 9.3, 0.0, 1.0);
            }
            // A header band with an arrow up (previous section) or down (next).
            "hfPrevious" | "hfNext" => {
                pen.rect(3.0, 5.0, 12.0, 15.0);
                pen.frect(4.5, 6.5, 10.5, 9.0);
                let (y0, y1, dy) = if name == "hfPrevious" { (15.0, 5.5, -1.0) } else { (5.0, 14.5, 1.0) };
                pen.line(&[(16.0, y0), (16.0, y1 - dy * 1.2)]);
                pen.head(16.0, y1, 0.0, dy);
            }
            // Two section pages joined by a chain link.
            "hfLink" => {
                pen.rect(2.5, 3.0, 8.5, 11.0);
                pen.frect(3.8, 4.3, 7.2, 6.2);
                pen.rect(11.5, 9.0, 17.5, 17.0);
                pen.frect(12.8, 10.3, 16.2, 12.2);
                let link = |cx: f32, cy: f32| {
                    p.circle_stroke(pen.pt(cx, cy), 1.9 * 1.2 * pen.s(), pen.stroke());
                };
                link(8.6, 13.4);
                link(11.4, 6.6);
                pen.line(&[(9.2, 11.6), (10.8, 8.4)]);
            }
            // The page's top edge, the header and the distance between them.
            "hfFromTop" => {
                pen.line(&[(3.0, 3.0), (17.0, 3.0)]);
                pen.frect(8.0, 10.0, 17.0, 13.0);
                pen.line(&[(5.0, 5.5), (5.0, 12.0)]);
                pen.head(5.0, 4.6, 0.0, -1.0);
                pen.head(5.0, 13.4, 0.0, 1.0);
                pen.dashed(8.0, 17.0, 16.0);
            }
            "hfFromBottom" => {
                pen.line(&[(3.0, 17.0), (17.0, 17.0)]);
                pen.frect(8.0, 7.0, 17.0, 10.0);
                pen.line(&[(5.0, 8.0), (5.0, 14.5)]);
                pen.head(5.0, 6.6, 0.0, -1.0);
                pen.head(5.0, 15.4, 0.0, 1.0);
                pen.dashed(8.0, 17.0, 4.0);
            }
            // A header band crossed out.
            "hfClose" => {
                pen.rect(3.0, 3.0, 17.0, 17.0);
                pen.frect(4.5, 4.5, 15.5, 7.0);
                pen.line(&[(7.0, 10.0), (13.0, 16.0)]);
                pen.line(&[(13.0, 10.0), (7.0, 16.0)]);
            }
            // Field braces around a grey result.
            "hfField" => {
                pen.frect(6.5, 7.5, 13.5, 12.5);
                pen.line(&[(5.0, 4.0), (3.5, 5.0), (3.5, 9.0), (2.0, 10.0), (3.5, 11.0), (3.5, 15.0), (5.0, 16.0)]);
                pen.line(&[(15.0, 4.0), (16.5, 5.0), (16.5, 9.0), (18.0, 10.0), (16.5, 11.0), (16.5, 15.0), (15.0, 16.0)]);
            }
            // A picture (frame, sun, hill) on a page.
            "hfPictureFile" => {
                pen.page(3.0, 2.0, 15.0, 18.0);
                pen.rect(8.0, 9.0, 18.0, 17.0);
                pen.fill(&[(9.0, 16.0), (12.5, 12.0), (15.0, 14.5), (17.0, 13.0), (17.0, 16.0)], pen.gold());
                p.circle_filled(pen.pt(15.0, 11.4), 1.3 * 1.2 * pen.s(), pen.gold());
            }
            _ => return false,
        }
        true
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn every_symbol_draws_and_tiny_or_broken_rects_are_skipped() {
            let ctx = egui::Context::default();
            let p = Painter::new(ctx, egui::LayerId::background(), Rect::EVERYTHING);
            let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(20.0, 20.0));
            for name in ALL {
                assert!(paint(&p, r, name, Color32::BLACK, false), "{name}");
                assert!(paint(&p, r, name, Color32::WHITE, true), "{name}");
                assert!(!paint(&p, Rect::from_min_size(pos2(0.0, 0.0), vec2(0.1, 0.1)), name, Color32::BLACK, false));
                assert!(!paint(&p, Rect::from_min_size(pos2(f32::NAN, 0.0), vec2(20.0, 20.0)), name, Color32::BLACK, false));
            }
            assert!(!paint(&p, r, "bold", Color32::BLACK, false));
        }
    }
}
