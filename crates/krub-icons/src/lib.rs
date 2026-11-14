//! Original Quiet Glass artwork, MIT OR Apache-2.0. See docs/icon-design.md.
//! Shared source: keep this crate identical in all three KrubOffice repositories.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, vec2};

pub mod deck;
pub mod line;
pub mod menu;
pub mod sheet;
mod thai;
mod word;

/// Balanced gold distinguishes prominent details from softer background surfaces.
#[derive(Clone, Copy, Debug)]
pub enum GoldRole {
    Primary,
    Surface,
}

/// Shared toolbar gold; caller alpha and drawing opacity remain multiplicative.
pub fn gold(ink: Color32, role: GoldRole, opacity: f32) -> Color32 {
    let strength = match role {
        GoldRole::Primary => 230.0 / 255.0,
        GoldRole::Surface => 107.0 / 255.0,
    };
    Color32::from_rgba_unmultiplied(242, 183, 5, (f32::from(ink.a()) * strength * opacity.clamp(0.0, 1.0)).round() as u8)
}

/// Resolve existing application spellings to a single semantic catalog.
pub fn canonical_name(name: &str) -> String {
    let alias = match name {
        "file" | "blankPage" => "new-file",
        "open" | "folder" => "open-folder",
        "painter" | "Brush" => "format-painter",
        "strike" => "strikethrough",
        "grow" => "font-grow",
        "shrink" => "font-shrink",
        "case" => "change-case",
        "clear" => "clear-formatting",
        "fontcolor" => "font-color",
        "effects" => "text-effects",
        "indent" => "increase-indent",
        "outdent" => "decrease-indent",
        "thaiDistribute" => "thai-distribution",
        "thaiDigits" => "thai-digits",
        "find" | "search" => "search",
        "shading" | "fill" => "shape-fill",
        "borderPainter" => "format-painter",
        "spelling" | "Spell" => "spelling",
        "styles" => "quick-styles",
        "smartArt" => "smart-art",
        "smartArtColors" => "colors",
        "resetGraphic" => "reset-slide",
        "lasso" => "lasso-select",
        "replayPlay" | "mediaPlay" => "play",
        "replayPause" | "mediaPause" => "pause",
        "mediaStop" => "stop",
        "zoom" => "zoom-glass",
        "zoomOut" => "zoom-out",
        "theme" => "themes",
        "darkMode" | "interfaceTheme" | "appearance" => "palette",
        "options" | "settings" => "settings",
        "navPane" => "selection-pane",
        "sorter" => "view-sorter-small",
        "readMode" => "view-reading-small",
        "dropdown" | "chevron" => "chevron-down",
        "chevronLeft" => "chevron-left",
        "chevronRight" => "chevron-right",
        "chevronUp" => "chevron-up",
        "chevronDown" => "chevron-down",
        "newWindow" => "new-window",
        "zoom100" => "zoom-in",
        "closeFile" => "close",
        "gridlines" => "gridlines",
        "language" => "language",
        "textBox" => "text-box",
        "symbol" => "symbol",
        other => other,
    };
    let mut normalized = String::with_capacity(alias.len().min(128));
    for (index, ch) in alias.chars().take(128).enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            normalized.push('-');
        }
        normalized.push(ch.to_ascii_lowercase());
    }
    normalized
}

/// Draw every catalog entry through shared geometry and style. Returns false for unknown names.
pub fn paint_named(p: &Painter, r: Rect, name: &str, ink: Color32, flat: bool, disabled: bool) -> bool {
    let size = r.width().min(r.height());
    if !size.is_finite() || size <= 0.5 || !r.center().is_finite() {
        return false;
    }
    if let Some(alias) = sheet::Icon::ALL.iter().copied().find(|icon| icon.name() == name).and_then(sheet::canonical)
        && alias != name
    {
        return paint_named(p, r, alias, ink, flat, disabled);
    }
    let canonical = canonical_name(name);
    if let Some(icon) = deck::Icon::from_name(&canonical) {
        deck::paint_ex(p, r, icon, ink, disabled, flat);
        return true;
    }
    // These actions are also available in the spreadsheet-specific catalog.
    if let Some(icon) = sheet::Icon::ALL.iter().copied().find(|icon| icon.name() == canonical && sheet::canonical(*icon).is_none()) {
        let ink = if disabled { ink.gamma_multiply(0.35) } else { ink };
        if flat || disabled {
            sheet::paint_flat(p, r, icon, ink, Color32::TRANSPARENT);
        } else {
            sheet::paint(p, r, icon, ink);
        }
        return true;
    }
    if word::ALL.contains(&name) {
        let ink = if disabled { ink.gamma_multiply(0.35) } else { ink };
        word::paint(p, r, name, ink, flat || disabled);
        return true;
    }
    false
}

/// Complete inventory, used by the review gallery and coverage checks.
pub fn catalog() -> Vec<String> {
    let mut names: Vec<String> = word::ALL
        .iter()
        .map(|name| (*name).to_owned())
        .chain(deck::Icon::ALL.iter().map(|icon| icon.name().to_owned()))
        .chain(sheet::Icon::ALL.iter().map(|icon| icon.name().to_owned()))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Glass treatment for the existing openly licensed Thai letter in start-screen marks.
/// Reuses the caller's font and geometry, including dimming for unavailable apps.
pub fn glass_letter(painter: &Painter, centre: Pos2, letter: &str, font: egui::FontId, color: Color32) {
    let galley = painter.layout_no_wrap(letter.to_owned(), font, Color32::WHITE);
    let origin = centre - galley.size() * 0.5;
    let bounds = Rect::from_min_size(origin, galley.size());
    let stops = [(255, 243, 186), (255, 209, 61), (237, 170, 8), (255, 230, 138), (232, 163, 8), (255, 220, 82)];
    for band in 0..24 {
        let t = band as f32 / 23.0 * 5.0;
        let index = (t.floor() as usize).min(4);
        let fraction = t - index as f32;
        if let (Some(&(ar, ag, ab)), Some(&(br, bg, bb))) = (stops.get(index), stops.get(index + 1)) {
            let lerp = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * fraction).round() as u8;
            let tint = Color32::from_rgba_unmultiplied(lerp(ar, br), lerp(ag, bg), lerp(ab, bb), color.a());
            let top = bounds.top() + bounds.height() * band as f32 / 24.0;
            let clip = Rect::from_min_max(
                egui::pos2(bounds.left() - 1.0, top),
                egui::pos2(bounds.right() + 1.0, bounds.top() + bounds.height() * (band + 1) as f32 / 24.0),
            );
            painter.with_clip_rect(painter.clip_rect().intersect(clip)).galley_with_override_text_color(origin, galley.clone(), tint);
        }
    }
}

/// Common actions, independent of each application's command IDs.
#[derive(Clone, Copy, Debug)]
pub enum Icon {
    Document,
    Open,
    Save,
    Undo,
    Table,
    Chart,
    Present,
    Comment,
}

struct Pen<'a> {
    painter: &'a Painter,
    origin: Pos2,
    scale: f32,
    ink: Color32,
    tint: Color32,
    width: f32,
}

impl Pen<'_> {
    fn point(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.scale
    }

    fn line(&self, points: &[(f32, f32)]) {
        self.painter.add(Shape::line(points.iter().map(|&(x, y)| self.point(x, y)).collect(), Stroke::new(self.width, self.ink)));
        // Round caps retain clarity at small sizes; egui joins are tessellated.
        for &(x, y) in points.first().into_iter().chain(points.last()) {
            self.painter.circle_filled(self.point(x, y), self.width * 0.5, self.ink);
        }
    }

    fn polygon(&self, points: &[(f32, f32)]) {
        self.painter.add(Shape::convex_polygon(
            points.iter().map(|&(x, y)| self.point(x, y)).collect(),
            self.tint,
            Stroke::new(self.width, self.ink),
        ));
    }

    fn rect(&self, x: f32, y: f32, width: f32, height: f32, filled: bool) {
        let r = Rect::from_min_size(self.point(x, y), vec2(width, height) * self.scale);
        self.painter.rect(
            r,
            (1.5 * self.scale).round() as u8,
            if filled { self.tint } else { Color32::TRANSPARENT },
            Stroke::new(self.width, self.ink),
            StrokeKind::Middle,
        );
    }
}

/// Draw on a 24-unit master grid without changing the caller's layout or hit targets.
/// `flat` suppresses gold for coloured bands; disabled icons preserve the caller's ink alpha.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, ink: Color32, flat: bool, disabled: bool) {
    let size = rect.width().min(rect.height());
    if !size.is_finite() || size <= 0.5 || !rect.center().is_finite() {
        return;
    }
    let ink = if disabled { ink.gamma_multiply(0.35) } else { ink };
    let tint = if flat || disabled { ink.gamma_multiply(0.12) } else { gold(ink, GoldRole::Primary, 1.0) };
    let p = Pen { painter, origin: rect.center() - vec2(size, size) * 0.5, scale: size / 24.0, ink, tint, width: (size * 1.35 / 24.0).max(0.85) };
    match icon {
        Icon::Document => {
            p.polygon(&[(6.0, 3.5), (14.0, 3.5), (18.0, 7.5), (18.0, 20.5), (6.0, 20.5)]);
            p.line(&[(14.0, 3.5), (14.0, 7.5), (18.0, 7.5)]);
            p.line(&[(9.0, 12.0), (15.0, 12.0)]);
            if size > 16.0 {
                p.line(&[(9.0, 16.0), (13.0, 16.0)]);
            }
        }
        Icon::Open => {
            p.line(&[(3.5, 18.0), (3.5, 5.5), (9.5, 5.5), (11.5, 8.5), (20.5, 8.5)]);
            p.polygon(&[(3.5, 19.5), (6.0, 11.5), (21.0, 11.5), (18.0, 19.5)]);
        }
        Icon::Save => {
            p.line(&[(12.0, 3.0), (12.0, 14.0)]);
            p.line(&[(8.0, 10.0), (12.0, 14.0), (16.0, 10.0)]);
            p.line(&[(4.5, 14.0), (4.5, 20.0), (19.5, 20.0), (19.5, 14.0)]);
        }
        Icon::Undo => {
            p.line(&[(8.0, 5.0), (4.0, 9.0), (8.0, 13.0)]);
            p.line(&[(4.0, 9.0), (13.0, 9.0), (16.0, 9.8), (18.0, 12.0), (19.0, 15.0), (18.0, 18.0), (16.0, 20.0), (13.0, 21.0)]);
        }
        Icon::Table => {
            p.rect(3.0, 4.0, 18.0, 5.0, true);
            p.rect(3.0, 4.0, 18.0, 16.0, false);
            p.line(&[(3.0, 14.5), (21.0, 14.5)]);
            p.line(&[(9.0, 9.0), (9.0, 20.0)]);
            if size > 16.0 {
                p.line(&[(15.0, 9.0), (15.0, 20.0)]);
            }
        }
        Icon::Chart => {
            p.line(&[(3.0, 3.0), (3.0, 21.0), (21.0, 21.0)]);
            p.rect(7.0, 12.0, 3.0, 6.0, true);
            p.rect(13.0, 5.0, 3.0, 13.0, true);
            if size > 16.0 {
                p.line(&[(19.0, 10.0), (21.0, 6.0)]);
            }
        }
        Icon::Present => {
            p.rect(3.0, 4.0, 18.0, 13.0, false);
            p.polygon(&[(10.0, 7.0), (15.0, 10.5), (10.0, 14.0)]);
            p.line(&[(12.0, 17.0), (12.0, 21.0)]);
            p.line(&[(8.0, 21.0), (16.0, 21.0)]);
        }
        Icon::Comment => {
            // Body and tail are separate convex primitives (no concave tessellation).
            p.rect(3.0, 4.0, 18.0, 13.0, true);
            p.line(&[(5.0, 17.0), (5.0, 21.0), (11.0, 17.0)]);
            p.line(&[(7.0, 9.0), (17.0, 9.0)]);
            if size > 16.0 {
                p.line(&[(7.0, 12.0), (13.0, 12.0)]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sizes_and_states_tessellate_to_finite_vertices() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        {
            let painter = ctx.layer_painter(egui::LayerId::background());
            for size in [16.0, 20.0, 24.0, 32.0] {
                for icon in [Icon::Document, Icon::Open, Icon::Save, Icon::Undo, Icon::Table, Icon::Chart, Icon::Present, Icon::Comment] {
                    for (flat, disabled) in [(false, false), (true, false), (false, true)] {
                        paint(&painter, Rect::from_min_size(Pos2::ZERO, vec2(size, size)), icon, Color32::WHITE, flat, disabled);
                    }
                }
            }
        }
        let mut output = ctx.end_pass();
        output.textures_delta.clear();
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        assert!(!primitives.is_empty());
        for primitive in primitives {
            if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
            }
        }
    }

    #[test]
    fn invalid_bounds_are_ignored() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        {
            let painter = ctx.layer_painter(egui::LayerId::background());
            for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
                paint(&painter, Rect::from_min_size(Pos2::ZERO, vec2(size, size)), Icon::Open, Color32::WHITE, false, false);
            }
        }
        let mut output = ctx.end_pass();
        output.textures_delta.clear();
        assert!(output.shapes.is_empty());
    }
}

#[cfg(test)]
mod catalog_tests;
