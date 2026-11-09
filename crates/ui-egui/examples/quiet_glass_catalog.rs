//! Render the complete shared catalog without opening a window.
use egui::{Align2, Color32, FontId, Rect, pos2, vec2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args().nth(1).ok_or("usage: catalog output.png [filter]")?;
    let filter = std::env::args().nth(2).unwrap_or_default();
    let names: Vec<String> = krub_icons::catalog().into_iter().filter(|name| name.contains(&filter)).collect();
    println!("Rendering {} shared icon names", names.len());
    let columns = 12;
    let rows = names.len().div_ceil(columns);
    let width = columns as f32 * 100.0;
    let height = rows as f32 * 72.0;
    let mut harness = egui_kittest::Harness::builder().with_size(vec2(width, height)).with_pixels_per_point(1.0).wgpu().build_ui(|ui| {
        let p = ui.painter();
        p.rect_filled(Rect::from_min_size(pos2(0.0, 0.0), vec2(width, height)), 0.0, Color32::from_rgb(245, 247, 249));
        for (n, name) in names.iter().enumerate() {
            let x = (n % columns) as f32 * 100.0;
            let y = (n / columns) as f32 * 72.0;
            krub_icons::paint_named(
                p,
                Rect::from_min_size(pos2(x + 12.0, y + 8.0), vec2(28.0, 28.0)),
                name,
                Color32::from_rgb(40, 51, 60),
                false,
                false,
            );
            krub_icons::paint_named(
                p,
                Rect::from_min_size(pos2(x + 52.0, y + 14.0), vec2(16.0, 16.0)),
                name,
                Color32::from_rgb(40, 51, 60),
                false,
                false,
            );
            p.text(pos2(x + 50.0, y + 52.0), Align2::CENTER_CENTER, name, FontId::proportional(9.0), Color32::from_rgb(40, 51, 60));
        }
    });
    harness.run_steps(3);
    harness.render()?.save(out)?;
    Ok(())
}
