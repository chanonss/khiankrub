use egui::{Align2, Rect, Stroke, vec2};
pub fn paint(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, accent: egui::Color32) {
    if !(draw_0(p, r, name, ink, accent) || draw_1(p, r, name, ink, accent) || draw_2(p, r, name, ink, accent) || draw_3(p, r, name, ink, accent)) {
        crate::paint_named(p, r, name, ink, false, false);
    }
}
fn draw_0(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, _accent: egui::Color32) -> bool {
    let accent = ink;
    let s = (r.width() / 24.0).min(r.height() / 24.0);
    let o = r.center() - vec2(12.0 * s, 12.0 * s);
    let pt = |x: f32, y: f32| egui::pos2(o.x + x * s, o.y + (y + 2.0) * s);
    let line = |a: (f32, f32), b: (f32, f32), c: egui::Color32| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new((1.75 * s).max(1.0), c));
        for point in [a, b] {
            p.circle_filled(pt(point.0, point.1), (1.75 * s).max(1.0) * 0.5, c);
        }
    };
    let page = |x0: f32, y0: f32, x1: f32, y1: f32| {
        p.rect_stroke(Rect::from_min_max(pt(x0, y0), pt(x1, y1)), 1.0, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
    };
    // Text lines across `x0..x1`, starting at `y`, every `gap`.
    let rows = |x0: f32, x1: f32, y: f32, n: usize, gap: f32, c: egui::Color32| {
        for i in 0..n {
            line((x0, y + i as f32 * gap), (x1, y + i as f32 * gap), c);
        }
    };
    let head = |tip: (f32, f32), dx: f32, dy: f32| {
        let t = pt(tip.0, tip.1);
        let back = vec2(-dx, -dy) * 3.0 * s;
        let side = vec2(-dy, dx) * 2.2 * s;
        p.add(egui::Shape::convex_polygon(vec![t, t + back + side, t + back - side], accent, Stroke::NONE));
    };
    let dashed = |y: f32, x0: f32, x1: f32, double: bool| {
        for dy in if double { vec![-0.8, 0.8] } else { vec![0.0] } {
            p.extend(egui::Shape::dashed_line(&[pt(x0, y + dy), pt(x1, y + dy)], Stroke::new(1.0, accent), 2.0 * s, 1.5 * s));
        }
    };
    match name {
        "dirHorizontal" => {
            page(4.0, 1.0, 20.0, 19.0);
            rows(6.5, 17.5, 5.0, 3, 3.0, ink);
            line((7.0, 15.5), (16.0, 15.5), accent);
            head((18.0, 15.5), 1.0, 0.0);
        }
        "dirDown" | "dirUp" => {
            page(4.0, 1.0, 20.0, 19.0);
            for x in [8.0, 11.0, 14.0] {
                line((x, 4.0), (x, 16.0), ink);
            }
            if name == "dirDown" {
                line((17.0, 4.0), (17.0, 14.0), accent);
                head((17.0, 16.5), 0.0, 1.0);
            } else {
                line((17.0, 16.0), (17.0, 6.0), accent);
                head((17.0, 3.5), 0.0, -1.0);
            }
        }
        // Orientation: a tall or a wide page.
        "portrait" => {
            page(6.0, 0.5, 18.0, 19.5);
            rows(8.0, 16.0, 4.0, 4, 3.0, ink);
            line((8.0, 1.0), (16.0, 1.0), accent);
        }
        "landscape" => {
            page(1.5, 4.0, 22.5, 16.0);
            rows(4.0, 20.0, 7.0, 3, 3.0, ink);
            line((2.0, 4.5), (22.0, 4.5), accent);
        }
        // Columns: the page's text in one, two or three columns, or a narrow one on a side.
        _ if name.starts_with("cols") => {
            page(3.0, 1.0, 21.0, 19.0);
            let spans: &[(f32, f32)] = match name {
                "cols_two" => &[(5.0, 11.0), (13.0, 19.0)],
                "cols_three" => &[(5.0, 8.7), (10.2, 13.8), (15.3, 19.0)],
                "cols_left" => &[(5.0, 8.5), (10.5, 19.0)],
                "cols_right" => &[(5.0, 13.5), (15.5, 19.0)],
                _ => &[(5.0, 19.0)],
            };
            for (i, (x0, x1)) in spans.iter().enumerate() {
                let c = if spans.len() > 1 && i % 2 == 0 { accent } else { ink };
                rows(*x0, *x1, 4.0, 5, 3.0, if name == "colsMore" { ink } else { c });
            }
            if name == "colsMore" {
                for x in [9.0, 12.0, 15.0] {
                    p.circle_filled(pt(x, 17.0), 1.0 * s, accent);
                }
            }
        }
        // Breaks: where the text stops and goes on.
        "brk_page" => {
            page(5.0, 0.5, 19.0, 19.5);
            rows(7.0, 17.0, 3.5, 2, 3.0, ink);
            dashed(10.0, 2.0, 22.0, false);
            rows(7.0, 17.0, 13.5, 2, 3.0, ink);
        }
        _ => return false,
    }
    true
}
fn draw_1(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, _accent: egui::Color32) -> bool {
    let accent = ink;
    let s = (r.width() / 24.0).min(r.height() / 24.0);
    let o = r.center() - vec2(12.0 * s, 12.0 * s);
    let pt = |x: f32, y: f32| egui::pos2(o.x + x * s, o.y + (y + 2.0) * s);
    let line = |a: (f32, f32), b: (f32, f32), c: egui::Color32| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new((1.75 * s).max(1.0), c));
        for point in [a, b] {
            p.circle_filled(pt(point.0, point.1), (1.75 * s).max(1.0) * 0.5, c);
        }
    };
    let page = |x0: f32, y0: f32, x1: f32, y1: f32| {
        p.rect_stroke(Rect::from_min_max(pt(x0, y0), pt(x1, y1)), 1.0, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
    };
    // Text lines across `x0..x1`, starting at `y`, every `gap`.
    let rows = |x0: f32, x1: f32, y: f32, n: usize, gap: f32, c: egui::Color32| {
        for i in 0..n {
            line((x0, y + i as f32 * gap), (x1, y + i as f32 * gap), c);
        }
    };
    let head = |tip: (f32, f32), dx: f32, dy: f32| {
        let t = pt(tip.0, tip.1);
        let back = vec2(-dx, -dy) * 3.0 * s;
        let side = vec2(-dy, dx) * 2.2 * s;
        p.add(egui::Shape::convex_polygon(vec![t, t + back + side, t + back - side], accent, Stroke::NONE));
    };
    let dashed = |y: f32, x0: f32, x1: f32, double: bool| {
        for dy in if double { vec![-0.8, 0.8] } else { vec![0.0] } {
            p.extend(egui::Shape::dashed_line(&[pt(x0, y + dy), pt(x1, y + dy)], Stroke::new(1.0, accent), 2.0 * s, 1.5 * s));
        }
    };
    let digit = |x: f32, y: f32, t: &str, c: egui::Color32| {
        p.text(pt(x, y), Align2::CENTER_CENTER, t, egui::FontId::proportional(6.5 * s), c);
    };
    match name {
        "brk_column" => {
            page(3.0, 1.0, 21.0, 19.0);
            rows(5.0, 11.0, 4.0, 2, 3.0, ink);
            rows(13.0, 19.0, 4.0, 5, 3.0, ink);
            dashed(10.5, 5.0, 11.0, false);
            line((8.0, 13.0), (8.0, 16.5), accent);
            line((8.0, 16.5), (12.0, 16.5), accent);
            line((12.0, 16.5), (12.0, 2.5), accent);
            head((13.5, 2.5), 1.0, 0.0);
        }
        "brk_textWrapping" => {
            rows(3.0, 21.0, 4.0, 1, 0.0, ink);
            rows(3.0, 12.0, 9.0, 1, 0.0, ink);
            rows(3.0, 21.0, 15.0, 1, 0.0, ink);
            line((19.0, 6.5), (19.0, 11.5), accent);
            line((19.0, 11.5), (14.5, 11.5), accent);
            head((12.5, 11.5), -1.0, 0.0);
        }
        _ if name.starts_with("brk_") => {
            page(5.0, 0.5, 19.0, 19.5);
            rows(7.0, 17.0, 3.5, 2, 3.0, ink);
            dashed(10.0, 2.0, 22.0, true);
            rows(7.0, 17.0, 13.5, 2, 3.0, ink);
            match name {
                "brk_evenPage" => digit(21.5, 15.5, "2", accent),
                "brk_oddPage" => digit(21.5, 15.5, "1", accent),
                "brk_continuous" => {}
                _ => head((21.5, 16.5), 0.0, 1.0),
            }
        }
        // Line Numbers: the lines and their numbers in the margin.
        _ if name.starts_with("ln") => {
            page(3.0, 0.5, 21.0, 19.5);
            rows(9.0, 19.0, 3.5, 5, 3.2, ink);
            let numbers: &[&str] = match name {
                "ln_continuous" | "lnOptions" => &["1", "2", "3", "4", "5"],
                "ln_restartPage" | "ln_restartSection" => &["1", "2", "3", "1", "2"],
                _ => &[],
            };
            for (i, n) in numbers.iter().enumerate() {
                digit(6.0, 3.5 + i as f32 * 3.2, n, accent);
            }
            if name == "ln_restartSection" {
                dashed(11.6, 8.0, 20.0, true);
            } else if name == "ln_restartPage" {
                line((3.0, 11.6), (21.0, 11.6), ink);
            }
        }
        // Page Number: where the number goes, and its options.
        "pn_top" | "pn_bottom" | "pn_xofy" | "pn_position" => {
            page(5.0, 0.5, 19.0, 19.5);
            rows(7.5, 16.5, 7.0, 3, 2.6, ink);
            match name {
                "pn_top" => digit(16.0, 3.5, "1", accent),
                "pn_bottom" => digit(12.0, 16.5, "1", accent),
                "pn_xofy" => digit(12.0, 16.5, "1/2", accent),
                _ => {
                    digit(15.0, 4.5, "1", accent);
                    line((15.0, 0.5), (15.0, 2.2), accent);
                    line((16.6, 4.5), (19.0, 4.5), accent);
                }
            }
        }
        "pn_current" => {
            rows(2.0, 22.0, 5.0, 1, 0.0, ink);
            line((2.0, 10.5), (9.0, 10.5), ink);
            p.rect_stroke(Rect::from_min_max(pt(10.0, 7.0), pt(15.0, 14.0)), 1.0, Stroke::new(1.0, accent), egui::StrokeKind::Middle);
            digit(12.5, 10.5, "1", accent);
            line((16.0, 10.5), (22.0, 10.5), ink);
            rows(2.0, 16.0, 16.0, 1, 0.0, ink);
        }
        "pn_format" => {
            digit(5.0, 5.0, "1", ink);
            digit(12.0, 5.0, "i", ink);
            digit(19.0, 5.0, "๑", accent);
            digit(5.0, 14.0, "2", ink);
            digit(12.0, 14.0, "ii", ink);
            digit(19.0, 14.0, "๒", accent);
        }
        // Text to table: lines of text turning into a grid.
        _ => return false,
    }
    true
}
fn draw_2(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, _accent: egui::Color32) -> bool {
    let accent = ink;
    let s = (r.width() / 24.0).min(r.height() / 24.0);
    let o = r.center() - vec2(12.0 * s, 12.0 * s);
    let pt = |x: f32, y: f32| egui::pos2(o.x + x * s, o.y + (y + 2.0) * s);
    let line = |a: (f32, f32), b: (f32, f32), c: egui::Color32| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new((1.75 * s).max(1.0), c));
        for point in [a, b] {
            p.circle_filled(pt(point.0, point.1), (1.75 * s).max(1.0) * 0.5, c);
        }
    };
    let page = |x0: f32, y0: f32, x1: f32, y1: f32| {
        p.rect_stroke(Rect::from_min_max(pt(x0, y0), pt(x1, y1)), 1.0, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
    };
    // Text lines across `x0..x1`, starting at `y`, every `gap`.
    let rows = |x0: f32, x1: f32, y: f32, n: usize, gap: f32, c: egui::Color32| {
        for i in 0..n {
            line((x0, y + i as f32 * gap), (x1, y + i as f32 * gap), c);
        }
    };
    let head = |tip: (f32, f32), dx: f32, dy: f32| {
        let t = pt(tip.0, tip.1);
        let back = vec2(-dx, -dy) * 3.0 * s;
        let side = vec2(-dy, dx) * 2.2 * s;
        p.add(egui::Shape::convex_polygon(vec![t, t + back + side, t + back - side], accent, Stroke::NONE));
    };
    match name {
        "textToTable" => {
            rows(2.0, 10.0, 5.0, 3, 4.0, ink);
            line((11.5, 9.0), (14.0, 9.0), accent);
            head((15.5, 9.0), 1.0, 0.0);
            p.rect_stroke(Rect::from_min_max(pt(16.5, 3.0), pt(23.0, 15.0)), 0.5, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
            line((16.5, 9.0), (23.0, 9.0), ink);
            line((19.75, 3.0), (19.75, 15.0), ink);
        }
        // Margins: the page with its text area outlined, as wide as the preset leaves it.
        _ if name.starts_with("mg_") => {
            let margin_box = |x0: f32, y0: f32, x1: f32, y1: f32| {
                let pts = [pt(x0, y0), pt(x1, y0), pt(x1, y1), pt(x0, y1), pt(x0, y0)];
                p.extend(egui::Shape::dashed_line(&pts, Stroke::new(1.0, accent), 1.6 * s, 1.2 * s));
            };
            if name == "mg_mirrored" {
                // Facing pages: the inside margins are the wider ones.
                page(0.5, 1.5, 11.5, 18.5);
                page(12.5, 1.5, 23.5, 18.5);
                margin_box(1.8, 3.5, 8.5, 16.5);
                margin_box(15.5, 3.5, 22.2, 16.5);
            } else {
                page(5.0, 0.5, 19.0, 19.5);
                // Left, top, right and bottom insets, in the picture's units.
                let (l, t, r, b) = match name {
                    "mg_narrow" => (1.3, 1.3, 1.3, 1.3),
                    "mg_moderate" => (1.8, 2.6, 1.8, 2.6),
                    "mg_wide" => (4.2, 2.6, 4.2, 2.6),
                    "mg_office2003" => (3.2, 2.6, 3.2, 2.6),
                    _ => (2.6, 2.6, 2.6, 2.6),
                };
                margin_box(5.0 + l, 0.5 + t, 19.0 - r, 19.5 - b);
                rows(5.0 + l + 0.8, 19.0 - r - 0.8, 0.5 + t + 2.0, 3, 2.6, ink);
                if name == "mg_custom" {
                    for x in [9.0, 12.0, 15.0] {
                        p.circle_filled(pt(x, 15.0), 1.0 * s, accent);
                    }
                }
            }
        }
        // Borders: a 2 × 2 table; the borders the choice sets are solid, the rest dotted.
        _ if name.starts_with("tb_") => {
            let k = name.strip_prefix("tb_").unwrap_or("");
            let outer = matches!(k, "all" | "outside");
            let inner = matches!(k, "all" | "inside");
            let edge = |a: (f32, f32), b: (f32, f32), on: bool| {
                if on {
                    p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new(1.7 * s.max(0.8), accent));
                } else {
                    // Dark enough to read at 1× (a faint 0.8 pt dotted line vanished there).
                    p.extend(egui::Shape::dashed_line(&[pt(a.0, a.1), pt(b.0, b.1)], Stroke::new(1.0, ink.gamma_multiply(0.85)), 1.6 * s, 1.2 * s));
                }
            };
            edge((4.0, 2.0), (20.0, 2.0), outer || k == "top");
            edge((4.0, 18.0), (20.0, 18.0), outer || k == "bottom");
            edge((4.0, 2.0), (4.0, 18.0), outer || k == "left");
            edge((20.0, 2.0), (20.0, 18.0), outer || k == "right");
            edge((4.0, 10.0), (20.0, 10.0), inner || k == "insideH");
            edge((12.0, 2.0), (12.0, 18.0), inner || k == "insideV");
            if k == "diagDown" {
                edge((4.0, 2.0), (20.0, 18.0), true);
            } else if k == "diagUp" {
                edge((4.0, 18.0), (20.0, 2.0), true);
            }
        }
        // Horizontal Line: text above and below a full-width rule.
        "hline" => {
            rows(4.0, 20.0, 3.5, 2, 3.0, ink);
            p.line_segment([pt(2.0, 10.5), pt(22.0, 10.5)], Stroke::new(2.0 * s.max(0.8), accent));
            rows(4.0, 20.0, 14.5, 2, 3.0, ink);
        }
        // View Gridlines: a table's cells outlined with dotted lines.
        "gridlines" => {
            for y in [2.0, 10.0, 18.0] {
                p.extend(egui::Shape::dashed_line(&[pt(4.0, y), pt(20.0, y)], Stroke::new(1.3, accent), 1.6 * s, 1.2 * s));
            }
            for x in [4.0, 12.0, 20.0] {
                p.extend(egui::Shape::dashed_line(&[pt(x, 2.0), pt(x, 18.0)], Stroke::new(1.3, accent), 1.6 * s, 1.2 * s));
            }
        }
        // Table Layout › Select and Delete: a 3 × 3 table with the cell, column, row or table
        // shaded; Delete crosses it out.
        _ if name.starts_with("sel_") || name.starts_with("del_") => {
            let (x0, y0, x1, y1) = match name.strip_prefix("sel_").or_else(|| name.strip_prefix("del_")).unwrap_or("") {
                "cell" => (9.0, 7.5, 15.0, 12.5),
                "column" => (9.0, 2.0, 15.0, 18.0),
                "row" => (3.0, 7.5, 21.0, 12.5),
                _ => (3.0, 2.0, 21.0, 18.0),
            };
            p.rect_filled(Rect::from_min_max(pt(x0, y0), pt(x1, y1)), 0.0, accent.gamma_multiply(0.6));
            p.rect_stroke(Rect::from_min_max(pt(3.0, 2.0), pt(21.0, 18.0)), 0.5, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
            for x in [9.0, 15.0] {
                line((x, 2.0), (x, 18.0), ink);
            }
            for y in [7.5, 12.5] {
                line((3.0, y), (21.0, y), ink);
            }
            if name.starts_with("del_") {
                let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                // In `ink`, so it shows on the shaded cells.
                let h = 2.8;
                p.line_segment([pt(cx - h, cy - h), pt(cx + h, cy + h)], Stroke::new(2.0 * s.max(0.8), ink));
                p.line_segment([pt(cx - h, cy + h), pt(cx + h, cy - h)], Stroke::new(2.0 * s.max(0.8), ink));
            }
        }
        // Table Layout › AutoFit: the table drawn in to its text, out to the page's edges, or with
        // fixed column widths marked on a ruler.
        "fit_contents" => {
            p.rect_stroke(Rect::from_min_max(pt(8.0, 5.0), pt(16.0, 15.0)), 0.5, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
            line((12.0, 5.0), (12.0, 15.0), ink);
            rows(9.2, 10.8, 8.5, 2, 3.0, ink);
            rows(13.2, 14.8, 8.5, 2, 3.0, ink);
            line((1.5, 10.0), (4.5, 10.0), accent);
            head((6.5, 10.0), 1.0, 0.0);
            line((22.5, 10.0), (19.5, 10.0), accent);
            head((17.5, 10.0), -1.0, 0.0);
        }
        "fit_window" => {
            line((1.0, 1.0), (1.0, 19.0), ink);
            line((23.0, 1.0), (23.0, 19.0), ink);
            p.rect_stroke(Rect::from_min_max(pt(2.5, 7.0), pt(21.5, 17.0)), 0.5, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
            line((12.0, 7.0), (12.0, 17.0), ink);
            line((10.0, 3.5), (5.5, 3.5), accent);
            head((3.5, 3.5), -1.0, 0.0);
            line((14.0, 3.5), (18.5, 3.5), accent);
            head((20.5, 3.5), 1.0, 0.0);
        }
        _ => return false,
    }
    true
}
fn draw_3(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, _accent: egui::Color32) -> bool {
    let accent = ink;
    let s = (r.width() / 24.0).min(r.height() / 24.0);
    let o = r.center() - vec2(12.0 * s, 12.0 * s);
    let pt = |x: f32, y: f32| egui::pos2(o.x + x * s, o.y + (y + 2.0) * s);
    let line = |a: (f32, f32), b: (f32, f32), c: egui::Color32| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new((1.75 * s).max(1.0), c));
        for point in [a, b] {
            p.circle_filled(pt(point.0, point.1), (1.75 * s).max(1.0) * 0.5, c);
        }
    };
    // Text lines across `x0..x1`, starting at `y`, every `gap`.
    match name {
        "fit_fixed" => {
            p.rect_stroke(Rect::from_min_max(pt(3.0, 7.0), pt(21.0, 17.0)), 0.5, Stroke::new(1.1, ink), egui::StrokeKind::Middle);
            for x in [9.0, 15.0] {
                line((x, 7.0), (x, 17.0), ink);
            }
            line((3.0, 3.5), (21.0, 3.5), accent);
            for x in [3.0, 9.0, 15.0, 21.0] {
                line((x, 1.5), (x, 5.0), accent);
            }
        }
        _ => return false,
    }
    true
}
