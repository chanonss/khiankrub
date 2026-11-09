//! File › Print (#641): the print settings and a preview of the pages.
//!
//! Page range and parity shape the PDF. Native printer settings and copy collation go to
//! the shared spooler; browser printing and saved print PDFs retain copies in the PDF.

use crate::widgets::SteadyUi;
use std::sync::Arc;

use egui::{Rect, ScrollArea, Sense, Stroke, TextureHandle, Ui, pos2, vec2};
use serde_json::{Value, json};
use wordcraft_layout::DocLayout;

use crate::theme::{Tokens, medium, regular};
use crate::{WordApp, icons};

/// The Print page's settings. Not saved with the document or the UI state.
#[derive(Clone, Debug, PartialEq)]
pub struct PrintSettings {
    /// `all`, `current`, `selection` or `custom`.
    pub range: String,
    /// The custom range text, like `1-3, 5`.
    pub pages: String,
    /// `all`, `odd` or `even`.
    pub parity: String,
    pub copies: u32,
    pub collate: bool,
    /// The page shown in the preview (0-based).
    pub preview: usize,
    /// `auto`, `portrait` or `landscape`.
    pub orientation: String,
    /// The preview's zoom (1.0 is the page's real size), or `None` to fit the whole page.
    pub zoom: Option<f32>,
    /// Dedicated preview state, kept separate from document settings.
    pub preview_zoom: Option<f32>,
    pub preview_effective_zoom: f32,
    pub preview_fit_width: bool,
    pub preview_columns: usize,
    pub preview_rows: usize,
    pub preview_arrangement: PreviewArrangement,
    pub preview_return_to_print: bool,
    /// Scroll through every page continuously (rows of pages, one after another), not one
    /// screenful of pages at a time.
    pub preview_continuous: bool,
    /// A page to scroll to on the next frame (continuous view).
    pub preview_scroll_to: Option<usize>,
    /// What the continuous view's scroll position was last matched to: zoom, arrangement and
    /// columns. When it changes the view keeps the current page in place.
    pub preview_key: Option<(u32, PreviewArrangement, usize)>,
    /// The File page showing before the preview opened; closing the preview restores it, so the
    /// File tab doesn't reopen the preview. Empty when none.
    pub preview_prev_page: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewArrangement {
    #[default]
    Grid,
    Facing,
    FacingWithCover,
}

impl Default for PrintSettings {
    fn default() -> Self {
        PrintSettings {
            range: "all".into(),
            pages: String::new(),
            parity: "all".into(),
            copies: 1,
            collate: true,
            preview: 0,
            zoom: None,
            preview_zoom: None,
            preview_effective_zoom: 1.0,
            preview_fit_width: false,
            preview_columns: 1,
            preview_rows: 1,
            preview_arrangement: PreviewArrangement::Grid,
            preview_return_to_print: false,
            preview_continuous: true,
            preview_scroll_to: None,
            preview_key: None,
            preview_prev_page: String::new(),
            orientation: "auto".into(),
        }
    }
}

impl PrintSettings {
    /// The params of `ui.print` and `file.printPdf` for these settings.
    pub fn params(&self) -> Value {
        let mut v = json!({
            "range": self.range,
            "parity": self.parity,
            "copies": self.copies,
            "collate": self.collate,
            "orientation": self.orientation,
        });
        if self.range == "custom" {
            v["pages"] = json!(self.pages);
        }
        v
    }
}

const RANGES: [(&str, &str); 4] = [("all", "All pages"), ("current", "Current page"), ("selection", "Selected pages"), ("custom", "Custom range")];
const PARITIES: [(&str, &str); 3] = [("all", "All pages in the range"), ("odd", "Odd pages only"), ("even", "Even pages only")];
const ORIENTATIONS: [(&str, &str); 3] = [("auto", "Auto orientation"), ("portrait", "Portrait"), ("landscape", "Landscape")];

/// A choice list over `(id, label)` pairs; true when the choice changed.
fn choice(ui: &mut Ui, salt: &str, current: &mut String, items: &[(&str, &str)]) -> bool {
    let shown = items.iter().find(|(id, _)| *id == current.as_str()).map_or("", |(_, l)| *l);
    let mut changed = false;
    egui::ComboBox::from_id_salt(salt).width(260.0).selected_text(tl!(shown)).show_ui(ui, |ui| {
        for (id, label) in items {
            if ui.steady_label(current == id, tl!(label)).clicked() && current != id {
                *current = (*id).to_string();
                changed = true;
            }
        }
    });
    changed
}

/// The output layout, kept while the document is unchanged (laying out every frame would be slow).
fn layout_of(app: &WordApp, ctx: &egui::Context) -> Arc<DocLayout> {
    let id = egui::Id::new("print_page_layout");
    let rev = app.session.rev();
    if let Some((r, l)) = ctx.data(|d| d.get_temp::<(u64, Arc<DocLayout>)>(id))
        && r == rev
    {
        return l;
    }
    let l = app.session.export_layout();
    ctx.data_mut(|d| d.insert_temp(id, (rev, l.clone())));
    l
}

/// The texture of one page, re-rendered only when the document, page or size changes.
fn page_texture(app: &WordApp, ctx: &egui::Context, lay: &DocLayout, page: usize, width: f32) -> Option<TextureHandle> {
    // Bound the cache even for documents with many pages.
    let id = egui::Id::new(("print_page_texture", page % 16));
    let key = (app.session.rev(), page, width as u32, (ctx.pixels_per_point() * 100.0) as u32);
    if let Some((k, t)) = ctx.data(|d| d.get_temp::<((u64, usize, u32, u32), TextureHandle)>(id))
        && k == key
    {
        return Some(t);
    }
    let pg = lay.pages.get(page)?;
    // Zoomed in, the texture stays within what the GPU takes and is stretched beyond that.
    let max_side = ctx.input(|i| i.max_texture_side) as f32 / pg.w.max(pg.h).max(1.0);
    let scale = (width / pg.w.max(1.0) * ctx.pixels_per_point()).clamp(0.05, 4.0).min(max_side);
    let img = wordcraft_render::render_page(&app.session.doc, pg, scale, &Default::default());
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &img.to_straight());
    let tex = ctx.load_texture("print_page_preview", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, (key, tex.clone())));
    Some(tex)
}

/// The Print page: settings on the left, the preview on the right.
pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let lay = layout_of(app, &ctx);
    let total = lay.pages.len();
    let current = app.session.current_pages(&lay);
    app.ui.print.preview = app.ui.print.preview.min(total.saturating_sub(1));

    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(380.0);
            // The settings scroll on their own: the preview beside them keeps the whole height (#1156).
            egui::ScrollArea::vertical().id_salt("print_settings").show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                let native = app.services.print_native.is_some();
                let hook = native || app.services.print.is_some();
                if native {
                    if let Some(discover) = &app.services.printers {
                        crate::printing::show(&mut app.printing, ui, discover, |label| crate::i18n::t(label).to_owned());
                    }
                    app.ui.print.copies = app.printing.settings.copies;
                } else if hook {
                    ui.label(tl!("Send the document straight to the system print dialog, or save a copy in another format below."));
                    ui.label(
                        egui::RichText::new(tl!("The printer, two-sided printing and pages per sheet are chosen in that dialog.")).color(t.text_dim),
                    );
                } else {
                    ui.label(tl!("Save a copy in another format. Printing goes through a PDF you can print from any viewer."));
                }
                ui.add_space(4.0);
                if ui.button(tl!("Open Print Preview")).clicked() {
                    let _ = app.run("ui.printPreview", json!({"value": true, "returnToPrint": true}));
                }
                if hook
                    && ui
                        .add_enabled(
                            !native || app.printing.ready(),
                            egui::Button::new(egui::RichText::new(tl!("Print")).font(medium(13.5))).min_size(vec2(320.0, 34.0)),
                        )
                        .clicked()
                {
                    let params = app.ui.print.params();
                    let _ = app.run("ui.print", params);
                }
                if ui
                    .add(egui::Button::new(egui::RichText::new(tl!("Save these pages as PDF…")).font(medium(13.5))).min_size(vec2(320.0, 34.0)))
                    .clicked()
                {
                    app.print_pdf_dialog();
                }
                ui.add_space(6.0);

                if native {
                    if app.ui.print.copies > 1 {
                        ui.add_enabled(!app.printing.busy(), egui::Checkbox::new(&mut app.ui.print.collate, tl!("Collate copies")));
                    }
                } else {
                    ui.label(egui::RichText::new(tl!("Copies")).font(medium(13.0)));
                    ui.horizontal(|ui| {
                        let mut n = app.ui.print.copies;
                        ui.add(crate::numfield::NumField::new(&mut n).range(1..=wordcraft_engine::print::MAX_COPIES).speed(0.2));
                        app.ui.print.copies = n.clamp(1, wordcraft_engine::print::MAX_COPIES);
                        if app.ui.print.copies > 1 {
                            ui.checkbox(&mut app.ui.print.collate, tl!("Collate copies"));
                        }
                    });
                }

                ui.label(egui::RichText::new(tl!("Pages")).font(medium(13.0)));
                if choice(ui, "print_range", &mut app.ui.print.range, &RANGES) {
                    // Show the first page that will print.
                    app.ui.print.preview = if app.ui.print.range == "all" { 0 } else { current.0.min(total.saturating_sub(1)) };
                }
                if app.ui.print.range == "custom" {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.ui.print.pages)
                            .hint_text(tl!("Pages like 1-3, 5"))
                            .desired_width(260.0)
                            .char_limit(wordcraft_engine::print::MAX_SPEC_CHARS),
                    );
                }
                choice(ui, "print_parity", &mut app.ui.print.parity, &PARITIES);
                ui.label(egui::RichText::new(tl!("Orientation")).font(medium(13.0)));
                choice(ui, "print_orientation", &mut app.ui.print.orientation, &ORIENTATIONS);

                match wordcraft_engine::print::plan(&app.ui.print.params(), total, current) {
                    Ok(plan) => {
                        let msg = crate::i18n::fmt(
                            tl!("{count} of {total} pages will print"),
                            &[("count", &plan.chosen.to_string()), ("total", &total.to_string())],
                        );
                        ui.label(egui::RichText::new(msg).color(t.text_dim));
                    }
                    Err(e) => {
                        ui.label(egui::RichText::new(e.to_string()).color(t.red));
                    }
                }
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                if hook {
                    ui.label(tl!("Or save a copy in another format:"));
                }
                export_formats(app, ui);
            });
        });
        ui.add_space(24.0);
        ui.vertical(|ui| preview(app, ui, &lay, total));
    });
}

/// Room around the page inside the preview pane.
const PREVIEW_MARGIN: f32 = 10.0;
/// The preview's bottom bar: the page indicator on the left, the zoom on the right.
const PREVIEW_BAR: f32 = 34.0;

/// Page indices in the current spread. `None` is the empty left side before a cover.
fn preview_slots(total: usize, page: usize, columns: usize, rows: usize, arrangement: PreviewArrangement) -> Vec<Option<usize>> {
    if total == 0 {
        return Vec::new();
    }
    let page = page.min(total - 1);
    let count = if arrangement == PreviewArrangement::Grid { columns.clamp(1, 4) * rows.clamp(1, 4) } else { 2 };
    let start = match arrangement {
        PreviewArrangement::Grid | PreviewArrangement::Facing => page / count * count,
        PreviewArrangement::FacingWithCover if page == 0 => return vec![None, Some(0)],
        PreviewArrangement::FacingWithCover => 1 + (page - 1) / 2 * 2,
    };
    (0..count).map(|offset| start.checked_add(offset).filter(|index| *index < total)).collect()
}

/// Every page of the document in the arrangement's rows, for the continuous view, and the number
/// of pages per row. Single pages are a grid of one column; the cover stands alone on the right.
fn continuous_slots(total: usize, columns: usize, arrangement: PreviewArrangement) -> (Vec<Option<usize>>, usize) {
    match arrangement {
        PreviewArrangement::Grid => {
            let columns = columns.clamp(1, 4);
            ((0..total).map(Some).collect(), columns)
        }
        PreviewArrangement::Facing => ((0..total).map(Some).collect(), 2),
        PreviewArrangement::FacingWithCover => {
            let mut slots = Vec::with_capacity(total + 1);
            if total > 0 {
                slots.push(None);
            }
            slots.extend((0..total).map(Some));
            (slots, 2)
        }
    }
}

/// Full-window, read-only print preview. Page arrangement only affects this view.
pub fn show_full_preview(app: &mut WordApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let lay = layout_of(app, &ctx);
    let total = lay.pages.len();
    app.ui.print.preview = app.ui.print.preview.min(total.saturating_sub(1));
    egui::Panel::top("full_print_preview_toolbar").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button(tl!("Close Preview")).clicked() {
                let _ = app.run("ui.printPreview", json!({"value": false}));
            }
            if ui.button(tl!("Print")).clicked() {
                app.ui.backstage_page = "print".into();
            }
            ui.separator();
            ui.label(tl!("Layout"));
            egui::ComboBox::from_id_salt("preview_arrangement")
                .selected_text(tl!(match app.ui.print.preview_arrangement {
                    PreviewArrangement::Grid => "Page grid",
                    PreviewArrangement::Facing => "Facing pages",
                    PreviewArrangement::FacingWithCover => "Facing pages with cover",
                }))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.ui.print.preview_arrangement, PreviewArrangement::Grid, tl!("Page grid"));
                    ui.selectable_value(&mut app.ui.print.preview_arrangement, PreviewArrangement::Facing, tl!("Facing pages"));
                    ui.selectable_value(&mut app.ui.print.preview_arrangement, PreviewArrangement::FacingWithCover, tl!("Facing pages with cover"));
                });
            if app.ui.print.preview_arrangement == PreviewArrangement::Grid {
                ui.label(tl!("Columns"));
                ui.add(egui::DragValue::new(&mut app.ui.print.preview_columns).range(1..=4));
                if !app.ui.print.preview_continuous {
                    ui.label(tl!("Rows"));
                    ui.add(egui::DragValue::new(&mut app.ui.print.preview_rows).range(1..=4));
                }
            }
            ui.checkbox(&mut app.ui.print.preview_continuous, tl!("Continuous scroll"));
            ui.separator();
            let slots = if app.ui.print.preview_continuous {
                // The row of pages the view is on.
                let (all, columns) = continuous_slots(total, app.ui.print.preview_columns, app.ui.print.preview_arrangement);
                let at = all.iter().position(|slot| *slot == Some(app.ui.print.preview)).unwrap_or(0);
                all.chunks(columns).nth(at / columns).map(<[_]>::to_vec).unwrap_or_default()
            } else {
                preview_slots(total, app.ui.print.preview, app.ui.print.preview_columns, app.ui.print.preview_rows, app.ui.print.preview_arrangement)
            };
            let first = slots.iter().flatten().next().copied().unwrap_or(0);
            let last = slots.iter().flatten().last().copied().unwrap_or(0);
            if ui.add_enabled(first > 0, egui::Button::new("‹")).on_hover_text(tl!("Previous Page")).clicked() {
                // A row back: the pages the row above shows.
                let row_back = if app.ui.print.preview_continuous {
                    let (all, columns) = continuous_slots(total, app.ui.print.preview_columns, app.ui.print.preview_arrangement);
                    let at = all.iter().position(|slot| *slot == Some(first)).unwrap_or(0);
                    all.chunks(columns).nth((at / columns).saturating_sub(1)).and_then(|row| row.iter().flatten().next().copied())
                } else {
                    None
                };
                app.ui.print.preview = row_back.unwrap_or_else(|| first.saturating_sub(1));
                app.ui.print.preview_scroll_to = Some(app.ui.print.preview);
            }
            ui.label(crate::i18n::fmt(
                tl!("Page {page} of {pages}"),
                &[("page", &(first + usize::from(total > 0)).to_string()), ("pages", &total.to_string())],
            ));
            if ui.add_enabled(last + 1 < total, egui::Button::new("›")).on_hover_text(tl!("Next Page")).clicked() {
                app.ui.print.preview = last + 1;
                app.ui.print.preview_scroll_to = Some(last + 1);
            }
            ui.separator();
            if ui.button("−").on_hover_text(tl!("Zoom Out")).clicked() {
                app.ui.print.preview_fit_width = false;
                app.ui.print.preview_zoom = Some(zoom_step(app.ui.print.preview_effective_zoom, false));
            }
            let zoom_label = if app.ui.print.preview_fit_width {
                tl!("Page Width").to_owned()
            } else {
                app.ui.print.preview_zoom.map_or_else(|| tl!("Fit pages").to_owned(), |z| format!("{}%", (z * 100.0).round()))
            };
            egui::ComboBox::from_id_salt("full_preview_zoom").selected_text(zoom_label).show_ui(ui, |ui| {
                if ui.selectable_label(app.ui.print.preview_zoom.is_none() && !app.ui.print.preview_fit_width, tl!("Fit pages")).clicked() {
                    app.ui.print.preview_zoom = None;
                    app.ui.print.preview_fit_width = false;
                }
                if ui.selectable_label(app.ui.print.preview_fit_width, tl!("Page Width")).clicked() {
                    app.ui.print.preview_zoom = None;
                    app.ui.print.preview_fit_width = true;
                }
                for pct in [25, 50, 75, 100, 150, 200] {
                    if ui.button(format!("{pct}%")).clicked() {
                        app.ui.print.preview_fit_width = false;
                        app.ui.print.preview_zoom = Some(pct as f32 / 100.0);
                    }
                }
            });
            if ui.button("+").on_hover_text(tl!("Zoom In")).clicked() {
                app.ui.print.preview_fit_width = false;
                app.ui.print.preview_zoom = Some(zoom_step(app.ui.print.preview_effective_zoom, true));
            }
        });
    });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
        let continuous = app.ui.print.preview_continuous;
        let (slots, columns) = if continuous {
            continuous_slots(total, app.ui.print.preview_columns, app.ui.print.preview_arrangement)
        } else {
            let slots =
                preview_slots(total, app.ui.print.preview, app.ui.print.preview_columns, app.ui.print.preview_rows, app.ui.print.preview_arrangement);
            let columns = if app.ui.print.preview_arrangement == PreviewArrangement::Grid { app.ui.print.preview_columns.clamp(1, 4) } else { 2 };
            (slots, columns)
        };
        let rows = slots.len().div_ceil(columns).max(1);
        // The zoom to fit is by the largest page in the document, so it stays put as you scroll.
        let sizing: Vec<usize> = if continuous { (0..total).collect() } else { slots.iter().flatten().copied().collect() };
        let (pw, ph) = sizing.iter().filter_map(|i| lay.pages.get(*i)).fold((1.0_f32, 1.0_f32), |(w, h), page| (w.max(page.w), h.max(page.h)));
        let gap = 12.0;
        let available = ui.available_size();
        let fit_rows = if continuous { 1 } else { rows };
        let fit_width = ((available.x - gap * (columns + 1) as f32) / (pw * columns as f32)).max(0.01);
        let fit = if app.ui.print.preview_fit_width {
            fit_width
        } else {
            fit_width.min((available.y - gap * (fit_rows + 1) as f32) / (ph * fit_rows as f32)).max(0.01)
        };
        let scale = app.ui.print.preview_zoom.map_or(fit, |z| z.clamp(0.1, 5.0) * crate::canvas::PX_PER_PT);
        app.ui.print.preview_effective_zoom = scale / crate::canvas::PX_PER_PT;
        // Each row is as tall as its tallest page; `tops[r]` is where row `r` starts below the area's top.
        let row_heights: Vec<f32> = slots
            .chunks(columns)
            .map(|row| row.iter().flatten().filter_map(|i| lay.pages.get(*i)).fold(0.0_f32, |h, page| h.max(page.h)) * scale)
            .collect();
        let mut tops = Vec::with_capacity(row_heights.len());
        let mut y = gap;
        for h in &row_heights {
            tops.push(y);
            y += h + gap;
        }
        let grid = vec2(columns as f32 * pw * scale + (columns + 1) as f32 * gap, y.max(2.0 * gap));
        // When the zoom or arrangement changes, stay on the current page.
        let key = (scale.to_bits(), app.ui.print.preview_arrangement, columns);
        let moved = app.ui.print.preview_key.is_none_or(|old| old != key);
        app.ui.print.preview_key = Some(key);
        let row_of = |page: usize| slots.iter().position(|slot| *slot == Some(page)).map(|at| at / columns);
        let mut scroll = ScrollArea::both().id_salt("full_print_preview").auto_shrink(false);
        let jump = if continuous { app.ui.print.preview_scroll_to.take().or(moved.then_some(app.ui.print.preview)) } else { None };
        if !continuous {
            app.ui.print.preview_scroll_to = None;
        }
        if let Some(top) = jump.and_then(&row_of).and_then(|row| tops.get(row)) {
            scroll = scroll.vertical_scroll_offset((top - gap).max(0.0));
        }
        let out = scroll.show(ui, |ui| {
            let room = grid.max(ui.available_size());
            let (area, _) = ui.allocate_exact_size(room, Sense::hover());
            // Centered across, and down too while the pages are shorter than the pane.
            let origin = pos2(area.center().x - grid.x / 2.0 + gap, area.top() + ((room.y - grid.y) / 2.0).max(0.0));
            for (slot, page) in slots.iter().enumerate() {
                let Some(index) = *page else { continue };
                let Some(pg) = lay.pages.get(index) else { continue };
                let cell = pos2(origin.x + (slot % columns) as f32 * (pw * scale + gap), origin.y + tops[slot / columns] - gap);
                // A page narrower than its column stands in the middle of it.
                let cell = cell + vec2((pw - pg.w) * scale / 2.0, 0.0);
                let rect = Rect::from_min_size(cell, vec2(pg.w * scale, pg.h * scale));
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let label = crate::i18n::fmt(tl!("Page {page} of {pages}"), &[("page", &(index + 1).to_string()), ("pages", &total.to_string())]);
                ui.interact(rect, ui.id().with(("preview page", index)), Sense::hover())
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, &label));
                ui.painter().rect_filled(rect.translate(vec2(2.0, 3.0)), 0.0, t.page_shadow);
                ui.painter().rect(rect, 0.0, egui::Color32::WHITE, Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
                if let Some(texture) = page_texture(app, &ctx, &lay, index, rect.width()) {
                    ui.painter().image(texture.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
                }
            }
        });
        // The page indicator follows the scroll: the first row still on screen, unless a jump
        // is under way (the offset this frame is not yet where it is going).
        if continuous && jump.is_none() {
            let top = out.state.offset.y + out.inner_rect.height() * 0.25;
            let row = tops.iter().rposition(|t0| *t0 <= top).unwrap_or(0);
            let current = row_of(app.ui.print.preview);
            if current != Some(row)
                && let Some(page) = slots.chunks(columns).nth(row).and_then(|r| r.iter().flatten().next().copied())
            {
                app.ui.print.preview = page;
            }
        } else if jump.is_some() {
            ui.ctx().request_repaint();
        }
    });
}

/// The next preview zoom one step (10%) in or out, from 10% to 500% as in the status bar.
fn zoom_step(z: f32, up: bool) -> f32 {
    let z = (z * 10.0).round() / 10.0;
    (if up { z + 0.1 } else { z - 0.1 }).clamp(0.1, 5.0)
}

/// The preview (#1156): the page fills the pane, a bar below pages through the document and
/// zooms; zoomed in, the page scrolls both ways.
fn preview(app: &mut WordApp, ui: &mut Ui, lay: &DocLayout, total: usize) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let page = app.ui.print.preview;
    let (pw, ph) = lay.pages.get(page).map_or((612.0, 792.0), |p| (p.w.max(1.0), p.h.max(1.0)));
    let pane = vec2(ui.available_width().max(120.0), (ui.available_height() - PREVIEW_BAR).max(120.0));
    // Screen points per page point.
    let fit = ((pane.x - 2.0 * PREVIEW_MARGIN) / pw).min((pane.y - 2.0 * PREVIEW_MARGIN) / ph).max(0.01);
    let scale = app.ui.print.zoom.map_or(fit, |z| z * crate::canvas::PX_PER_PT);
    let size = vec2(pw * scale, ph * scale);
    let label = crate::i18n::fmt(tl!("Page {page} of {pages}"), &[("page", &(page + 1).to_string()), ("pages", &total.to_string())]);
    egui::ScrollArea::both().id_salt("print_preview").auto_shrink(false).max_width(pane.x).max_height(pane.y).show(ui, |ui| {
        let room = vec2((size.x + 2.0 * PREVIEW_MARGIN).max(ui.available_width()), (size.y + 2.0 * PREVIEW_MARGIN).max(ui.available_height()));
        let (area, _) = ui.allocate_exact_size(room, Sense::hover());
        let r = Rect::from_center_size(area.center(), size);
        let resp = ui.interact(r, ui.id().with("print_preview_page"), Sense::hover());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, &label));
        ui.painter().rect_filled(r.translate(vec2(2.0, 3.0)), 0.0, t.page_shadow);
        ui.painter().rect(r, 0.0, egui::Color32::WHITE, Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
        if let Some(tex) = page_texture(app, &ctx, lay, page, size.x) {
            ui.painter().image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
    });
    ui.horizontal(|ui| {
        ui.set_height(PREVIEW_BAR);
        let nav = |ui: &mut Ui, icon: &str, tip: &str, enabled: bool| {
            let (r, resp) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, tip));
            let col = if enabled { t.icon } else { t.text_disabled };
            if enabled && resp.hovered() {
                ui.painter().rect_filled(r, 3.0, t.hover);
            }
            icons::paint(ui.painter(), r.shrink(4.0), icon, col, col);
            enabled && resp.on_hover_text(tip).clicked()
        };
        if nav(ui, "chevronLeft", tl!("Previous Page"), page > 0) {
            app.ui.print.preview = page - 1;
        }
        ui.label(egui::RichText::new(&label).font(regular(13.0)));
        if nav(ui, "chevronRight", tl!("Next Page"), page + 1 < total) {
            app.ui.print.preview = page + 1;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let z = scale / crate::canvas::PX_PER_PT;
            if nav(ui, "onePage", tl!("One Page"), true) {
                app.ui.print.zoom = None;
            }
            ui.label(egui::RichText::new(format!("{}%", (z * 100.0).round())).font(regular(13.0)));
            if nav(ui, "plus", tl!("Zoom In"), z < 5.0) {
                app.ui.print.zoom = Some(zoom_step(z, true));
            }
            // A narrow window shortens the slider, then drops it, before it covers the page indicator.
            let room = ui.available_width() - 26.0 - 2.0 * ui.spacing().item_spacing.x - 12.0;
            if room >= 40.0 {
                let mut v = z;
                ui.spacing_mut().slider_width = room.min(110.0);
                if ui.add(egui::Slider::new(&mut v, 0.1..=5.0).logarithmic(true).show_value(false)).on_hover_text(tl!("Zoom")).changed() {
                    app.ui.print.zoom = Some(v.clamp(0.1, 5.0));
                }
            }
            if nav(ui, "minus", tl!("Zoom Out"), z > 0.1) {
                app.ui.print.zoom = Some(zoom_step(z, false));
            }
        });
    });
}

/// The export "format" of a Word 2003 XML Document: an `.xml` file saved with `format: "wordml2003"`.
pub(crate) const EXPORT_WORD_2003: &str = "wordml2003";

/// The "save a copy" formats (File › Export, and below the settings on the Print page).
pub fn export_formats(app: &mut WordApp, ui: &mut Ui) {
    for (label, ext) in [
        ("PDF document (*.pdf)", "pdf"),
        ("Word document (*.docx)", "docx"),
        ("Word template (*.dotx)", "dotx"),
        ("Word XML Document (*.xml)", "xml"),
        ("Word 2003 XML Document (*.xml)", EXPORT_WORD_2003),
        ("OpenDocument Text (*.odt)", "odt"),
        ("Rich Text Format (*.rtf)", "rtf"),
        ("Web page (*.html)", "html"),
        ("Markdown (*.md)", "md"),
        ("LaTeX (*.tex)", "tex"),
        ("Plain text (*.txt)", "txt"),
        ("Page image (*.png)", "png"),
    ] {
        if ui.add(egui::Button::new(egui::RichText::new(tl!(label)).font(medium(13.5))).min_size(vec2(320.0, 34.0))).clicked() {
            app.export_dialog(ext);
        }
        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod full_preview_tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn grids_and_facing_pages_group_every_page_once() {
        assert_eq!(preview_slots(0, 0, 2, 2, PreviewArrangement::Grid), vec![]);
        assert_eq!(preview_slots(5, 3, 2, 2, PreviewArrangement::Grid), vec![Some(0), Some(1), Some(2), Some(3)]);
        assert_eq!(preview_slots(5, 4, 2, 2, PreviewArrangement::Grid), vec![Some(4), None, None, None]);
        assert_eq!(preview_slots(5, 1, 4, 4, PreviewArrangement::Facing), vec![Some(0), Some(1)]);
        assert_eq!(preview_slots(5, 0, 4, 4, PreviewArrangement::FacingWithCover), vec![None, Some(0)]);
        assert_eq!(preview_slots(5, 2, 4, 4, PreviewArrangement::FacingWithCover), vec![Some(1), Some(2)]);
        assert_eq!(preview_slots(5, usize::MAX, 4, 4, PreviewArrangement::FacingWithCover), vec![Some(3), Some(4)]);
    }

    #[test]
    fn continuous_view_lists_every_page_in_rows() {
        assert_eq!(continuous_slots(3, 1, PreviewArrangement::Grid), (vec![Some(0), Some(1), Some(2)], 1));
        assert_eq!(continuous_slots(3, 9, PreviewArrangement::Grid).1, 4);
        assert_eq!(continuous_slots(3, 1, PreviewArrangement::Facing), (vec![Some(0), Some(1), Some(2)], 2));
        assert_eq!(continuous_slots(3, 1, PreviewArrangement::FacingWithCover), (vec![None, Some(0), Some(1), Some(2)], 2));
        assert_eq!(continuous_slots(0, 1, PreviewArrangement::FacingWithCover), (vec![], 2));
    }

    #[test]
    fn opening_and_closing_preserves_document_and_returns_to_origin() {
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let before = app.session.doc.clone();
        app.run("ui.printPreview", json!({"value": true})).unwrap();
        assert!(app.ui.backstage && app.ui.backstage_page == "printPreview");
        app.run("ui.printPreview", json!({"value": false})).unwrap();
        assert!(!app.ui.backstage);
        assert_ne!(app.ui.backstage_page, "printPreview", "the File tab must not reopen the preview");
        app.run("ui.printPreview", json!({"value": true, "returnToPrint": true})).unwrap();
        app.run("ui.printPreview", json!({"value": false})).unwrap();
        assert!(app.ui.backstage && app.ui.backstage_page == "print");
        assert_eq!(serde_json::to_value(&app.session.doc).unwrap(), serde_json::to_value(&before).unwrap());
    }

    #[test]
    fn preview_draws_and_close_returns_to_document() {
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("ui.printPreview", json!({"value": true})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1100.0, 800.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
        h.run_steps(5);
        assert!(h.query_by_label("Close Preview").is_some());
        h.get_by_label("Close Preview").click();
        h.run_steps(2);
        assert!(!h.state().ui.backstage);
    }
}
