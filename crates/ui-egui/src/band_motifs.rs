//! The Thai motifs drawn faintly on the band at the top (File › Options › General › Background):
//! ประจำยาม, กระจัง, ก้านขด, ดอกพิกุล and ผ้าขาวม้า. Simple original renderings of these traditional
//! patterns, drawn in code in white over the band's colour. They start a little over a third of the
//! way across and fade in, so the tabs and Quick Access buttons on the left stay on plain colour.

use egui::epaint::{CubicBezierShape, PathShape, PathStroke};
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

use crate::theme::BandMotif;

/// Where the band (title bar and tab strip together) is on screen, kept by the title bar for the
/// tab strip, so a motif runs on across both.
fn band_rect_id() -> egui::Id {
    egui::Id::new("khiankrub_band_rect")
}

/// Remember the whole band's rectangle (the title bar's, down to the tab strip's bottom).
pub fn set_band_rect(ctx: &egui::Context, band: Rect) {
    ctx.data_mut(|d| d.insert_temp(band_rect_id(), band));
}

/// The whole band's rectangle, if the title bar has drawn this frame or before.
pub fn band_rect(ctx: &egui::Context) -> Option<Rect> {
    ctx.data(|d| d.get_temp(band_rect_id()))
}

/// How far across the band the motif starts, and over how many points it fades in.
const START: f32 = 0.36;
const FADE: f32 = 120.0;

/// Paint `motif` over `band` (clipped to the painter's clip rect). `dark`: the dark theme, where
/// the motif is a little fainter.
pub fn paint(painter: &Painter, band: Rect, motif: BandMotif, dark: bool) {
    draw(painter, band, motif, band.min.x + band.width() * START, if dark { 0.15 } else { 0.2 });
}

/// A sample of `motif` filling `rect` (the Options picker): no fade, a little stronger.
pub fn paint_sample(painter: &Painter, rect: Rect, motif: BandMotif) {
    draw(painter, rect, motif, rect.min.x - FADE, 0.4);
}

fn draw(painter: &Painter, band: Rect, motif: BandMotif, start: f32, opacity: f32) {
    if motif == BandMotif::None || band.width() < 1.0 || band.height() < 1.0 {
        return;
    }
    let pen = Pen { painter, start, opacity };
    match motif {
        BandMotif::None => {}
        BandMotif::PrajamYam => prajam_yam(&pen, band),
        BandMotif::Krajang => krajang(&pen, band),
        BandMotif::KanKhod => kan_khod(&pen, band),
        BandMotif::Pikul => pikul(&pen, band),
        BandMotif::PhaKhaoMa => pha_khao_ma(&pen, band),
    }
}

struct Pen<'a> {
    painter: &'a Painter,
    start: f32,
    opacity: f32,
}

impl Pen<'_> {
    /// White at the motif's opacity times `k`, faded in from `start`.
    fn white(&self, x: f32, k: f32) -> Color32 {
        let fade = ((x - self.start) / FADE).clamp(0.0, 1.0);
        Color32::from_white_alpha((255.0 * self.opacity * k * fade).round() as u8)
    }

    /// Whether something centred at `x`, `reach` wide either side, shows at all.
    fn shows(&self, x: f32, reach: f32) -> bool {
        x + reach > self.start
    }

    fn stroke(&self, x: f32, k: f32) -> Stroke {
        Stroke::new(1.2, self.white(x, k))
    }

    fn curve(&self, pts: [Pos2; 4], closed: bool, x: f32) {
        self.painter.add(CubicBezierShape::from_points_stroke(pts, closed, Color32::TRANSPARENT, self.stroke(x, 1.0)));
    }

    fn path(&self, pts: Vec<Pos2>, closed: bool, x: f32) {
        let stroke = PathStroke::new(1.2, self.white(x, 1.0));
        self.painter.add(if closed { PathShape::closed_line(pts, stroke) } else { PathShape::line(pts, stroke) });
    }
}

/// `p` turned by `deg` degrees about `c`.
fn turn(c: Pos2, p: Pos2, deg: f32) -> Pos2 {
    let (s, co) = deg.to_radians().sin_cos();
    let d = p - c;
    c + vec2(d.x * co - d.y * s, d.x * s + d.y * co)
}

/// The x positions of a lattice of `pitch` anchored at the band's left edge (shifted by `offset`),
/// from just before the motif starts to past the band's right edge.
fn columns(pen: &Pen, band: Rect, pitch: f32, offset: f32) -> Vec<f32> {
    let mut xs = Vec::new();
    let mut x = band.min.x + offset;
    while x < band.max.x + pitch {
        if pen.shows(x, pitch) {
            xs.push(x);
        }
        x += pitch;
        if xs.len() > 4000 {
            break;
        }
    }
    xs
}

/// ประจำยาม: a diamond with four petals meeting at a centre dot, in a staggered grid.
fn prajam_yam(pen: &Pen, band: Rect) {
    const P: f32 = 34.0;
    let r = P * 0.42;
    let mut y = band.min.y - P / 2.0;
    let mut row = 0;
    while y < band.max.y + P {
        let offset = if row % 2 == 1 { P / 2.0 } else { 0.0 };
        for x in columns(pen, band, P, offset) {
            let c = pos2(x, y);
            pen.path(vec![pos2(x, y - r), pos2(x + r, y), pos2(x, y + r), pos2(x - r, y)], true, x);
            for k in 0..4 {
                let a = k as f32 * 90.0;
                let t = |dx: f32, dy: f32| turn(c, c + vec2(dx * r, dy * r), a);
                pen.curve([c, t(-0.45, -0.35), t(-0.25, -0.8), t(0.0, -0.82)], false, x);
                pen.curve([t(0.0, -0.82), t(0.25, -0.8), t(0.45, -0.35), c], false, x);
            }
            pen.painter.circle_filled(c, r * 0.16, pen.white(x, 1.0));
        }
        y += P / 2.0;
        row += 1;
    }
}

/// กระจัง: a row of leaf-and-flame points rising from the band's bottom edge, each with an inner
/// drop, and a row of small dots along the top.
fn krajang(pen: &Pen, band: Rect) {
    const W: f32 = 22.0;
    let b = band.max.y;
    let hh = band.height() * 0.55;
    for x in columns(pen, band, W, 0.0) {
        let mid = x + W / 2.0;
        let p = |fx: f32, fy: f32| pos2(x + W * fx, b - hh * fy);
        pen.curve([p(0.0, 0.0), p(0.05, 0.5), p(0.35, 0.55), p(0.5, 1.0)], false, mid);
        pen.curve([p(0.5, 1.0), p(0.65, 0.55), p(0.95, 0.5), p(1.0, 0.0)], false, mid);
        pen.curve([p(0.5, 0.72), p(0.38, 0.42), p(0.3, 0.2), p(0.5, 0.08)], false, mid);
        pen.curve([p(0.5, 0.08), p(0.7, 0.2), p(0.62, 0.42), p(0.5, 0.72)], false, mid);
        pen.painter.circle_filled(pos2(mid, band.min.y + band.height() * 0.18), 1.4, pen.white(mid, 1.0));
    }
}

/// ก้านขด: a wavy stem across the band with a tendril curling from each crest and trough.
fn kan_khod(pen: &Pen, band: Rect) {
    const P: f32 = 40.0;
    let (y0, h) = (band.min.y, band.height());
    for x in columns(pen, band, P, 0.0) {
        let p = |fx: f32, fy: f32| pos2(x + P * fx, y0 + h * fy);
        pen.curve([p(0.0, 0.5), p(0.25, 0.15), p(0.5, 0.15), p(0.5, 0.5)], false, x + P * 0.25);
        pen.curve([p(0.5, 0.5), p(0.5, 0.85), p(0.75, 0.85), p(1.0, 0.5)], false, x + P * 0.75);
        spiral(pen, p(0.32, 0.3), h * 0.2, 1.0);
        spiral(pen, p(0.82, 0.7), h * 0.2, -1.0);
    }
}

/// A curl winding in from radius `r` about `c`; `dir` -1 mirrors it.
fn spiral(pen: &Pen, c: Pos2, r: f32, dir: f32) {
    let turns = std::f32::consts::PI * 3.2;
    let pts: Vec<Pos2> = (0..=13)
        .map(|i| {
            let t = turns * i as f32 / 13.0;
            let rr = r * (1.0 - t / (std::f32::consts::PI * 3.6));
            pos2(c.x + dir * rr * t.cos(), c.y + rr * t.sin())
        })
        .collect();
    pen.path(pts, false, c.x);
}

/// ดอกพิกุล: small eight-petal flowers, each petal a slim point, around a ring.
fn pikul(pen: &Pen, band: Rect) {
    const P: f32 = 24.0;
    let r = P * 0.32;
    let mut y = band.min.y + P / 2.0;
    let mut row = 0;
    while y < band.max.y + P {
        let offset = if row % 2 == 1 { P / 2.0 } else { 0.0 };
        for x in columns(pen, band, P, offset) {
            let c = pos2(x, y);
            let fill = pen.white(x, 0.8);
            for k in 0..8 {
                let a = k as f32 * 45.0;
                let t = |dx: f32, dy: f32| turn(c, c + vec2(dx * r, dy * r), a);
                pen.painter.add(Shape::convex_polygon(vec![c, t(-0.22, -0.75), t(0.0, -1.0), t(0.22, -0.75)], fill, Stroke::NONE));
            }
            pen.painter.circle_stroke(c, r * 0.22, Stroke::new(1.0, pen.white(x, 1.0)));
        }
        y += P * 0.8;
        row += 1;
    }
}

/// ผ้าขาวม้า: light bands across and down the band, overlapping like a woven check, with fine
/// threads between.
fn pha_khao_ma(pen: &Pen, band: Rect) {
    const P: f32 = 16.0;
    for x in columns(pen, band, P * 2.0, 0.0) {
        pen.painter.rect_filled(Rect::from_min_size(pos2(x, band.min.y), vec2(P, band.height())), 0.0, pen.white(x + P / 2.0, 0.35));
    }
    let mut y = band.min.y;
    while y < band.max.y {
        for x in columns(pen, band, P, 0.0) {
            pen.painter.rect_filled(Rect::from_min_size(pos2(x, y), vec2(P, P)), 0.0, pen.white(x + P / 2.0, 0.35));
        }
        y += P * 2.0;
    }
    for x in columns(pen, band, P, P / 2.0) {
        pen.painter.line_segment([pos2(x, band.min.y), pos2(x, band.max.y)], Stroke::new(0.6, pen.white(x, 0.5)));
    }
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2};

    use crate::theme::BandMotif;

    /// Every motif draws something on the band's right part and nothing on its left third, where
    /// the tabs are; None draws nothing.
    #[test]
    fn motifs_draw_on_the_right_and_leave_the_left_plain() {
        let ctx = egui::Context::default();
        let band = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 68.0));
        for motif in BandMotif::ALL {
            let out = ctx.run_ui(egui::RawInput::default(), |ui| super::paint(ui.painter(), band, motif, false));
            let shapes: Vec<_> = out.shapes.iter().map(|c| c.shape.clone()).collect();
            out.drop_without_applying_deltas();
            let visible: Vec<_> = shapes.iter().filter(|s| s.visual_bounding_rect().is_positive()).collect();
            if motif == BandMotif::None {
                assert!(visible.is_empty(), "None draws nothing");
                continue;
            }
            assert!(visible.len() > 10, "{motif:?} draws a pattern");
            let left_edge = visible.iter().map(|s| s.visual_bounding_rect().min.x).fold(f32::INFINITY, f32::min);
            assert!(left_edge > 1200.0 * 0.3, "{motif:?} keeps the tabs' side plain (starts at {left_edge})");
        }
    }
}
