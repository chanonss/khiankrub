//! Preview-only gold treatments, applied to rendered shapes without changing the shared library.
use egui::{Align2, Color32, FontId, Rect, Shape, pos2, vec2};

fn gold(color: &mut Color32, variant: usize) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if variant == 0 || a == 0 || r < 170 || g < 100 || b > 80 || r <= g {
        return;
    }
    let opacity = if variant == 1 {
        if a >= 48 { 230 } else { 107 }
    } else if a >= 48 {
        255
    } else {
        166
    };
    *color = Color32::from_rgba_unmultiplied(242, 183, 5, opacity);
}

fn recolor(shape: &mut Shape, variant: usize) {
    match shape {
        Shape::Rect(s) => gold(&mut s.fill, variant),
        Shape::Circle(s) => gold(&mut s.fill, variant),
        Shape::Path(s) => gold(&mut s.fill, variant),
        Shape::Mesh(mesh) => {
            for vertex in &mut std::sync::Arc::make_mut(mesh).vertices {
                gold(&mut vertex.color, variant);
            }
        }
        Shape::Vec(shapes) => {
            for shape in shapes {
                recolor(shape, variant);
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).ok_or("usage: gold_comparison output.png")?;
    let mut h = egui_kittest::Harness::builder().with_size(vec2(1080.0, 760.0)).with_pixels_per_point(1.0).wgpu().build_ui(|ui| {
        let p = ui.painter();
        let ink = Color32::from_rgb(37, 51, 68);
        p.rect_filled(Rect::from_min_size(pos2(0.0, 0.0), vec2(1080.0, 760.0)), 0.0, Color32::from_rgb(245, 247, 249));
        p.text(pos2(28.0, 30.0), Align2::LEFT_CENTER, "Gold accents · colour proof", FontId::proportional(26.0), ink);
        p.text(pos2(28.0, 63.0), Align2::LEFT_CENTER, "Same artwork at 32 px and 16 px", FontId::proportional(18.0), ink);
        let icons = [("paste", "Paste"), ("copy", "Copy"), ("painter", "Painter"), ("table", "Table"), ("chart", "Chart"), ("picture", "Picture")];
        for dark in [false, true] {
            let top = if dark { 412.0 } else { 92.0 };
            let foreground = if dark { Color32::from_rgb(235, 240, 245) } else { ink };
            if dark {
                p.rect_filled(Rect::from_min_size(pos2(0.0, 392.0), vec2(1080.0, 368.0)), 0.0, Color32::from_rgb(31, 40, 49));
            }
            for (n, (_, label)) in icons.iter().enumerate() {
                p.text(pos2(259.0 + n as f32 * 135.0, top), Align2::CENTER_CENTER, *label, FontId::proportional(18.0), foreground);
            }
            for (variant, label) in ["Current", "Balanced", "Bold gold"].iter().enumerate() {
                let y = top + 56.0 + variant as f32 * 88.0;
                p.text(pos2(24.0, y), Align2::LEFT_CENTER, *label, FontId::proportional(18.0), foreground);
                for (n, (name, _)) in icons.iter().enumerate() {
                    let x = 220.0 + n as f32 * 135.0;
                    let start = ui.ctx().graphics_mut(|g| g.get_mut(p.layer_id()).map(|list| list.next_idx().0)).unwrap_or(0);
                    for (size, dx) in [(32.0, 0.0), (16.0, 52.0)] {
                        krub_icons::paint_named(
                            p,
                            Rect::from_min_size(pos2(x + dx, y - size * 0.5), vec2(size, size)),
                            name,
                            foreground,
                            false,
                            false,
                        );
                    }
                    ui.ctx().graphics_mut(|g| {
                        if let Some(list) = g.get_mut(p.layer_id()) {
                            let end = list.next_idx().0;
                            for index in start..end {
                                list.mutate_shape(egui::layers::ShapeIdx(index), |entry| recolor(&mut entry.shape, variant));
                            }
                        }
                    });
                }
            }
        }
    });
    h.run_steps(3);
    h.render()?.save(output)?;
    Ok(())
}
