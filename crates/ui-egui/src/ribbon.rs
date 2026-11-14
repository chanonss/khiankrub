//! The ribbon: tab strip and every tab's groups. Each control runs a command by id.

use crate::widgets::SteadyUi;
use egui::{Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::colors::{Remember, Target};
use crate::options::MeasureExt;
use crate::theme::{Tokens, medium, semibold};
use crate::widgets::{CONTENT_H, big, big_toggle, combo, group, menu_button, small, split, split_labeled};
use crate::{WordApp, icons};

pub use wordcraft_engine::ribbon::TABS;

/// True when the caret/selection touches a picture (#147).
pub fn has_picture_selected(s: &wordcraft_engine::Session) -> bool {
    matches!(wordcraft_engine::cmd::objects::selected(s), Some((_, wordcraft_doc::para::InlineObject::Image { .. })))
}

/// Whether the selection is a shape, text box or Drawing Canvas (so Shape Format is shown).
pub fn has_shape_selected(s: &wordcraft_engine::Session) -> bool {
    match wordcraft_engine::cmd::objects::selected(s) {
        Some((_, o)) => matches!(o, wordcraft_doc::para::InlineObject::Shape { .. }) || o.is_canvas(),
        None => false,
    }
}

/// Reading controls replace editing tabs entirely. All settings are transient view controls.
fn reading_toolbar(app: &mut WordApp, ui: &mut Ui, t: Tokens) {
    egui::Panel::top("reading_toolbar")
        .exact_size(46.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(12, 6)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(tl!("Reading Mode")).font(semibold(14.0)));
                ui.separator();
                let w = app.session.view.read_column_width;
                let active = match w {
                    None => "Auto",
                    Some(v) if v <= 180.0 => "Narrow",
                    Some(v) if v <= 240.0 => "Medium",
                    Some(v) if v <= 350.0 => "Wide",
                    _ => "Page",
                };
                ui.menu_button(format!("{}: {}", tl!("Column width"), tl!(active)), |ui| {
                    for (name, value) in [("Auto", "auto"), ("Narrow", "narrow"), ("Medium", "medium"), ("Wide", "wide"), ("Page", "page")] {
                        if ui.steady_label(active == name, tl!(name)).clicked() {
                            let _ = app.run("view.readColumnWidth", json!({"value": value}));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                let pct = (app.session.view.read_zoom * 100.0).round() as u32;
                ui.menu_button(format!("{}: {pct}%", tl!("Text size")), |ui| {
                    for percent in [75, 90, 100, 110, 125, 150] {
                        if ui.steady_label(pct == percent, format!("{percent}%")).clicked() {
                            let _ = app.run("view.readTextSize", json!({"value": percent}));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                let spacing = app.session.view.read_line_spacing;
                let label = spacing.map_or_else(|| tl!("Document").to_owned(), |v| format!("{v}×"));
                ui.menu_button(format!("{}: {}", tl!("Line spacing"), label), |ui| {
                    if ui.steady_label(spacing.is_none(), tl!("Document")).clicked() {
                        let _ = app.run("view.readLineSpacing", json!({"value": "document"}));
                        ui.close();
                    }
                    for value in [1.0, 1.25, 1.5, 2.0] {
                        if ui.steady_label(spacing == Some(value), format!("{value}×")).clicked() {
                            let _ = app.run("view.readLineSpacing", json!({"value": value}));
                            ui.close();
                        }
                    }
                });
                ui.separator();
                // Headings to jump to, in the Navigation pane beside the columns (Outline view
                // is an editing layout, so it isn't offered here).
                if ui.button(tl!("Navigation Pane")).clicked() {
                    let _ = app.run("view.navigationPane", json!({"value": true}));
                    app.ui.nav_tab = "headings".into();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(tl!("Edit Document")).clicked() {
                        let _ = app.run("view.printLayout", json!({}));
                    }
                });
            });
        });
}

/// Whether the selection is a SmartArt graphic WordCraft can edit (so SmartArt Design is shown).
pub fn has_smart_art_selected(s: &wordcraft_engine::Session) -> bool {
    wordcraft_engine::cmd::smart_art::selected_smart_art(s).is_some()
}

/// Whether the selection is a chart WordCraft can edit (so Chart Design is shown).
pub fn has_chart_selected(s: &wordcraft_engine::Session) -> bool {
    wordcraft_engine::cmd::chart::selected_chart(s).is_some()
}

/// Contextual tabs for the current selection (pure, tested).
pub fn contextual_tabs(s: &wordcraft_engine::Session) -> Vec<&'static str> {
    let mut tabs = Vec::new();
    if s.sel.focus.path.cell().is_some() {
        tabs.push("Table Design");
        tabs.push("Table Layout");
    }
    if has_picture_selected(s) {
        tabs.push("Picture Format");
    }
    if has_shape_selected(s) {
        tabs.push("Shape Format");
    }
    if has_smart_art_selected(s) {
        tabs.push("SmartArt Design");
    }
    if has_chart_selected(s) {
        tabs.push("Chart Design");
    }
    // Editing a header or footer.
    if s.in_header_footer() {
        tabs.push("Header & Footer");
    }
    // Editing an equation.
    if s.math.is_some() {
        tabs.push("Equation");
    }
    // Outline view (drawn first, next to File, as Word does).
    if s.view.mode == wordcraft_layout::ViewMode::Outline {
        tabs.push("Outlining");
    }
    tabs
}

/// Tab to show when the stored tab is no longer applicable (pure, tested).
pub fn resolve_tab<'a>(current: &'a str, available: &[&str]) -> &'a str {
    if available.contains(&current) { current } else { crate::DEFAULT_TAB }
}

/// Width of the » button that holds the tabs the strip has no room for.
const MORE_TABS_W: f32 = 24.0;

/// Which tabs fit on a strip `avail` points wide (#897), from each tab's width and whether it must
/// stay. The others give way from the end, leaving `more` points for the » menu that holds them
/// (pure, tested).
pub fn fit_tabs(tabs: &[(f32, bool)], avail: f32, more: f32) -> Vec<bool> {
    let mut fits = vec![true; tabs.len()];
    let mut total: f32 = tabs.iter().map(|(w, _)| w).sum();
    if total <= avail {
        return fits;
    }
    for (fit, (w, keep)) in fits.iter_mut().zip(tabs).rev() {
        if total + more <= avail {
            break;
        }
        if !keep {
            *fit = false;
            total -= w;
        }
    }
    fits
}

const TAB_NUMBER_SPACE: f32 = 21.0;

fn band_action(ui: &mut Ui, icon: &str, label: &str, color: egui::Color32, fill: egui::Color32, outline: Stroke) -> egui::Response {
    let band = Tokens::get(ui.ctx()).band();
    let (rect, response) = ui.allocate_exact_size(vec2(28.0, 24.0), Sense::click());
    let fill = if response.hovered() { if fill == egui::Color32::TRANSPARENT { band.hover } else { fill.gamma_multiply(0.85) } } else { fill };
    ui.painter().rect(rect, 4.0, fill, outline, egui::StrokeKind::Inside);
    icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(15.0, 15.0)), icon, color, color);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    response
}

/// Whether the caret is inside a table (so the contextual tabs and their badges are shown).
pub fn in_table_public(app: &WordApp) -> bool {
    app.session.sel.focus.path.cell().is_some()
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    if app.session.view.read_mode {
        reading_toolbar(app, ui, t);
        return;
    }
    // Editing an equation brings up the Equation tab; leaving it goes back.
    if app.session.math.is_some() {
        if app.equation_prev_tab.is_none() {
            app.equation_prev_tab = Some(app.ui.tab.clone());
            app.ui.tab = "Equation".into();
        }
    } else if let Some(prev) = app.equation_prev_tab.take()
        && app.ui.tab == "Equation"
    {
        app.ui.tab = prev;
    }
    // Going into a header or footer brings up the Header & Footer tab; leaving it goes back.
    if app.session.in_header_footer() {
        if app.hf_prev_tab.is_none() {
            app.hf_prev_tab = Some(app.ui.tab.clone());
            app.ui.tab = "Header & Footer".into();
        }
    } else if let Some(prev) = app.hf_prev_tab.take()
        && app.ui.tab == "Header & Footer"
    {
        app.ui.tab = prev;
    }
    // Outline view brings up the Outlining tab; closing it goes back.
    if app.session.view.mode == wordcraft_layout::ViewMode::Outline {
        if app.outline_prev_tab.is_none() {
            app.outline_prev_tab = Some(app.ui.tab.clone());
            app.ui.tab = "Outlining".into();
        }
    } else if let Some(prev) = app.outline_prev_tab.take()
        && app.ui.tab == "Outlining"
    {
        app.ui.tab = prev;
    }
    // The tabs shown: File, the user's tabs in their order (Customize Ribbon, #379), then the
    // contextual ones (the Outlining tab right after File). A contextual tab (Table, Picture
    // Format) may be stored while the selection moved away, and a tab may have been hidden.
    let mut user_tabs = app.session.ribbon.tabs().into_owned();
    user_tabs.retain(|t| !cfg!(target_arch = "wasm32") || t.name != "Zotero");
    let mut tabs: Vec<(String, bool)> = vec![("File".to_string(), false)];
    tabs.extend(user_tabs.iter().filter(|e| !e.hidden).map(|e| (e.name.clone(), e.custom)));
    for ct in contextual_tabs(&app.session) {
        if ct == "Outlining" {
            tabs.insert(1.min(tabs.len()), (ct.to_string(), false));
        } else if !tabs.iter().any(|(n, _)| n == ct) {
            tabs.push((ct.to_string(), false));
        }
    }
    {
        let names: Vec<&str> = tabs.iter().map(|(n, _)| n.as_str()).collect();
        let next = resolve_tab(&app.ui.tab, &names);
        let next = if names.contains(&next) { next } else { names.iter().copied().find(|n| *n != "File").unwrap_or(next) };
        if next != app.ui.tab {
            app.ui.tab = next.to_string();
        }
    }
    // Tab strip.
    egui::Panel::top("tabs")
        .exact_size(30.0)
        // No line under the strip: the active tab runs on into the ribbon.
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.tab_strip).inner_margin(egui::Margin { left: 8, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            // The navy band: white tab names; the active tab is a white tab joined to the ribbon
            // below it, its name in navy.
            crate::theme::band_ui(ui);
            let band = t.band();
            // The band's motif, carried on from the title bar.
            let (_, motif) = crate::theme::band(ui.ctx());
            let strip = ui.clip_rect();
            let motif_band = crate::band_motifs::band_rect(ui.ctx()).unwrap_or(strip);
            crate::band_motifs::paint(ui.painter(), motif_band, motif, t.dark);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Share, Editing mode, Comments, drawn first so the tabs get only the room
                    // left of them (#897). With no document open (#1129) there is nothing to
                    // share, edit or comment on: only they are greyed out, not the tabs drawn
                    // after them, so File still opens New and Open.
                    ui.scope(|ui| {
                        if !app.has_document() {
                            ui.disable();
                        }
                        let resp = band_action(ui, "share", tl!("Share"), band.on_accent, band.accent, Stroke::NONE);
                        if resp.on_hover_text(tl!("Export a copy to share (PDF, Word document)")).clicked() {
                            let _ = app.run("ui.backstage", json!({"value": true, "page": "export"}));
                        }
                        ui.add_space(6.0);
                    });
                    // Report a problem, left of Share: gold outline, and it works with no document open.
                    if crate::report::report_url("", "").is_some() {
                        let resp = band_action(
                            ui,
                            "report",
                            tl!("Report a problem"),
                            band.accent,
                            egui::Color32::TRANSPARENT,
                            Stroke::new(1.0, band.accent),
                        );
                        if resp.on_hover_text(tl!("Tell the developer about a problem (opens a form)")).clicked() {
                            let _ = app.run("ui.reportProblem", json!({}));
                        }
                        ui.add_space(6.0);
                    }
                    ui.scope(|ui| {
                        if !app.has_document() {
                            ui.disable();
                        }
                        let track = app.session.doc.settings.track_changes;
                        let view_only = app.session.view_only;
                        let (mode_icon, mode_label) = if view_only {
                            ("eye", tl!("View only"))
                        } else if track {
                            ("trackChanges", tl!("Reviewing"))
                        } else {
                            ("pencil", tl!("Editing"))
                        };
                        let mode =
                            band_action(ui, mode_icon, mode_label, band.text, egui::Color32::TRANSPARENT, Stroke::NONE).on_hover_text(mode_label);
                        egui::Popup::menu(&mode).show(|ui| {
                            if menu_icon_button(ui, "pencil", tl!("Editing — edit the document directly"), "", true, !view_only && !track).clicked()
                            {
                                // Leave view-only first: it refuses the tracking command.
                                let _ = app.run("review.viewOnly", json!({"value": false}));
                                let _ = app.run("review.trackChanges", json!({"value": false}));
                                ui.close();
                            }
                            if menu_icon_button(ui, "trackChanges", tl!("Reviewing — edits become suggestions"), "", true, !view_only && track)
                                .clicked()
                            {
                                let _ = app.run("review.viewOnly", json!({"value": false}));
                                let _ = app.run("review.trackChanges", json!({"value": true}));
                                ui.close();
                            }
                            if menu_icon_button(ui, "eye", tl!("View only — no changes or suggestions"), "", true, view_only).clicked() {
                                let _ = app.run("review.viewOnly", json!({"value": true}));
                                ui.close();
                            }
                        });
                        ui.add_space(4.0);
                        if band_action(ui, "comment", tl!("Comments"), band.text, egui::Color32::TRANSPARENT, Stroke::NONE)
                            .on_hover_text(tl!("Comments"))
                            .clicked()
                        {
                            let _ = app.run("view.commentsPane", json!({}));
                        }
                    });
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        // The tabs that don't fit go into a » menu at the end of the strip. File, the
                        // active tab and the contextual tabs (appended last) stay on the strip (#897).
                        let labels: Vec<&str> = tabs.iter().map(|(tab, custom)| if *custom { tab.as_str() } else { tl!(tab) }).collect();
                        let gap = ui.spacing().item_spacing.x;
                        let show_numbers = app.ui.show_ribbon_tab_numbers;
                        let widths: Vec<(f32, bool)> = tabs
                            .iter()
                            .zip(&labels)
                            .map(|((tab, custom), shown)| {
                                let font = if *tab == app.ui.tab { semibold(12.5) } else { medium(12.5) };
                                let label_width = ui.ctx().fonts_mut(|f| f.layout_no_wrap(shown.to_string(), font, t.text).size().x);
                                let number_width = if show_numbers && tab != "File" { TAB_NUMBER_SPACE } else { 0.0 };
                                let contextual = !custom && wordcraft_engine::ribbon::CONTEXTUAL_TABS.contains(&tab.as_str());
                                (label_width + 18.0 + number_width + gap, tab == "File" || *tab == app.ui.tab || contextual)
                            })
                            .collect();
                        let available = ui.available_width();
                        let mut fits = fit_tabs(&widths, available, MORE_TABS_W + gap);
                        let occupied = |fits: &[bool]| {
                            let visible: f32 = widths.iter().zip(fits).filter(|(_, fit)| **fit).map(|((w, _), _)| w).sum();
                            visible + if fits.contains(&false) { MORE_TABS_W + gap } else { 0.0 }
                        };
                        if occupied(&fits) > available {
                            // At narrow widths keep File and the selected tab. Contextual tabs can
                            // still be reached through the overflow menu.
                            let essential: Vec<_> =
                                widths.iter().zip(&tabs).map(|((w, _), (tab, _))| (*w, tab == "File" || *tab == app.ui.tab)).collect();
                            fits = fit_tabs(&essential, available, MORE_TABS_W + gap);
                        }
                        let file_width = widths.first().map_or(0.0, |(width, _)| *width);
                        let tab_width = (occupied(&fits) - file_width).max(0.0);
                        let first_tab_x = ui.cursor().min.x + file_width;
                        let desired_start = strip.center().x - tab_width / 2.0;
                        // Centred by default; off (File › Options › General) the tabs start right after File.
                        let offset = if app.ui.ribbon_tabs_centered {
                            (desired_start - first_tab_x).clamp(0.0, (available - file_width - tab_width).max(0.0))
                        } else {
                            0.0
                        };

                        for (index, ((((tab, custom), shown), (width, _)), fits)) in tabs.iter().zip(&labels).zip(&widths).zip(&fits).enumerate() {
                            if !fits {
                                continue;
                            }
                            let (tab, shown) = (tab.as_str(), *shown);
                            let contextual = !custom && wordcraft_engine::ribbon::CONTEXTUAL_TABS.contains(&tab);
                            let (r, resp) = ui.allocate_exact_size(vec2(width - gap, 30.0), Sense::click());
                            if app.ui.keytips == crate::keytips::Phase::Tabs {
                                crate::keytips::record_tab(r, tab, &app.ui.keytip_prefix, &mut app.keytip_rects);
                            }
                            let active = app.ui.tab == tab && !app.ui.backstage;
                            resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, shown));
                            if active {
                                // A tab of the ribbon's colour, from just below the strip's top
                                // down into the ribbon, so the two read as one.
                                let tab_r = Rect::from_min_max(pos2(r.min.x, r.min.y + 3.0), pos2(r.max.x, r.max.y + 1.0));
                                // Rounded top corners, and bottom corners that curve out into the
                                // ribbon with the same radius: a ribbon-coloured square beside each,
                                // with a band-coloured circle cut out of its upper outer corner.
                                const R: f32 = 8.0;
                                ui.painter().rect_filled(tab_r, egui::CornerRadius { nw: R as u8, ne: R as u8, sw: 0, se: 0 }, t.ribbon);
                                // The strip is cut off where the ribbon starts: curve up from there,
                                // or the ribbon hides the curve's lower part and it looks tighter.
                                let bottom = tab_r.max.y.min(ui.clip_rect().max.y);
                                // Each is a fan of triangles from the tab's foot to the quarter
                                // circle, so the band (and its motif) shows above the curve.
                                for (x, outer) in [(tab_r.min.x, tab_r.min.x - R), (tab_r.max.x, tab_r.max.x + R)] {
                                    let centre = pos2(outer, bottom - R);
                                    let mut mesh = egui::epaint::Mesh::default();
                                    mesh.colored_vertex(pos2(x, bottom), t.ribbon);
                                    const STEPS: u32 = 8;
                                    for i in 0..=STEPS {
                                        let a = std::f32::consts::FRAC_PI_2 * i as f32 / STEPS as f32;
                                        // From straight below the centre round to level with it, on the tab's side.
                                        let p = centre + vec2((x - outer).signum() * R * a.sin(), R * a.cos());
                                        mesh.colored_vertex(p, t.ribbon);
                                        if i > 0 {
                                            mesh.add_triangle(0, i, i + 1);
                                        }
                                    }
                                    ui.painter().add(mesh);
                                }
                            } else if resp.hovered() {
                                ui.painter().rect_filled(r.shrink2(vec2(0.0, 4.0)), 4.0, band.hover);
                            }
                            // The active tab's name in the app's navy (light blue in the dark theme); the
                            // others white; contextual tabs (Table Design…) in the icon's gold.
                            let color = if active {
                                if t.dark { t.accent_text } else { t.band_ink }
                            } else if contextual {
                                band.accent_text
                            } else {
                                band.text
                            };
                            let mut label_center = r.center();
                            if show_numbers && index > 0 {
                                let badge = pos2(r.left() + 17.0, r.center().y);
                                ui.painter().circle_filled(badge, 8.0, egui::Color32::WHITE);
                                if active {
                                    ui.painter().circle_stroke(badge, 8.0, Stroke::new(0.75, t.border_strong));
                                }
                                ui.painter().text(badge, Align2::CENTER_CENTER, index.to_string(), semibold(9.0), t.band_ink);
                                label_center.x += TAB_NUMBER_SPACE / 2.0;
                            }
                            ui.painter().text(label_center, Align2::CENTER_CENTER, shown, if active { semibold(12.5) } else { medium(12.5) }, color);
                            if resp.clicked() {
                                let _ = app.run("ui.tab", json!({"tab": tab}));
                            }
                            if resp.double_clicked() && tab != "File" {
                                let _ = app.run("ui.collapseRibbon", json!({}));
                            }
                            resp.context_menu(|ui| crate::widgets::customize_menu_items(ui, app));
                            if index == 0 && offset > 0.0 {
                                ui.add_space(offset);
                            }
                        }
                        let left_out: Vec<(&str, &str)> = tabs
                            .iter()
                            .zip(&labels)
                            .zip(&fits)
                            .filter(|(_, fits)| !**fits)
                            .map(|(((tab, _), shown), _)| (tab.as_str(), *shown))
                            .collect();
                        if !left_out.is_empty() {
                            let (r, resp) = ui.allocate_exact_size(vec2(MORE_TABS_W, 30.0), Sense::click());
                            if resp.hovered() {
                                ui.painter().rect_filled(r.shrink2(vec2(0.0, 4.0)), 4.0, t.hover);
                            }
                            ui.painter().text(r.center(), Align2::CENTER_CENTER, "»", medium(12.5), band.text);
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("More tabs")));
                            egui::Popup::menu(&resp).show(|ui| {
                                for (tab, shown) in left_out {
                                    if ui.steady_label(false, shown).clicked() {
                                        let _ = app.run("ui.tab", json!({"tab": tab}));
                                        ui.close();
                                    }
                                }
                            });
                        }
                    });
                });
            });
        });
    if app.ui.ribbon_collapsed {
        return;
    }
    egui::Panel::top("ribbon")
        .exact_size(CONTENT_H + crate::widgets::label_strip_h(app) + 10.0)
        .frame(
            // No outline: its top edge drew a grey line between the active tab and the ribbon. The
            // panel's own divider still separates the ribbon from the ruler below.
            egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left: 8, right: 8, top: 4, bottom: 4 }),
        )
        .show(ui, |ui| {
            // Right-click on the ribbon (outside a button): Customize Ribbon. Registered first, so
            // the buttons drawn over it keep their own clicks.
            // No document open (#1129): every command is greyed out but Help's.
            if !app.has_document() && app.ui.tab != "Help" {
                ui.disable();
            }
            let bg = ui.interact(ui.max_rect(), ui.id().with("ribbon_background"), Sense::click());
            bg.context_menu(|ui| crate::widgets::customize_menu_items(ui, app));
            let entry = user_tabs.iter().find(|e| e.name == app.ui.tab);
            let scroll = egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                let ribbon_row = egui::UiBuilder::new()
                    .layout(egui::Layout::left_to_right(egui::Align::Center).with_cross_align(egui::Align::Min))
                    .ui_stack_info(egui::UiStackInfo::default().with_tag(crate::widgets::RIBBON_CONTROL_TAG));
                ui.scope_builder(ribbon_row, |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                    match entry {
                        // Every tab hidden: nothing to show.
                        Some(e) if e.hidden => return,
                        Some(e) if e.custom => return custom_groups(app, ui, &e.groups),
                        _ => {}
                    }
                    match app.ui.tab.as_str() {
                        "Home" => home(app, ui),
                        "Insert" => insert(app, ui),
                        "Draw" => draw(app, ui),
                        "Design" => design(app, ui),
                        "Layout" => layout(app, ui),
                        "References" => references(app, ui),
                        "Mailings" => mailings(app, ui),
                        "Review" => review(app, ui),
                        "View" => view(app, ui),
                        "Developer" => developer(app, ui),
                        "Zotero" => zotero(app, ui),
                        "Help" => help(app, ui),
                        "Table Design" => table_design(app, ui),
                        "Table Layout" => table_layout(app, ui),
                        "Equation" if app.session.math.is_some() => crate::equation_tab::show(app, ui),
                        "Picture Format" => picture_format(app, ui),
                        "Shape Format" => shape_format(app, ui),
                        "SmartArt Design" => smart_art_design(app, ui),
                        "Outlining" => outlining(app, ui),
                        "Chart Design" => chart_design(app, ui),
                        "Header & Footer" => header_footer(app, ui),
                        _ => home(app, ui),
                    }
                    if let Some(e) = entry {
                        custom_groups(app, ui, &e.groups);
                    }
                });
            });
            // The bar is hidden, so a row wider than the window would be clipped with no sign. Edge chevrons show
            // where more is and scroll it on click. Every command also stays reachable through Search Commands.
            let r = scroll.inner_rect;
            let overflow = scroll.content_size.x - r.width();
            let offset = scroll.state.offset.x;
            let step = 160.0;
            let mut next = None;
            if offset > 1.0 {
                let left = Rect::from_min_max(r.left_top(), pos2(r.left() + 18.0, r.bottom()));
                if edge_cue(ui, left, "‹", tl!("Scroll ribbon left"), &t) {
                    next = Some(offset - step);
                }
            }
            if overflow - offset > 1.0 {
                let right = Rect::from_min_max(pos2(r.right() - 18.0, r.top()), r.right_bottom());
                if edge_cue(ui, right, "›", tl!("Scroll ribbon right"), &t) {
                    next = Some(offset + step);
                }
            }
            if let Some(x) = next {
                let mut state = scroll.state;
                state.offset.x = x.clamp(0.0, overflow.max(0.0));
                state.store(ui.ctx(), scroll.id);
            }
        });
}

/// Header, Footer and Page Number: on Insert and on the Header & Footer tab.
fn hf_menus(ui: &mut Ui, app: &mut WordApp) {
    menu_button(ui, app, "header", Some("Header"), "Header", true, "insert.header", |ui, app| {
        mi(ui, app, "Blank", "insert.header", json!({"preset": "blank"}));
        mi(ui, app, "Blank (Three Columns)", "insert.header", json!({"preset": "blankThree"}));
        mi(ui, app, "Document Title", "insert.header", json!({"preset": "title"}));
        ui.separator();
        mi(ui, app, "Edit Header", "insert.editHeader", json!({}));
        mi(ui, app, "Remove Header", "insert.removeHeader", json!({}));
    });
    menu_button(ui, app, "footer", Some("Footer"), "Footer", true, "insert.footer", |ui, app| {
        mi(ui, app, "Blank", "insert.footer", json!({"preset": "blank"}));
        mi(ui, app, "Blank (Three Columns)", "insert.footer", json!({"preset": "blankThree"}));
        mi(ui, app, "Page Number", "insert.footer", json!({"preset": "pageNumber"}));
        ui.separator();
        mi(ui, app, "Edit Footer", "insert.editFooter", json!({}));
        mi(ui, app, "Remove Footer", "insert.removeFooter", json!({}));
    });
    menu_button(ui, app, "pageNumber", Some("Page\nNumber"), "Page Number", true, "insert.pageNumber", |ui, app| {
        pic_item(ui, app, "Top of Page", None, "pn_top", "insert.pageNumber", json!({"position": "top", "align": "right"}));
        pic_item(ui, app, "Bottom of Page", None, "pn_bottom", "insert.pageNumber", json!({"position": "bottom", "align": "center"}));
        pic_item(ui, app, "Page X of Y", None, "pn_xofy", "insert.pageNumber", json!({"position": "bottom", "format": "x of y"}));
        pic_item(ui, app, "Current Position", None, "pn_current", "insert.pageNumber", json!({"position": "current"}));
        ui.separator();
        pic_item(ui, app, "Format Page Numbers…", None, "pn_format", "ui.dialog", json!({"name": "pageNumberFormat"}));
        pic_item(ui, app, "Page Number Position…", None, "pn_position", "ui.dialog", json!({"name": "pageNumberPosition"}));
    });
}

/// The Header & Footer tab, shown while a header or footer is edited: insert into it, move
/// between sections, Link to Previous, the section's options and position, and Close.
fn header_footer(app: &mut WordApp, ui: &mut Ui) {
    let st = wordcraft_engine::cmd::hf::state(&mut app.session);
    let page = wordcraft_engine::cmd::hf::context(&mut app.session).map_or(0, |c| c.page);
    group(ui, "Header & Footer", app, hf_menus);
    group(ui, "Insert", app, |ui, app| {
        big(ui, app, "dateTime", "Date &\nTime", "insert.dateTime", json!({}), false);
        big(ui, app, "hfField", "Field", "ui.dialog", json!({"name": "field"}), false);
        big(ui, app, "hfPictureFile", "Picture\nfrom File", "insert.picture", json!({}), false);
    });
    group(ui, "Navigation", app, |ui, app| {
        ui.add_enabled_ui(!st.header, |ui| {
            big(ui, app, "hfGoHeader", "Go to\nHeader", "insert.editHeader", json!({"page": page}), false);
        });
        ui.add_enabled_ui(st.header, |ui| {
            big(ui, app, "hfGoFooter", "Go to\nFooter", "insert.editFooter", json!({"page": page}), false);
        });
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                ui.add_enabled_ui(st.section > 0, |ui| {
                    small(ui, app, "hfPrevious", None, "Previous Section", "hf.previous", json!({}), false);
                });
                ui.add_enabled_ui(st.section + 1 < st.sections, |ui| {
                    small(ui, app, "hfNext", None, "Next Section", "hf.next", json!({}), false);
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            ui.add_enabled_ui(st.can_link, |ui| {
                small(ui, app, "hfLink", Some("Link to Previous"), "Link to Previous", "layout.linkToPrevious", json!({}), st.linked)
                    .on_hover_text(tl!("Use the previous section's header or footer; turn off to give this section its own"));
            });
        });
    });
    group(ui, "Options", app, |ui, app| {
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                let mut first = st.different_first;
                if ui.checkbox(&mut first, tl!("Different First Page")).changed() {
                    let _ = app.run("layout.differentFirstPage", json!({"value": first}));
                }
                ui.add_space(8.0);
                let mut odd = st.different_odd_even;
                if ui.checkbox(&mut odd, tl!("Different Odd & Even Pages")).changed() {
                    let _ = app.run("layout.differentOddEven", json!({"value": odd}));
                }
            });
            ui.add_space(crate::widgets::ROW_GAP + 8.0);
            let mut text = st.show_document_text;
            if ui.checkbox(&mut text, tl!("Show Document Text")).changed() {
                let _ = app.run("hf.showDocumentText", json!({"value": text}));
            }
        });
    });
    group(ui, "Position", app, |ui, app| {
        let unit = crate::options::unit();
        let u = unit.pt_per_unit();
        stack(ui, |ui| {
            for (icon, label, key, pt) in [
                ("hfFromTop", "Header from Top:", "header", st.header_from_top),
                ("hfFromBottom", "Footer from Bottom:", "footer", st.footer_from_bottom),
            ] {
                crate::widgets::row(ui, |ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                    let t = Tokens::get(ui.ctx());
                    icons::paint(ui.painter(), r.shrink(2.0), icon, t.icon, t.accent);
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(tl!(label)).size(11.5));
                    ui.add_space(4.0);
                    let mut v = pt / u;
                    let field = crate::numfield::NumField::new(&mut v).speed(0.05).range(0.0..=300.0 / u).measure(unit).max_decimals(2);
                    let r = ui.add_sized(vec2(78.0, 20.0), field);
                    if r.changed() {
                        if r.dragged() && !r.drag_started() {
                            app.session.join_next_undo();
                        }
                        let _ = app.run("hf.position", json!({key: (v * u).clamp(0.0, 300.0)}));
                    }
                });
            }
        });
    });
    group(ui, "Close", app, |ui, app| {
        big(ui, app, "hfClose", "Close Header\nand Footer", "insert.closeHeader", json!({}), false);
    });
}

/// Custom groups (Customize Ribbon, #379): up to three commands as large buttons, more as small
/// labelled buttons in columns of three. Each shows the command's own icon and label.
fn custom_groups(app: &mut WordApp, ui: &mut Ui, groups: &[wordcraft_engine::ribbon::CustomGroup]) {
    for g in groups {
        let ids: Vec<(&str, String)> =
            g.commands.iter().filter_map(|id| app.session.registry.get(id).map(|s| (s.id, tl!(s.label).to_string()))).collect();
        crate::widgets::group_titled(ui, &g.name, &g.name, app, |ui, app| {
            if ids.len() <= 3 {
                for (id, label) in &ids {
                    big(ui, app, crate::widgets::command_icon(id), &two_lines(label), id, json!({}), false);
                }
                return;
            }
            for column in ids.chunks(3) {
                stack(ui, |ui| {
                    for (id, label) in column {
                        small(ui, app, crate::widgets::command_icon(id), Some(label), label, id, json!({}), false);
                    }
                });
            }
        });
    }
}

/// A large button's label on two lines when it is long (broken at the space nearest the middle).
fn two_lines(label: &str) -> String {
    let n = label.chars().count();
    if n <= 10 {
        return label.to_string();
    }
    let mid = n / 2;
    let best = label.char_indices().filter(|(_, c)| *c == ' ').min_by_key(|(i, _)| label[..*i].chars().count().abs_diff(mid)).map(|(i, _)| i);
    match best {
        Some(i) => format!("{}\n{}", &label[..i], label[i..].trim_start()),
        None => label.to_string(),
    }
}

/// A chevron over a ribbon edge that is also a button; `true` on the frame it is clicked.
fn edge_cue(ui: &mut Ui, rect: Rect, glyph: &str, hint: &str, t: &Tokens) -> bool {
    let resp = ui.interact(rect, ui.id().with(glyph), Sense::click()).on_hover_text(hint);
    let ink = if resp.hovered() { t.text } else { t.text_dim };
    ui.painter().rect_filled(rect, 0.0, t.ribbon);
    crate::icons::paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), vec2(18.0, 18.0)),
        if glyph.contains('‹') || glyph.contains('❮') { "chevron-left" } else { "chevron-right" },
        ink,
        ink,
    );
    resp.clicked()
}

/// A column of rows or small buttons, centred in the group's height as in Word (two rows sit in
/// the middle, three fill it). The centring uses the column's height from the last frame; a frame
/// whose height changed is redrawn before it is shown.
pub(crate) fn stack(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let id = ui.next_auto_id().with("stack_height");
    // Only a column standing in a group's full height is centred, not one nested in a row.
    let room = ui.available_height();
    let centre = ui.layout().is_horizontal() && room >= CONTENT_H - 0.5;
    let last: Option<f32> = ui.data(|d| d.get_temp(id));
    let pad = if centre { last.map_or(0.0, |h| ((CONTENT_H - h) / 2.0).max(0.0).floor()) } else { 0.0 };
    let inner = ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
        ui.add_space(pad);
        add(ui);
    });
    let h = inner.response.rect.height() - pad;
    if centre && last.is_none_or(|l| (l - h).abs() > 0.5) {
        ui.data_mut(|d| d.insert_temp(id, h));
        ui.ctx().request_discard("ribbon column height");
    }
}

/// Layout › Size: the named paper sizes, then More Paper Sizes… for a custom size in Page
/// Setup (#1433).
pub(crate) fn size_menu(ui: &mut Ui, app: &mut WordApp) {
    // The current paper size is checked.
    let x = wordcraft_engine::cmd::page::sect(&app.session);
    let current = wordcraft_geom::paper_name(x.page_w, x.page_h);
    for (n, w, h) in wordcraft_geom::PAPER_SIZES {
        // Paper names are the industry's (Letter, A4, B5 (JIS)) and stay as they are; the
        // envelopes say what they are. Sizes in the measurement unit, written the region's
        // way (`21 cm × 29,7 cm`, #838).
        let name = if n.starts_with("Envelope ") { tl!(n) } else { n };
        let (width, height) = (crate::options::length(*w), crate::options::length(*h));
        let label = crate::i18n::fmt(tl!("{paper}   {width} × {height}"), &[("paper", name), ("width", &width), ("height", &height)]);
        mi_check(ui, app, &label, current == Some(*n), "layout.size", json!({"name": n}));
    }
    ui.separator();
    mi(ui, app, crate::dialogs::PAPER_LABELS.more, "ui.dialog", json!({"name": "pageSetup"}));
}

pub(crate) fn mi(ui: &mut Ui, app: &mut WordApp, label: &str, id: &str, params: Value) {
    mi_text(ui, app, tl!(label), id, params);
}

/// Home › Multilevel List › My Lists: the lists saved by name (Define New Multilevel List), each
/// with how it numbers; one picked numbers the selected paragraphs with it. Remove from My Lists
/// takes one out of the settings (lists already in documents stay).
fn saved_list_items(ui: &mut Ui, app: &mut WordApp) {
    if app.ui.saved_lists.is_empty() {
        return;
    }
    ui.separator();
    ui.label(egui::RichText::new(tl!("My Lists")).small().weak());
    let lists = app.ui.saved_lists.clone();
    for l in &lists {
        let label = format!("{}   {}", l.name, l.sample());
        // The name is the user's: shown as it is, not looked up for a translation.
        let enabled = crate::widgets::enabled(app, "list.define");
        if ui.add_enabled(enabled, egui::Button::new(label).min_size(vec2(200.0, 0.0))).clicked() {
            crate::widgets::run_from_ribbon(app, "list.define", l.params());
            ui.close();
        }
    }
    ui.menu_button(tl!("Remove from My Lists"), |ui| {
        for l in &lists {
            if ui.button(&l.name).clicked() {
                app.ui.saved_lists.retain(|x| x.name != l.name);
                ui.close();
            }
        }
    });
}

/// Home › Borders and Table Design › Borders, as in Word: the sides, the presets, the inside
/// and diagonal rules, Horizontal Line, then Draw Table, View Gridlines and the dialog. In a table
/// they set the table's borders (diagonals: the selected cells'); elsewhere the paragraph's, and
/// the table-only choices are off.
fn border_items(ui: &mut Ui, app: &mut WordApp) {
    let in_table = crate::widgets::enabled(app, "table.borders");
    let side = |k: &str| -> (&'static str, Value) {
        if in_table {
            ("table.borders", json!({"kind": k}))
        } else {
            // A paragraph's inside border is the one between its lines (`insideH`).
            ("para.borders", json!({"kind": if k == "insideH" { "inside" } else { k }}))
        }
    };
    let groups: [&[(&str, &str)]; 3] = [
        &[("Bottom Border", "bottom"), ("Top Border", "top"), ("Left Border", "left"), ("Right Border", "right")],
        &[("No Border", "none"), ("All Borders", "all"), ("Outside Borders", "outside"), ("Inside Borders", "inside")],
        &[
            ("Inside Horizontal Border", "insideH"),
            ("Inside Vertical Border", "insideV"),
            ("Diagonal Down Border", "diagDown"),
            ("Diagonal Up Border", "diagUp"),
        ],
    ];
    for group in groups {
        for (l, k) in group {
            let (id, params) = if matches!(*k, "insideV" | "diagDown" | "diagUp") { ("table.borders", json!({"kind": k})) } else { side(k) };
            pic_item(ui, app, l, None, &format!("tb_{k}"), id, params);
        }
        ui.separator();
    }
    pic_item(ui, app, "Horizontal Line", None, "hline", "insert.horizontalLine", json!({}));
    ui.separator();
    let gridlines = app.session.view.table_gridlines;
    pic_item(ui, app, "Draw Table", None, "drawTable", "table.draw", json!({}));
    pic_item(ui, app, "View Gridlines", Some(gridlines), "gridlines", "table.viewGridlines", json!({}));
    pic_item(ui, app, "Borders and Shading…", None, "borders", "para.borders", json!({}));
}

/// A menu item with a small picture before its name (Layout's Margins, Text Direction, Orientation,
/// Columns, Breaks and Line Numbers; Table Design's Borders; Table Layout's Select, Delete and
/// AutoFit), as in Word. `checked` (`Some`) highlights the current setting.
fn pic_item(ui: &mut Ui, app: &mut WordApp, label: &str, checked: Option<bool>, pic: &str, id: &str, params: Value) {
    let enabled = crate::widgets::enabled(app, id);
    let atom = ui.id().with(("layout picture", pic));
    let on = checked.unwrap_or(false);
    let resp = ui
        .add_enabled_ui(enabled, |ui| {
            egui::Button::new((egui::Atom::custom(atom, vec2(24.0, 20.0)), tl!(label))).selected(on).min_size(vec2(220.0, 28.0)).atom_ui(ui)
        })
        .inner;
    if let Some(r) = resp.rect(atom) {
        let t = Tokens::get(ui.ctx());
        let (ink, accent) = if enabled { (t.icon, t.accent) } else { (t.text_disabled, t.text_disabled) };
        layout_picture(ui.painter(), r, pic, ink, accent);
    }
    resp.response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, on, tl!(label)));
    if resp.response.clicked() {
        crate::widgets::run_from_ribbon(app, id, params);
        ui.close();
    }
}

/// The small pictures of [`pic_item`], drawn on a 24 × 20 grid: pages, text lines, columns,
/// break marks and line numbers in `ink`, what the option changes in `accent`.
fn layout_picture(p: &egui::Painter, r: Rect, name: &str, ink: egui::Color32, accent: egui::Color32) {
    if !icons::home_menu::paint(p, r, name, ink) && !icons::layout_menu::paint(p, r, name, ink) && !icons::table_menu::paint(p, r, name, ink) {
        krub_icons::menu::paint(p, r, name, ink, accent);
    }
}

/// Insert › Shapes: a menu item with a small picture of the shape before its name.
fn shape_item(ui: &mut Ui, app: &mut WordApp, label: &str, kind: &str, arrow: &str, id: &str, params: Value) {
    let enabled = crate::widgets::enabled(app, id);
    let pic = ui.id().with(("shape preview", kind, arrow));
    let resp = ui
        .add_enabled_ui(enabled, |ui| {
            egui::Button::new((egui::Atom::custom(pic, vec2(24.0, 18.0)), tl!(label))).min_size(vec2(220.0, 26.0)).atom_ui(ui)
        })
        .inner;
    if let Some(r) = resp.rect(pic) {
        let t = Tokens::get(ui.ctx());
        let ink = if enabled { t.icon } else { t.text_disabled };
        shape_preview(ui.painter(), r, kind, arrow, ink, t.accent.gamma_multiply(if enabled { 0.25 } else { 0.0 }));
    }
    if resp.response.clicked() {
        crate::widgets::run_from_ribbon(app, id, params);
        ui.close();
    }
}

/// A shape drawn small in `r`: outline in `ink`, filled with `fill` (connectors are lines; `arrow`
/// "end" puts a head on their end).
fn shape_preview(p: &egui::Painter, r: Rect, kind: &str, arrow: &str, ink: egui::Color32, fill: egui::Color32) {
    use std::f32::consts::{PI, TAU};
    let stroke = Stroke::new(1.3, ink);
    let r = r.shrink(1.5);
    let (c, w, h) = (r.center(), r.width(), r.height());
    // A polygon filled as a fan from `centre` (the star, the arrow and the heart are not convex,
    // but every edge is visible from their centre), then outlined.
    let poly = |pts: Vec<egui::Pos2>, centre: egui::Pos2| {
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(centre, fill);
        for q in &pts {
            mesh.colored_vertex(*q, fill);
        }
        let n = pts.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        p.add(egui::Shape::mesh(mesh));
        p.add(egui::Shape::closed_line(pts, stroke));
    };
    let head = |tip: egui::Pos2, from: egui::Pos2| {
        let d = (tip - from).normalized() * 5.0;
        let side = vec2(-d.y, d.x) * 0.6;
        p.add(egui::Shape::convex_polygon(vec![tip, tip - d + side, tip - d - side], ink, Stroke::NONE));
    };
    match kind {
        "rectangle" => {
            p.rect(r, 0.0, fill, stroke, egui::StrokeKind::Middle);
        }
        "roundedRectangle" => {
            p.rect(r, 4.0, fill, stroke, egui::StrokeKind::Middle);
        }
        "ellipse" => {
            p.add(egui::Shape::ellipse_filled(c, r.size() / 2.0, fill));
            p.add(egui::Shape::ellipse_stroke(c, r.size() / 2.0, stroke));
        }
        "triangle" => poly(vec![r.center_top(), r.right_bottom(), r.left_bottom()], c),
        "diamond" => poly(vec![r.center_top(), r.right_center(), r.center_bottom(), r.left_center()], c),
        "line" => {
            p.line_segment([r.left_bottom(), r.right_top()], stroke);
        }
        // A drawing canvas: a dashed frame with a small shape inside.
        "canvas" => {
            let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
            p.extend(egui::Shape::dashed_line(&corners, Stroke::new(1.0, ink), 2.5, 2.0));
            p.circle(c, h * 0.22, fill, stroke);
        }
        "arrow" => {
            let pts = [(0.0, -0.2), (0.6, -0.2), (0.6, -0.5), (1.0, 0.0), (0.6, 0.5), (0.6, 0.2), (0.0, 0.2)];
            poly(pts.iter().map(|(x, y)| egui::pos2(r.min.x + x * w, c.y + y * h)).collect(), egui::pos2(r.min.x + 0.6 * w, c.y));
        }
        "star" => {
            let rad = h / 2.0 + 1.0;
            let pts = (0..10)
                .map(|i| {
                    let a = -PI / 2.0 + i as f32 * TAU / 10.0;
                    let k = if i % 2 == 0 { rad } else { rad * 0.42 };
                    c + vec2(a.cos(), a.sin()) * k
                })
                .collect();
            poly(pts, c);
        }
        "heart" => {
            let pts = (0..40)
                .map(|i| {
                    let a = i as f32 * TAU / 40.0;
                    let x = 16.0 * a.sin().powi(3);
                    let y = 13.0 * a.cos() - 5.0 * (2.0 * a).cos() - 2.0 * (3.0 * a).cos() - (4.0 * a).cos();
                    c + vec2(x / 17.0 * h * 0.55, -y / 17.0 * h * 0.55 + 0.5)
                })
                .collect();
            poly(pts, c);
        }
        _ => {
            // Connectors run from the bottom left to the top right, with a dot on the start.
            let (a, b) = (r.left_bottom(), r.right_top());
            let last_from = match kind {
                "elbowConnector" => {
                    let mid = c.x;
                    p.add(egui::Shape::line(vec![a, egui::pos2(mid, a.y), egui::pos2(mid, b.y), b], stroke));
                    egui::pos2(mid, b.y)
                }
                "curvedConnector" => {
                    let pts: Vec<egui::Pos2> = (0..=16)
                        .map(|i| {
                            let t = i as f32 / 16.0;
                            let (c1, c2) = (egui::pos2(c.x, a.y), egui::pos2(c.x, b.y));
                            let u = 1.0 - t;
                            let v =
                                a.to_vec2() * u * u * u + c1.to_vec2() * 3.0 * u * u * t + c2.to_vec2() * 3.0 * u * t * t + b.to_vec2() * t * t * t;
                            v.to_pos2()
                        })
                        .collect();
                    let before = pts[pts.len() - 3];
                    p.add(egui::Shape::line(pts, stroke));
                    before
                }
                _ => {
                    p.line_segment([a, b], stroke);
                    a
                }
            };
            p.circle_filled(a, 1.8, ink);
            if arrow == "end" {
                head(b, last_from);
            }
        }
    }
}

/// A menu button with an original icon, an accessible label, and the existing shortcut text.
fn menu_icon_button(ui: &mut Ui, icon: &str, label: &str, shortcut: &str, enabled: bool, selected: bool) -> egui::Response {
    let atom = ui.id().with(("menu icon", icon, label));
    let result = ui
        .add_enabled_ui(enabled, |ui| {
            egui::Button::new((egui::Atom::custom(atom, vec2(20.0, 18.0)), label))
                .shortcut_text(shortcut)
                .selected(selected)
                .min_size(vec2(200.0, 0.0))
                .atom_ui(ui)
        })
        .inner;
    if let Some(rect) = result.rect(atom) {
        let t = Tokens::get(ui.ctx());
        let (ink, accent) = if enabled { (t.icon, t.accent) } else { (t.text_disabled, t.text_disabled) };
        icons::paint(ui.painter(), rect.shrink(1.0), icon, ink, accent);
    }
    result.response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, label));
    result.response
}

fn mi_icon(ui: &mut Ui, app: &mut WordApp, label: &str, icon: &str, id: &str, params: Value) {
    let shortcut = crate::widgets::shortcut_text(app, id);
    let enabled = crate::widgets::enabled(app, id);
    if menu_icon_button(ui, icon, tl!(label), &shortcut, enabled, false).clicked() {
        crate::widgets::run_from_ribbon(app, id, params);
        ui.close();
    }
}

/// A menu item whose text is already in the interface language.
fn mi_text(ui: &mut Ui, app: &mut WordApp, text: &str, id: &str, params: Value) {
    let sc = crate::widgets::shortcut_text(app, id);
    let enabled = crate::widgets::enabled(app, id);
    let resp = ui.add_enabled(enabled, egui::Button::new(text).shortcut_text(sc).min_size(vec2(200.0, 0.0)));
    if resp.clicked() {
        crate::widgets::run_from_ribbon(app, id, params);
        ui.close();
    }
}

/// The proofing languages the Language menu offers: tag and English name (translated when drawn).
pub(crate) const PROOFING_LANGUAGES: [(&str, &str); 9] = [
    ("en-US", "English (United States)"),
    ("en-GB", "English (United Kingdom)"),
    ("pl-PL", "Polish"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("it-IT", "Italian"),
    ("hr-HR", "Croatian"),
    ("tr-TR", "Turkish"),
    ("ru-RU", "Russian"),
    ("cs-CZ", "Czech"),
];

/// The English name of a proofing language tag, as the status bar shows it (no tag: WordCraft's
/// default, US English): a listed language by its name; another region of one by the language
/// and the region ("French (Canada)"); a bare language subtag by its usual variant; else the tag.
pub(crate) fn language_name(tag: Option<&str>) -> String {
    let tag = tag.unwrap_or("en-US");
    let named =
        |t: &str| PROOFING_LANGUAGES.iter().chain(crate::dialogs_layout::LANGUAGES.iter()).find(|(l, _)| l.eq_ignore_ascii_case(t)).map(|(_, n)| *n);
    // Any language the Language dialog offers (French, German…) by its name, not its tag (#860).
    if let Some(name) = named(tag) {
        return name.to_string();
    }
    let mut parts = tag.split(['-', '_']);
    let primary = parts.next().unwrap_or("");
    // The region: the first two-letter or three-digit subtag after the language (and script).
    let region =
        parts.find(|p| (p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic())) || (p.len() == 3 && p.chars().all(|c| c.is_ascii_digit())));
    match region {
        // A region the list doesn't have: the language's name without its own region or variant.
        Some(r) => {
            let language = crate::dialogs_layout::LANGUAGES
                .iter()
                .find(|(l, _)| l.split('-').next().is_some_and(|p| p.eq_ignore_ascii_case(primary)))
                .map(|(_, n)| n.split(" (").next().unwrap_or(n));
            match language {
                Some(language) => format!("{language} ({})", region_name(r)),
                None => tag.to_string(),
            }
        }
        // A bare language subtag ("pt", "pl"): the variant WordCraft proofs.
        None => match wordcraft_engine::proof::ProofLang::from_tag(Some(tag)) {
            wordcraft_engine::proof::ProofLang::Pl => "Polish".into(),
            wordcraft_engine::proof::ProofLang::Hr => "Croatian".into(),
            wordcraft_engine::proof::ProofLang::Pt => "Portuguese (Brazil)".into(),
            wordcraft_engine::proof::ProofLang::It => "Italian".into(),
            wordcraft_engine::proof::ProofLang::Tr => "Turkish".into(),
            wordcraft_engine::proof::ProofLang::Ru => "Russian".into(),
            wordcraft_engine::proof::ProofLang::Cs => "Czech".into(),
            wordcraft_engine::proof::ProofLang::En if tag.eq_ignore_ascii_case("en") => "English (United States)".into(),
            _ => tag.to_string(),
        },
    }
}

/// The English name of a region subtag the language list has no entry for (else the subtag).
fn region_name(code: &str) -> String {
    const REGIONS: [(&str, &str); 62] = [
        ("AE", "U.A.E."),
        ("AO", "Angola"),
        ("AR", "Argentina"),
        ("AT", "Austria"),
        ("AU", "Australia"),
        ("BA", "Bosnia and Herzegovina"),
        ("BE", "Belgium"),
        ("BO", "Bolivia"),
        ("BR", "Brazil"),
        ("BY", "Belarus"),
        ("CA", "Canada"),
        ("CH", "Switzerland"),
        ("CL", "Chile"),
        ("CN", "China"),
        ("CO", "Colombia"),
        ("CR", "Costa Rica"),
        ("CV", "Cabo Verde"),
        ("CY", "Cyprus"),
        ("DE", "Germany"),
        ("DO", "Dominican Republic"),
        ("EC", "Ecuador"),
        ("EG", "Egypt"),
        ("ES", "Spain"),
        ("FR", "France"),
        ("GB", "United Kingdom"),
        ("GT", "Guatemala"),
        ("HK", "Hong Kong SAR"),
        ("IE", "Ireland"),
        ("IN", "India"),
        ("IQ", "Iraq"),
        ("IT", "Italy"),
        ("JM", "Jamaica"),
        ("JO", "Jordan"),
        ("KW", "Kuwait"),
        ("KZ", "Kazakhstan"),
        ("LB", "Lebanon"),
        ("LI", "Liechtenstein"),
        ("LU", "Luxembourg"),
        ("MA", "Morocco"),
        ("MC", "Monaco"),
        ("ME", "Montenegro"),
        ("MO", "Macao SAR"),
        ("MX", "Mexico"),
        ("MY", "Malaysia"),
        ("MZ", "Mozambique"),
        ("NL", "Netherlands"),
        ("NZ", "New Zealand"),
        ("PE", "Peru"),
        ("PH", "Philippines"),
        ("PR", "Puerto Rico"),
        ("PT", "Portugal"),
        ("PY", "Paraguay"),
        ("QA", "Qatar"),
        ("RU", "Russia"),
        ("SG", "Singapore"),
        ("SY", "Syria"),
        ("TW", "Taiwan"),
        ("UA", "Ukraine"),
        ("US", "United States"),
        ("UY", "Uruguay"),
        ("VE", "Venezuela"),
        ("ZA", "South Africa"),
    ];
    REGIONS.iter().find(|(c, _)| c.eq_ignore_ascii_case(code)).map_or_else(|| code.to_ascii_uppercase(), |(_, n)| (*n).to_string())
}

/// The language at the caret (resolved through the styles), translated, for the status bar.
pub(crate) fn caret_language(app: &WordApp) -> String {
    tl!(&language_name(caret_lang_tag(app).as_deref())).to_string()
}

/// The language tag at the caret, resolved through the styles and the document default.
fn caret_lang_tag(app: &WordApp) -> Option<String> {
    let s = &app.session;
    let style = s.doc.para_at(&s.sel.focus).and_then(|p| p.props.style.clone());
    s.doc.styles.resolve_char(style.as_deref(), &s.typing_props()).lang
}

/// The Hunspell dictionary the caret's language needs but doesn't have (`pl_PL`), if any (#1167).
pub(crate) fn caret_missing_dictionary(app: &WordApp) -> Option<&'static str> {
    wordcraft_engine::proof::missing_dictionary(caret_lang_tag(app).as_deref())
}

/// Review › Language (and the status bar's language): set the selection's proofing language, or
/// leave it unchecked. The current choice has a check mark; a language outside the short list
/// (French, German…) shows checked above it. Set Proofing Language… opens the Language dialog
/// with every language and Set As Default, and Language Preferences… opens Options › Language
/// (#860: the short list alone left no way to pick French).
pub(crate) fn language_menu(ui: &mut Ui, app: &mut WordApp) {
    let st = app.session.run("format.state", &json!({})).unwrap_or_default();
    // The selection's own language; else (nothing set on the text) the one it gets from its
    // style or the document default, as the status bar shows.
    let current = match st.get("lang") {
        Some(Value::String(t)) => t.clone(),
        Some(Value::Null) if app.session.sel.is_collapsed() => caret_lang_tag(app).unwrap_or_else(|| "en-US".into()),
        _ => String::new(),
    };
    let no_proof = st.get("noProof").and_then(Value::as_bool).unwrap_or(false);
    let listed = |tag: &str| {
        current.eq_ignore_ascii_case(tag)
            || (tag == "pl-PL" && current.eq_ignore_ascii_case("pl"))
            || (tag == "pt-BR" && current.eq_ignore_ascii_case("pt"))
            || (tag == "it-IT" && current.eq_ignore_ascii_case("it"))
            || (tag == "hr-HR" && current.eq_ignore_ascii_case("hr"))
            || (tag == "tr-TR" && current.eq_ignore_ascii_case("tr"))
            || (tag == "cs-CZ" && wordcraft_engine::proof::ProofLang::from_tag(Some(&current)) == wordcraft_engine::proof::ProofLang::Cs)
            || (tag == "en-US" && current.eq_ignore_ascii_case("en"))
    };
    let checks = language_checks(&current, listed);
    if let Some(name) = checks.extra {
        mi_check(ui, app, &name, true, "review.language", json!({"lang": current}));
    }
    for ((tag, name), on) in PROOFING_LANGUAGES.iter().zip(checks.listed) {
        mi_check(ui, app, name, on, "review.language", json!({"lang": tag}));
    }
    ui.separator();
    mi_check(ui, app, "Do Not Check Spelling or Grammar", no_proof, "review.language", json!({"noProof": !no_proof}));
    ui.separator();
    mi_check(ui, app, "Set Proofing Language…", false, "review.language", json!({}));
    mi_check(ui, app, "Language Preferences…", false, "file.options", json!({"pane": "language"}));
}

/// The Language menu's check marks for the `current` tag: which short-list entries are checked
/// (`listed` matches a tag), and the name of the current language when it is none of them,
/// shown checked above. A language named like a short-list entry checks that entry, so no name
/// shows twice.
struct LanguageChecks {
    extra: Option<String>,
    listed: [bool; PROOFING_LANGUAGES.len()],
}

fn language_checks(current: &str, listed: impl Fn(&str) -> bool) -> LanguageChecks {
    let name = (!current.is_empty()).then(|| language_name(Some(current)));
    let on = PROOFING_LANGUAGES.map(|(t, n)| listed(t) || name.as_deref() == Some(n));
    LanguageChecks { extra: name.filter(|_| !on.contains(&true)), listed: on }
}

/// A menu item with a check mark when `checked` (the current choice of several).
fn mi_check(ui: &mut Ui, app: &mut WordApp, label: &str, checked: bool, id: &str, params: Value) {
    // An invisible mark keeps the unchecked labels aligned with the checked one.
    let mark = egui::RichText::new("✓");
    let mark = if checked { mark } else { mark.color(egui::Color32::TRANSPARENT) };
    let enabled = crate::widgets::enabled(app, id);
    let resp = ui.add_enabled(enabled, egui::Button::new((mark, tl!(label))).selected(checked).min_size(vec2(200.0, 0.0)));
    if resp.clicked() {
        crate::widgets::run_from_ribbon(app, id, params);
        ui.close();
    }
}

/// Word's text wrapping styles in menu order: label, model value, `arrange.wrap` parameter.
pub(crate) const WRAP_STYLES: [(&str, wordcraft_doc::para::Wrap, &str); 7] = {
    use wordcraft_doc::para::Wrap;
    [
        ("In Line with Text", Wrap::Inline, "inline"),
        ("Square", Wrap::Square, "square"),
        ("Tight", Wrap::Tight, "tight"),
        ("Through", Wrap::Through, "through"),
        ("Top and Bottom", Wrap::TopAndBottom, "topAndBottom"),
        ("Behind Text", Wrap::BehindText, "behindText"),
        ("In Front of Text", Wrap::InFrontOfText, "inFrontOfText"),
    ]
};

/// The Wrap Text menu for the selected picture, shape or text box: the seven styles (the
/// current one checked), then Move with Text / Fix Position on Page for a floating one. The
/// canvas context menu shows it; the ribbon's Wrap Text button can share it.
pub(crate) fn wrap_text_items(ui: &mut Ui, app: &mut WordApp) {
    use wordcraft_engine::cmd::objects::{moves_with_text, selected};
    let state = selected(&app.session).and_then(|(_, o)| o.frame().map(|(_, _, f)| (f.wrap, moves_with_text(f))));
    let wrap = state.map(|(w, _)| w);
    for (label, w, v) in WRAP_STYLES {
        mi_check(ui, app, label, wrap == Some(w), "arrange.wrap", json!({"wrap": v}));
    }
    ui.separator();
    // An inline object is neither: it's part of the text.
    let with_text = state.filter(|(w, _)| *w != wordcraft_doc::para::Wrap::Inline).map(|(_, m)| m);
    mi_check(ui, app, "Move with Text", with_text == Some(true), "arrange.moveWithText", json!({"value": true}));
    mi_check(ui, app, "Fix Position on Page", with_text == Some(false), "arrange.moveWithText", json!({"value": false}));
}

/// A quiet divider between related paragraph controls, sized to one small-button row.
fn paragraph_separator(ui: &mut Ui) {
    let (r, _) = ui.allocate_exact_size(vec2(7.0, crate::widgets::SMALL_H), Sense::hover());
    let color = Tokens::get(ui.ctx()).border_strong;
    ui.painter().line_segment([pos2(r.center().x, r.top() + 4.0), pos2(r.center().x, r.bottom() - 4.0)], Stroke::new(1.0, color));
}

fn home(app: &mut WordApp, ui: &mut Ui) {
    let st = app.session.run("format.state", &json!({})).unwrap_or_default();
    let flag = |k: &str| st.get(k).and_then(Value::as_bool).unwrap_or(false);
    group(ui, "Clipboard", app, |ui, app| {
        menu_button(ui, app, "paste", Some("Paste"), "Paste (⌘V)", true, "edit.paste", |ui, app| {
            mi_icon(ui, app, "Paste", "paste", "edit.paste", json!({}));
            mi_icon(ui, app, "Keep Text Only", "textBox", "edit.pasteText", json!({}));
            mi_icon(ui, app, "Merge Formatting", "painter", "edit.pasteMerge", json!({}));
            mi_icon(ui, app, "Paste Special…", "paste", "edit.pasteSpecial", json!({}));
            // The Clipboard pane (the group had a launcher for it; groups have none now).
            ui.separator();
            mi_icon(ui, app, "Clipboard", "paste", "edit.clipboardPane", json!({}));
        });
        // Two rows: Cut and Copy, then the Format Painter.
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "cut", None, "Cut", "edit.cut", json!({}), false);
                small(ui, app, "copy", None, "Copy", "edit.copy", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                let active = app.session.painter.is_some();
                small(ui, app, "painter", None, "Format Painter", "edit.formatPainter", json!({}), active);
            });
        });
    });
    group(ui, "Font", app, |ui, app| {
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                let font = st.get("font").and_then(Value::as_str).unwrap_or("").to_string();
                let warning = app.font_box_warning(&font);
                let font = app.previews.font_shown(&font);
                let fams = app.previews.families();
                let prev = app.previews.font_preview_fn();
                if let Some(f) = crate::widgets::font_combo_warned(ui, "font", 150.0, &font, &fams, Some(&*prev), warning.as_deref()) {
                    let _ = if f == crate::widgets::ADD_FONT {
                        app.run("font.add", json!({}))
                    } else if f == crate::widgets::LOAD_SYSTEM_FONTS {
                        app.run("font.loadSystem", json!({}))
                    } else {
                        let f = app.previews.font_stored(&f);
                        app.run("format.font", json!({"name": f}))
                    };
                }
                let size = st
                    .get("size")
                    .and_then(Value::as_f64)
                    .map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { format!("{s}") })
                    .unwrap_or_default();
                let sizes: Vec<String> =
                    wordcraft_engine::cmd::format::SIZES.iter().map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { s.to_string() }).collect();
                if let Some(v) = combo(ui, "size", 52.0, &size, &sizes, None)
                    && let Ok(x) = v.trim().parse::<f64>()
                {
                    let _ = app.run("format.size", json!({"size": x}));
                }
                ui.add_space(3.0);
                small(ui, app, "grow", None, "Increase Font Size", "format.growFont", json!({}), false);
                small(ui, app, "shrink", None, "Decrease Font Size", "format.shrinkFont", json!({}), false);
                menu_button(ui, app, "case", None, "Change Case", false, "format.changeCase", |ui, app| {
                    mi(ui, app, "Sentence case.", "format.changeCase", json!({"mode": "sentence"}));
                    mi(ui, app, "lowercase", "format.changeCase", json!({"mode": "lower"}));
                    mi(ui, app, "UPPERCASE", "format.changeCase", json!({"mode": "upper"}));
                    mi(ui, app, "Capitalize Each Word", "format.changeCase", json!({"mode": "title"}));
                    mi(ui, app, "tOGGLE cASE", "format.changeCase", json!({"mode": "toggle"}));
                });
                small(ui, app, "clear", None, "Clear All Formatting", "format.clear", json!({}), false);
                small(ui, app, "phonetic", None, "Phonetic Guide", "format.phonetic", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "bold", None, "Bold", "format.bold", json!({}), flag("bold"));
                small(ui, app, "italic", None, "Italic", "format.italic", json!({}), flag("italic"));
                split(ui, app, "underline", "Underline", "format.underline", json!({}), flag("underline"), None, |ui, app| {
                    for (l, s) in [
                        ("Single", "single"),
                        ("Double", "double"),
                        ("Thick", "thick"),
                        ("Dotted", "dotted"),
                        ("Dashed", "dash"),
                        ("Dot-dash", "dotDash"),
                        ("Wave", "wave"),
                        ("Words only", "words"),
                    ] {
                        mi(ui, app, l, "format.underline", json!({"style": s}));
                    }
                });
                small(ui, app, "strike", None, "Strikethrough", "format.strikethrough", json!({}), flag("strike"));
                small(ui, app, "subscript", None, "Subscript", "format.subscript", json!({}), flag("subscript"));
                small(ui, app, "superscript", None, "Superscript", "format.superscript", json!({}), flag("superscript"));
                small(ui, app, "charborder", None, "Character Border", "format.border", json!({}), flag("border"));
                small(ui, app, "enclose", None, "Enclose Characters", "format.enclose", json!({}), false);
                ui.add_space(4.0);
                menu_button(ui, app, "effects", None, "Text Effects and Typography", false, "", |ui, app| {
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::wordart_menus::style_gallery(ui, app, "format.textEffects");
                    ui.separator();
                    ui.menu_button(tl!("Text Fill"), |ui| crate::wordart_menus::fill_menu(ui, app, &theme));
                    let outline = crate::wordart_menus::current_outline(app);
                    ui.menu_button(tl!("Text Outline"), |ui| crate::wordart_menus::outline_menu(ui, app, &theme, outline));
                    crate::wordart_menus::effects_menu(ui, app, &theme, false);
                    ui.separator();
                    mi(ui, app, "Outline", "format.outline", json!({}));
                    mi(ui, app, "Shadow", "format.shadow", json!({}));
                    mi(ui, app, "Small Caps", "format.smallCaps", json!({}));
                    mi(ui, app, "All Caps", "format.allCaps", json!({}));
                    mi(ui, app, "Double Strikethrough", "format.doubleStrikethrough", json!({}));
                });
                let hl = app.canvas.last_highlight.clone();
                split(ui, app, "highlight", "Text Highlight Color", "format.highlight", json!({"color": hl}), false, None, |ui, app| {
                    let grid = wordcraft_doc::props::Highlight::ALL;
                    egui::Grid::new("hl").spacing(vec2(3.0, 3.0)).show(ui, |ui| {
                        for (i, h) in grid.iter().enumerate().skip(1) {
                            let c = h.rgb().unwrap_or(wordcraft_doc::Rgb::WHITE);
                            let (r, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
                            ui.painter().rect_filled(r, 2.0, egui::Color32::from_rgb(c.0, c.1, c.2));
                            if resp.on_hover_text(h.name()).clicked() {
                                app.canvas.last_highlight = h.ooxml().to_string();
                                let _ = app.run("format.highlight", json!({"color": h.ooxml()}));
                                ui.close();
                            }
                            if i % 5 == 0 {
                                ui.end_row();
                            }
                        }
                    });
                    mi(ui, app, "No Color", "format.highlight", json!({"color": "none"}));
                });
                let fc = app.canvas.last_font_color.clone();
                let sw = wordcraft_doc::Rgb::parse(&fc).map(crate::theme::c32);
                split(ui, app, "fontcolor", "Font Color", "format.color", json!({"color": fc}), false, sw, |ui, app| {
                    mi(ui, app, "Automatic", "format.color", json!({"color": "auto"}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("format.color").remember(Remember::FontColor));
                });
            });
        });
    });
    group(ui, "Paragraph", app, |ui, app| {
        let rp = app.session.doc.para_at(&app.session.sel.focus).map(|p| app.session.doc.styles.resolve_para(&p.props));
        // The buttons show alignment as seen on the page (Align Right is a right-to-left paragraph's start).
        let align = rp.as_ref().map(|r| r.align.visual(r.bidi)).unwrap_or_default();
        let lists = wordcraft_engine::cmd::para::list_buttons(&app.session.doc, rp.as_ref().and_then(|r| r.numbering));
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                split(ui, app, "bullets", "Bullets", "para.bullets", json!({}), lists.bullets, None, |ui, app| {
                    ui.label(egui::RichText::new(tl!("Bullet Library")).small().weak());
                    ui.horizontal(|ui| {
                        for c in ["•", "○", "▪", "◆", "➢", "✓", "–"] {
                            // The interface fonts have no ➢, so its tile draws the arrowhead.
                            let label = if c == "➢" { " " } else { c };
                            let resp = ui.button(egui::RichText::new(label).size(16.0));
                            if c == "➢" {
                                let m = resp.rect.center();
                                let pts = [(-4.0, -5.0), (5.0, 0.0), (-4.0, 5.0), (-1.5, 0.0)].map(|(x, y)| m + vec2(x, y));
                                ui.painter().add(egui::Shape::convex_polygon(vec![pts[0], pts[1], pts[3]], Tokens::get(ui.ctx()).text, Stroke::NONE));
                                ui.painter().add(egui::Shape::convex_polygon(vec![pts[3], pts[1], pts[2]], Tokens::get(ui.ctx()).text, Stroke::NONE));
                            }
                            if resp.clicked() {
                                let _ = app.run("para.bullets", json!({"kind": c}));
                                ui.close();
                            }
                        }
                    });
                    mi(ui, app, "None", "para.bullets", json!({"off": true}));
                });
                split(ui, app, "numbering", "Numbering", "para.numbering", json!({}), lists.numbering, None, |ui, app| {
                    for (l, k) in [
                        ("1. 2. 3.", "numbered"),
                        ("1) 2) 3)", "numberedParen"),
                        ("I. II. III.", "outline"),
                        ("A. B. C.", "upperLetter"),
                        ("a) b) c)", "lowerLetter"),
                        ("a. b. c.", "lowerLetterDot"),
                        ("i. ii. iii.", "lowerRoman"),
                    ] {
                        mi(ui, app, l, "para.numbering", json!({"kind": k}));
                    }
                    ui.separator();
                    // Word's three Thai number formats.
                    let thai = [("๑. ๒. ๓.", "thaiNumbers"), ("ก. ข. ค.", "thaiLetters"), ("หนึ่ง. สอง. สาม.", "thaiCounting")];
                    for (l, k) in thai {
                        mi(ui, app, l, "para.numbering", json!({"kind": k}));
                    }
                    ui.separator();
                    mi(ui, app, "Restart at 1", "para.restartNumbering", json!({}));
                    mi(ui, app, "None", "para.numbering", json!({"off": true}));
                });
                split(ui, app, "multilevel", "Multilevel List", "para.multilevel", json!({}), lists.multilevel, None, |ui, app| {
                    mi(ui, app, "1. 1.1. 1.1.1.", "para.multilevel", json!({"kind": "legal"}));
                    mi(ui, app, "I. A. 1. a.", "para.multilevel", json!({"kind": "outline"}));
                    mi(ui, app, "๑. ๑.๑. ๑.๑.๑.", "para.multilevel", json!({"kind": "thaiLegal"}));
                    mi(ui, app, "๑. ก. (๑) (ก)", "para.multilevel", json!({"kind": "thaiOutline"}));
                    saved_list_items(ui, app);
                    ui.separator();
                    mi(ui, app, "Define New Multilevel List…", "list.define", json!({}));
                    ui.separator();
                    for lv in 0..5u64 {
                        mi(ui, app, &format!("Change to Level {}", lv + 1), "para.listLevel", json!({"level": lv}));
                    }
                });
                paragraph_separator(ui);
                small(ui, app, "outdent", None, "Decrease Indent", "para.outdent", json!({}), false);
                small(ui, app, "indent", None, "Increase Indent", "para.indent", json!({}), false);
                paragraph_separator(ui);
                small(ui, app, "sort", None, "Sort", "para.sort", json!({}), false);
                let marks = app.session.view.marks;
                small(ui, app, "pilcrow", None, "Show/Hide ¶", "view.marks", json!({}), marks);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                use wordcraft_doc::Align as A;
                small(ui, app, "alignLeft", None, "Align Left", "para.alignLeft", json!({}), align == A::Left);
                small(ui, app, "alignCenter", None, "Center", "para.alignCenter", json!({}), align == A::Center);
                small(ui, app, "alignRight", None, "Align Right", "para.alignRight", json!({}), align == A::Right);
                small(ui, app, "justify", None, "Justify", "para.justify", json!({}), align == A::Justify);
                small(ui, app, "thaiDistribute", None, "Thai Distribution", "para.thaiDistribute", json!({}), align == A::ThaiDistribute);
                small(ui, app, "thaiDigits", None, "Thai Digits", "format.thaiDigits", json!({}), false);
                paragraph_separator(ui);
                menu_button(ui, app, "lineSpacing", None, "Line and Paragraph Spacing", false, "para.lineSpacing", |ui, app| {
                    for v in [1.0, 1.15, 1.5, 2.0, 2.5, 3.0] {
                        mi(ui, app, &format!("{v}"), "para.lineSpacing", json!({"value": v}));
                    }
                    ui.separator();
                    mi(ui, app, "Add Space Before Paragraph", "para.addSpaceBefore", json!({}));
                    mi(ui, app, "Remove Space After Paragraph", "para.removeSpaceAfter", json!({}));
                    mi(ui, app, "Line Spacing Options…", "para.dialog", json!({}));
                });
                paragraph_separator(ui);
                let swatch = crate::colors::parse(&app.canvas.last_shading).map(|c| egui::Color32::from_rgb(c.0, c.1, c.2));
                split(ui, app, "shading", "Shading", "para.shading", json!({"color": app.canvas.last_shading.clone()}), false, swatch, |ui, app| {
                    mi(ui, app, "No Color", "para.shading", json!({"color": null}));
                    let theme = app.session.doc.settings.theme_colors.clone();
                    crate::colors::menu(ui, app, &theme, Target::cmd("para.shading").remember(Remember::Shading));
                });
                // In a table the button sets the selected cells' bottom border, like its menu.
                let cmd = if crate::widgets::enabled(app, "table.borders") { "table.borders" } else { "para.borders" };
                split(ui, app, "borders", "Borders", cmd, json!({"kind": "bottom"}), false, None, border_items);
            });
        });
    });
    group(ui, "Styles", app, |ui, app| {
        crate::previews::style_gallery(app, ui, &st);
    });
    group(ui, "Editing", app, |ui, app| {
        // Two rows: Find, then Replace and Select.
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "find", None, "Find", false, "edit.find", |ui, app| {
                    // As in Word: Find searches in the Navigation pane; Advanced Find is the dialog (#712).
                    mi_icon(ui, app, "Find", "find", "edit.find", json!({}));
                    mi_icon(ui, app, "Advanced Find…", "replace", "edit.advancedFind", json!({}));
                    mi_icon(ui, app, "Go To…", "bookmark", "ui.dialog", json!({"name": "goto"}));
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "replace", None, "Replace", "ui.dialog", json!({"name": "replace"}), false);
                menu_button(ui, app, "select", None, "Select", false, "select.all", |ui, app| {
                    mi_icon(ui, app, "Select All", "select", "select.all", json!({}));
                    mi_icon(ui, app, "Select Paragraph", "pilcrow", "select.paragraph", json!({}));
                    mi_icon(ui, app, "Select Sentence", "textBox", "select.sentence", json!({}));
                });
            });
        });
    });
    group(ui, "Voice", app, |ui, app| {
        big(ui, app, "dictate", "Dictate", "tools.dictate", json!({}), false);
    });
    group(ui, "Editor", app, |ui, app| {
        big(ui, app, "editor", "Editor", "review.spelling", json!({}), false);
    });
}

fn insert(app: &mut WordApp, ui: &mut Ui) {
    // Two-size ribbon: each group's main command large, the rest small in two rows.
    group(ui, "Pages", app, |ui, app| {
        menu_button(ui, app, "coverPage", Some("Cover Page"), "Cover Page", true, "insert.coverPage", |ui, app| {
            mi(ui, app, "Studio cover", "insert.coverPage", json!({}));
        });
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "blankPage", None, "Blank Page", "insert.blankPage", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "pageBreak", None, "Page Break", "insert.pageBreak", json!({}), false);
            });
        });
    });
    group(ui, "Tables", app, |ui, app| {
        menu_button(ui, app, "table", Some("Table"), "Add a Table", true, "insert.table", |ui, app| {
            crate::dialogs::table_grid_picker(ui, app);
            ui.separator();
            pic_item(ui, app, "Insert Table…", None, "table", "ui.dialog", json!({"name": "insertTable"}));
            let drawing = app.canvas.table_tool == Some(crate::table_pen::TableTool::Draw);
            pic_item(ui, app, "Draw Table", Some(drawing), "drawTable", "table.draw", json!({}));
            pic_item(ui, app, "Convert Text to Table…", None, "textToTable", "table.fromText", json!({}));
            ui.menu_button(tl!("Quick Tables"), |ui| {
                mi(ui, app, "Tabular List", "table.quick", json!({"kind": "tabular"}));
                mi(ui, app, "Matrix", "table.quick", json!({"kind": "matrix"}));
                mi(ui, app, "Calendar", "table.quick", json!({"kind": "calendar"}));
            });
        });
    });
    group(ui, "Illustrations", app, |ui, app| {
        big(ui, app, "picture", "Pictures", "insert.picture", json!({}), false);
        menu_button(ui, app, "shapes", Some("Shapes"), "Shapes", true, "insert.shape", |ui, app| {
            for (l, k) in [
                ("Rectangle", "rectangle"),
                ("Rounded Rectangle", "roundedRectangle"),
                ("Oval", "ellipse"),
                ("Triangle", "triangle"),
                ("Diamond", "diamond"),
                ("Line", "line"),
                ("Arrow", "arrow"),
                ("Star", "star"),
                ("Heart", "heart"),
            ] {
                shape_item(ui, app, l, k, "none", "insert.shape", json!({"kind": k}));
            }
            ui.separator();
            shape_item(ui, app, "New Drawing Canvas", "canvas", "none", "insert.canvas", json!({}));
            // Connectors: glue their ends to shapes by dragging them (shape.connect).
            for (l, k, arrow) in [
                ("Straight Connector", "straightConnector", "none"),
                ("Straight Arrow Connector", "straightConnector", "end"),
                ("Elbow Connector", "elbowConnector", "none"),
                ("Elbow Arrow Connector", "elbowConnector", "end"),
                ("Curved Connector", "curvedConnector", "none"),
                ("Curved Arrow Connector", "curvedConnector", "end"),
            ] {
                shape_item(ui, app, l, k, arrow, "insert.shape", json!({"kind": k, "arrow": arrow}));
            }
        });
        big(ui, app, "chart", "Chart", "insert.chart", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "icons", None, "Icons", "insert.icon", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "smartArt", None, "SmartArt", "insert.smartArt", json!({}), false);
            });
        });
    });
    group(ui, "Links", app, |ui, app| {
        big(ui, app, "link", "Link", "ui.dialog", json!({"name": "link"}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "bookmark", None, "Bookmark", "ui.dialog", json!({"name": "bookmark"}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "crossRef", None, "Cross-reference", "insert.crossReference", json!({}), false);
            });
        });
    });
    group(ui, "Comments", app, |ui, app| {
        big(ui, app, "newComment", "Comment", "review.newComment", json!({}), false);
    });
    group(ui, "Header & Footer", app, hf_menus);
    group(ui, "Text", app, |ui, app| {
        big(ui, app, "textBox", "Text\nBox", "insert.textBox", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "quickParts", Some("Quick Parts"), "Quick Parts", false, "insert.quickParts", |ui, app| {
                    let parts: Vec<String> = app.session.building_blocks.keys().cloned().collect();
                    for name in parts {
                        if ui.button(&name).clicked() {
                            let _ = app.run("insert.quickParts", json!({"insert": name}));
                            ui.close();
                        }
                    }
                    mi(ui, app, "Field…", "ui.dialog", json!({"name": "field"}));
                });
                menu_button(ui, app, "wordArt", Some("WordArt"), "WordArt", false, "", |ui, app| {
                    crate::wordart_menus::style_gallery(ui, app, "insert.wordArt")
                });
                small(ui, app, "dropCap", Some("Drop Cap"), "Drop Cap", "insert.dropCap", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "signature", None, "Signature Line", "insert.signatureLine", json!({}), false);
                small(ui, app, "dateTime", None, "Date & Time", "insert.dateTime", json!({}), false);
                small(ui, app, "object", None, "Object", "insert.object", json!({}), false);
            });
        });
    });
    group(ui, "Symbols", app, |ui, app| {
        crate::equation_tab::insert_button(ui, app);
        menu_button(ui, app, "symbol", Some("Symbol"), "Symbol", true, "insert.symbol", |ui, app| {
            // Recently used symbols first (their font on hover), then common ones.
            let mut items: Vec<(String, String)> = app.ui.recent_symbols.iter().map(|r| (r.ch.clone(), r.font.clone())).collect();
            for c in [
                "©", "®", "™", "§", "¶", "€", "£", "¥", "°", "±", "≠", "≤", "≥", "÷", "×", "∞", "µ", "α", "β", "π", "Ω", "∑", "√", "→", "←", "✓",
                "★", "♥", "—", "…",
            ] {
                if !items.iter().any(|(ch, f)| ch == c && f.is_empty()) {
                    items.push((c.to_string(), String::new()));
                }
            }
            items.truncate(30);
            egui::Grid::new("syms").show(ui, |ui| {
                for (i, (c, font)) in items.iter().enumerate() {
                    let b = ui.button(egui::RichText::new(c).size(16.0));
                    let b = if font.is_empty() { b } else { b.on_hover_text(font) };
                    if b.clicked() {
                        let params = if font.is_empty() { json!({"char": c}) } else { json!({"char": c, "font": font}) };
                        if app.run("insert.symbol", params).is_ok() {
                            crate::dialogs_insert::remember_symbol(app, c, font);
                        }
                        ui.close();
                    }
                    if i % 6 == 5 {
                        ui.end_row();
                    }
                }
            });
            ui.separator();
            mi(ui, app, "More Symbols…", "ui.dialog", json!({"name": "symbol"}));
        });
    });
}

fn draw(app: &mut WordApp, ui: &mut Ui) {
    use wordcraft_doc::freeform::InkTool;
    use wordcraft_engine::cmd::draw::{DrawMode, lasso_selection};
    let mode = app.session.view.draw.mode;
    // The pen the colour and thickness menus change: the one in use, else the pen.
    let tool = mode.pen().unwrap_or(InkTool::Pen);
    let set = app.session.view.draw.settings(tool);
    // The gallery pen in use, if any: changing the colour or thickness changes it too.
    let custom =
        app.session.view.draw.pens.iter().position(|p| mode == DrawMode::Pen(p.kind) && p.color == set.color && (p.width - set.width).abs() < 0.01);
    let lasso_on = lasso_selection(&app.session).is_some();
    group(ui, "Drawing Tools", app, |ui, app| {
        big_toggle(ui, app, "select", "Select", "draw.select", mode == DrawMode::Select);
        big_toggle(ui, app, "lasso", "Lasso", "draw.lasso", mode == DrawMode::Lasso);
        big_toggle(ui, app, "eraser", "Eraser", "draw.eraser", mode == DrawMode::Eraser);
        big_toggle(ui, app, "pen", "Pen", "draw.pen", mode == DrawMode::Pen(InkTool::Pen) && custom.is_none());
        big_toggle(ui, app, "pencil", "Pencil", "draw.pencil", mode == DrawMode::Pen(InkTool::Pencil) && custom.is_none());
        big_toggle(ui, app, "highlight", "Highlighter", "draw.highlighter", mode == DrawMode::Pen(InkTool::Highlighter) && custom.is_none());
        // Pens added with Add Pen; right-click one to remove it.
        let pens = app.session.view.draw.pens.clone();
        for (i, pen) in pens.iter().enumerate() {
            gallery_pen(ui, app, i, *pen, custom == Some(i));
        }
        menu_button(ui, app, "addPen", Some("Add\nPen"), "Add Pen", true, "", |ui, app| {
            for (label, kind) in [("Pen", InkTool::Pen), ("Pencil", InkTool::Pencil), ("Highlighter", InkTool::Highlighter)] {
                // A new pen starts as that kind's current colour and thickness.
                let cur = app.session.view.draw.settings(kind);
                mi_icon(
                    ui,
                    app,
                    label,
                    match kind {
                        InkTool::Pen => "pen",
                        InkTool::Pencil => "pencil",
                        InkTool::Highlighter => "highlight",
                    },
                    "draw.addPen",
                    json!({"kind": kind.name(), "color": cur.color.hex(), "width": cur.width}),
                );
            }
        });
        stack(ui, |ui| {
            let sw = Some(crate::theme::c32(set.color));
            split(ui, app, "inkColor", "Color", pen_command(tool), json!({}), false, sw, |ui, app| {
                let theme = app.session.doc.settings.theme_colors.clone();
                // With ink lasso-selected the colour goes to the selection, else to the pen.
                let cmd = if lasso_on { "draw.lassoColor" } else { pen_command(tool) };
                crate::colors::menu(ui, app, &theme, Target::cmd(cmd));
                if !lasso_on {
                    update_gallery_pen(app, custom, tool);
                }
            });
            menu_button(ui, app, "thickness", None, "Thickness", false, "draw.pen", |ui, app| {
                let widths: &[f32] = if tool == InkTool::Highlighter { &[4.0, 8.0, 12.0, 18.0, 24.0] } else { &[0.5, 1.0, 1.5, 2.5, 3.5, 5.0] };
                for w in widths {
                    let label = crate::i18n::fmt(tl!("{n} pt"), &[("n", &w.to_string())]);
                    mi_check(ui, app, &label, (set.width - w).abs() < 0.01, pen_command(tool), json!({"width": w}));
                }
                update_gallery_pen(app, custom, tool);
            });
        });
    });
    group(ui, "Convert", app, |ui, app| {
        let on = app.session.view.draw.ink_to_shape;
        big_toggle(ui, app, "inkToShape", "Ink to\nShape", "draw.inkToShape", on);
        big(ui, app, "inkToMath", "Ink to\nMath", "draw.inkToMath", json!({}), false);
    });
    group(ui, "Insert", app, |ui, app| {
        big(ui, app, "canvas", "Drawing\nCanvas", "insert.canvas", json!({}), false);
    });
    group(ui, "Replay", app, |ui, app| {
        let on = app.session.view.draw.replay;
        big_toggle(ui, app, "replay", "Ink\nReplay", "draw.replay", on);
    });
}

/// The command that picks up `tool`.
fn pen_command(tool: wordcraft_doc::freeform::InkTool) -> &'static str {
    use wordcraft_doc::freeform::InkTool;
    match tool {
        InkTool::Pen => "draw.pen",
        InkTool::Pencil => "draw.pencil",
        InkTool::Highlighter => "draw.highlighter",
    }
}

/// After the colour or thickness menu changed `tool`'s settings, the gallery pen that was in use
/// (`custom`) takes them too.
fn update_gallery_pen(app: &mut WordApp, custom: Option<usize>, tool: wordcraft_doc::freeform::InkTool) {
    let set = app.session.view.draw.settings(tool);
    if let Some(p) = custom.and_then(|i| app.session.view.draw.pens.get_mut(i))
        && p.kind == tool
    {
        p.color = set.color;
        p.width = set.width;
    }
}

/// A pen of the gallery: its kind's icon over a bar of its colour and thickness; a click picks it
/// up, right-click offers Remove Pen.
fn gallery_pen(ui: &mut Ui, app: &mut WordApp, i: usize, pen: wordcraft_engine::cmd::draw::CustomPen, active: bool) {
    use wordcraft_doc::freeform::InkTool;
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(34.0, CONTENT_H), Sense::click());
    crate::widgets::bg(ui, r, &resp, active, &t);
    let icon = match pen.kind {
        InkTool::Pen => "pen",
        InkTool::Pencil => "pencil",
        InkTool::Highlighter => "highlight",
    };
    let color = crate::theme::c32(pen.color);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.center().x, r.min.y + 20.0), vec2(28.0, 28.0)), icon, t.icon, color);
    let bar = (pen.width * 0.6).clamp(1.5, 8.0);
    let y = r.min.y + 46.0;
    ui.painter().line_segment([pos2(r.min.x + 6.0, y), pos2(r.max.x - 6.0, y)], Stroke::new(bar, color));
    let kind = tl!(match pen.kind {
        InkTool::Pen => "Pen",
        InkTool::Pencil => "Pencil",
        InkTool::Highlighter => "Highlighter",
    });
    let size = crate::i18n::fmt(tl!("{n} pt"), &[("n", &pen.width.to_string())]);
    let resp = resp.on_hover_text(format!("{kind}: {size}, #{}", pen.color.hex()));
    if resp.clicked() {
        let _ = app.run(pen_command(pen.kind), json!({"color": pen.color.hex(), "width": pen.width}));
    }
    resp.context_menu(|ui| {
        if ui.button(tl!("Remove Pen")).clicked() {
            let _ = app.run("draw.removePen", json!({"index": i}));
            ui.close();
        }
    });
}

/// The Design tab's style set gallery in a box of its own, sized from its thumbnails and clipped
/// to it, with a gap on each side: the Colors and Fonts column can't be drawn over, or cut by,
/// the thumbnails (#638). Returns the box and the space the thumbnails used.
fn style_set_box(app: &mut WordApp, ui: &mut Ui) -> (Rect, Rect) {
    ui.add_space(4.0);
    let (gallery, _) = ui.allocate_exact_size(vec2(crate::previews::STYLE_SET_GALLERY_W, CONTENT_H), Sense::hover());
    let mut gui = ui.new_child(egui::UiBuilder::new().max_rect(gallery).layout(egui::Layout::left_to_right(egui::Align::Min)));
    gui.set_clip_rect(gallery.intersect(ui.clip_rect()));
    gui.spacing_mut().item_spacing = vec2(1.0, 1.0);
    crate::previews::style_set_gallery(app, &mut gui);
    ui.add_space(8.0);
    (gallery, gui.min_rect())
}

fn design(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Themes", app, |ui, app| {
        menu_button(ui, app, "themes", Some("Themes"), "Themes", true, "design.themes", |ui, app| {
            for (name, ..) in wordcraft_engine::cmd::design::THEMES {
                mi(ui, app, name, "design.theme", json!({"name": name}));
            }
        });
    });
    group(ui, "Document Formatting", app, |ui, app| {
        style_set_box(app, ui);
        stack(ui, |ui| {
            menu_button(ui, app, "colors", Some("Colors"), "Theme Colors", false, "design.themeColors", |ui, app| {
                for (name, ..) in wordcraft_engine::cmd::design::THEMES {
                    mi(ui, app, name, "design.themeColors", json!({"name": name}));
                }
            });
            menu_button(ui, app, "fonts", Some("Fonts"), "Theme Fonts", false, "design.themeFonts", |ui, app| {
                for (name, h, b, _) in wordcraft_engine::cmd::design::THEMES {
                    mi_text(ui, app, &format!("{}: {h} / {b}", tl!(name)), "design.themeFonts", json!({"heading": h, "body": b}));
                }
            });
        });
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "paraSpacing", Some("Paragraph Spacing"), "Paragraph Spacing", false, "design.paragraphSpacing", |ui, app| {
                    for (l, v) in [
                        ("Default", "default"),
                        ("No Paragraph Space", "none"),
                        ("Compact", "compact"),
                        ("Tight", "tight"),
                        ("Open", "open"),
                        ("Relaxed", "relaxed"),
                        ("Double", "double"),
                    ] {
                        mi_icon(ui, app, l, &format!("paraSpacing_{v}"), "design.paragraphSpacing", json!({"value": v}));
                    }
                });
                small(ui, app, "effectsDesign", Some("Effects"), "Effects", "design.effects", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "setDefault", Some("Set as Default"), "Set as Default", "design.setDefault", json!({}), false);
            });
        });
    });
    group(ui, "Page Background", app, |ui, app| {
        menu_button(ui, app, "watermark", Some("Watermark"), "Watermark", true, "design.watermark", |ui, app| {
            for w in ["DRAFT", "CONFIDENTIAL", "DO NOT COPY", "SAMPLE", "ASAP", "URGENT"] {
                mi(ui, app, w, "design.watermark", json!({"text": w}));
            }
            ui.separator();
            mi(ui, app, "Custom Watermark…", "ui.dialog", json!({"name": "watermark"}));
            mi(ui, app, "Remove Watermark", "design.watermark", json!({"remove": true}));
        });
        menu_button(ui, app, "pageColor", Some("Page\nColor"), "Page Color", true, "design.pageColor", |ui, app| {
            mi(ui, app, "No Color", "design.pageColor", json!({"color": null}));
            let theme = app.session.doc.settings.theme_colors.clone();
            crate::colors::menu(ui, app, &theme, Target::cmd("design.pageColor"));
        });
        big(ui, app, "pageBorders", "Page\nBorders", "design.pageBorders", json!({}), false);
    });
}

// Layout › Margins, Orientation, Size and Columns: the caret section's setting has a check
// mark, as Text Direction's does (#1206).
fn margins_items(ui: &mut Ui, app: &mut WordApp) {
    let current = wordcraft_engine::cmd::page::margins_preset_at(&app.session);
    // The two uniform presets name their margin in the measurement unit (`2,54 cm`, #838).
    for (l, k, uniform) in [
        ("Normal  {size} all", "normal", Some(72.0)),
        ("Narrow  {size} all", "narrow", Some(36.0)),
        ("Moderate", "moderate", None),
        ("Wide", "wide", None),
        ("Mirrored", "mirrored", None),
        ("Office 2003 Default", "office2003", None),
    ] {
        let label = match uniform {
            Some(pt) => crate::i18n::fmt(tl!(l), &[("size", &crate::options::length(pt))]),
            None => l.to_string(),
        };
        // Mirrored margins come from the document as well as from the preset (#1181).
        let on = current == Some(k) || (k == "mirrored" && app.session.doc.settings.mirror_margins);
        pic_item(ui, app, &label, Some(on), &format!("mg_{k}"), "layout.margins", json!({"preset": k}));
    }
    pic_item(ui, app, "Custom Margins…", None, "mg_custom", "ui.dialog", json!({"name": "pageSetup"}));
}

fn orientation_items(ui: &mut Ui, app: &mut WordApp) {
    let land = wordcraft_engine::cmd::page::sect(&app.session).landscape;
    pic_item(ui, app, "Portrait", Some(!land), "portrait", "layout.orientation", json!({"value": "portrait"}));
    pic_item(ui, app, "Landscape", Some(land), "landscape", "layout.orientation", json!({"value": "landscape"}));
}

fn columns_items(ui: &mut Ui, app: &mut WordApp) {
    let current = wordcraft_engine::cmd::page::columns_preset_at(&app.session);
    for (l, k, params) in [
        ("One", "one", json!({"count": 1})),
        ("Two", "two", json!({"count": 2})),
        ("Three", "three", json!({"count": 3})),
        ("Left", "left", json!({"preset": "left"})),
        ("Right", "right", json!({"preset": "right"})),
    ] {
        pic_item(ui, app, l, Some(current == Some(k)), &format!("cols_{k}"), "layout.columns", params);
    }
    ui.separator();
    pic_item(ui, app, "More Columns…", None, "colsMore", "ui.dialog", json!({"name": "columns"}));
}

fn layout(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Page Setup", app, |ui, app| {
        // Text Direction works on what the caret is in: table cells, a text box, else the section.
        let dir = wordcraft_engine::cmd::page::text_direction_at(&app.session);
        menu_button(ui, app, "textDirection", Some("Text\nDirection"), "Text Direction", true, "layout.textDirection", |ui, app| {
            use wordcraft_doc::props::TextDirection;
            for (label, d) in
                [("Horizontal", TextDirection::Horizontal), ("Rotate all text 90°", TextDirection::Down), ("Rotate all text 270°", TextDirection::Up)]
            {
                let pic = match d {
                    TextDirection::Down => "dirDown",
                    TextDirection::Up => "dirUp",
                    _ => "dirHorizontal",
                };
                pic_item(ui, app, label, Some(dir == d), pic, "layout.textDirection", json!({"value": d.ooxml()}));
            }
        });
        menu_button(ui, app, "margins", Some("Margins"), "Margins", true, "layout.margins", margins_items);
        menu_button(ui, app, "orientation", Some("Orientation"), "Orientation", true, "layout.orientation", orientation_items);
        menu_button(ui, app, "size", Some("Size"), "Size", true, "layout.size", size_menu);
        menu_button(ui, app, "columns", Some("Columns"), "Columns", true, "layout.columns", columns_items);
        // Large, as in OnlyOffice: each opens a set of choices.
        menu_button(ui, app, "breaks", Some("Breaks"), "Breaks", true, "layout.break", |ui, app| {
            ui.label(egui::RichText::new(tl!("Page Breaks")).strong());
            for (l, k) in [("Page", "page"), ("Column", "column"), ("Text Wrapping", "textWrapping")] {
                pic_item(ui, app, l, None, &format!("brk_{k}"), "layout.break", json!({"kind": k}));
            }
            ui.label(egui::RichText::new(tl!("Section Breaks")).strong());
            for (l, k) in [("Next Page", "nextPage"), ("Continuous", "continuous"), ("Even Page", "evenPage"), ("Odd Page", "oddPage")] {
                pic_item(ui, app, l, None, &format!("brk_{k}"), "layout.break", json!({"kind": k}));
            }
        });
        menu_button(ui, app, "lineNumbers", Some("Line\nNumbers"), "Line Numbers", true, "layout.lineNumbers", |ui, app| {
            for (l, v) in
                [("None", "none"), ("Continuous", "continuous"), ("Restart Each Page", "restartPage"), ("Restart Each Section", "restartSection")]
            {
                pic_item(ui, app, l, None, &format!("ln_{v}"), "layout.lineNumbers", json!({"value": v}));
            }
            ui.separator();
            pic_item(ui, app, "Line Numbering Options…", None, "lnOptions", "ui.dialog", json!({"name": "lineNumbers"}));
        });
        let hy = app.session.doc.settings.auto_hyphenation;
        menu_button(ui, app, "hyphenation", Some("Hyphenation"), "Hyphenation", true, "layout.hyphenation", |ui, app| {
            mi(ui, app, if hy { "✓ Automatic" } else { "Automatic" }, "layout.hyphenation", json!({"value": true}));
            mi(ui, app, if hy { "None" } else { "✓ None" }, "layout.hyphenation", json!({"value": false}));
            mi(ui, app, "Manual…", "ui.dialog", json!({"name": "manualHyphenation"}));
            ui.separator();
            mi(ui, app, "Hyphenation Options…", "ui.dialog", json!({"name": "hyphenation"}));
        });
    });
    group(ui, "Paragraph", app, |ui, app| {
        let rp = app.session.doc.para_at(&app.session.sel.focus).map(|p| app.session.doc.styles.resolve_para(&p.props));
        let Some(rp) = rp else { return };
        // Two rows of fields, as the two-size ribbon's two rows; each label names what it sets
        // (no header row); labels and fields sit on shared columns, every field the same width.
        const FIELD: egui::Vec2 = egui::Vec2::new(64.0, 20.0);
        ui.spacing_mut().interact_size.y = FIELD.y;
        // The gap before the Spacing column sits inside its cells: `add_space` in a grid moves the
        // next cell in release builds but trips a debug assertion.
        let spaced = |ui: &mut Ui, text: egui::RichText| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(text);
            });
        };
        egui::Grid::new("layout_para").spacing(vec2(4.0, crate::widgets::ROW_GAP + 8.0)).min_row_height(FIELD.y).show(ui, |ui| {
            // Indents in the File › Options unit.
            let u = crate::options::unit();
            let k = u.pt_per_unit();
            let mut l = rp.indent_left / k;
            let mut b = rp.space_before;
            ui.label(egui::RichText::new(tl!("Left indent:")).small());
            if ui
                .add_sized(FIELD, crate::numfield::NumField::new(&mut l).speed(3.6 / k).range(-792.0 / k..=1584.0 / k).measure(u).max_decimals(2))
                .changed()
            {
                let _ = app.run("para.indents", json!({"left": l * k}));
            }
            spaced(ui, egui::RichText::new(tl!("Space before:")).small());
            if ui.add_sized(FIELD, crate::numfield::NumField::new(&mut b).speed(1.0).range(0.0..=1584.0).suffix(format!(" {}", tl!("pt")))).changed()
            {
                let _ = app.run("para.spacing", json!({"before": b}));
            }
            ui.end_row();
            let mut r = rp.indent_right / k;
            let mut a = rp.space_after;
            ui.label(egui::RichText::new(tl!("Right indent:")).small());
            if ui
                .add_sized(FIELD, crate::numfield::NumField::new(&mut r).speed(3.6 / k).range(-792.0 / k..=1584.0 / k).measure(u).max_decimals(2))
                .changed()
            {
                let _ = app.run("para.indents", json!({"right": r * k}));
            }
            spaced(ui, egui::RichText::new(tl!("Space after:")).small());
            if ui.add_sized(FIELD, crate::numfield::NumField::new(&mut a).speed(1.0).range(0.0..=1584.0).suffix(format!(" {}", tl!("pt")))).changed()
            {
                let _ = app.run("para.spacing", json!({"after": a}));
            }
            ui.end_row();
        });
    });
    group(ui, "Arrange", app, |ui, app| {
        big(ui, app, "position", "Position", "arrange.position", json!({}), false);
        wrap_menu(ui, app);
        // One two-row block: order (forward, backward, Selection Pane), then align, group, rotate.
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "bringForward", None, "Bring Forward", "arrange.bringForward", json!({}), false);
                small(ui, app, "sendBackward", None, "Send Backward", "arrange.sendBackward", json!({}), false);
                small(ui, app, "selectionPane", None, "Selection Pane", "arrange.selectionPane", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "align", None, "Align", "arrange.align", json!({}), false);
                group_menu(ui, app, None);
                rotate_menu(ui, app, None);
            });
        });
    });
}

/// Arrange › Rotate: turn 90° either way or flip (the angle and mirroring, not the pixels).
/// Arrange › Wrap Text: how the selected object sits in the text, and (Square, Tight, Through)
/// which of its sides text flows on.
fn wrap_menu(ui: &mut Ui, app: &mut WordApp) {
    let (wrap, side) = wordcraft_engine::cmd::objects::selected(&app.session)
        .and_then(|(_, o)| o.frame().map(|(_, _, f)| (f.wrap, f.wrap_text)))
        .unwrap_or_default();
    menu_button(ui, app, "wrapText", Some("Wrap\nText"), "Wrap Text", true, "arrange.wrapText", |ui, app| {
        use wordcraft_doc::para::{Wrap, WrapText};
        for (l, w, v) in [
            ("In Line with Text", Wrap::Inline, "inline"),
            ("Square", Wrap::Square, "square"),
            ("Tight", Wrap::Tight, "tight"),
            ("Through", Wrap::Through, "through"),
            ("Top and Bottom", Wrap::TopAndBottom, "topAndBottom"),
            ("Behind Text", Wrap::BehindText, "behindText"),
            ("In Front of Text", Wrap::InFrontOfText, "inFrontOfText"),
        ] {
            mi_check(ui, app, l, wrap == w, "arrange.wrap", json!({"wrap": v}));
        }
        ui.separator();
        ui.menu_button(tl!("Wrap Text Sides"), |ui| {
            for (l, s, v) in [
                ("Both Sides", WrapText::BothSides, "bothSides"),
                ("Left Only", WrapText::Left, "left"),
                ("Right Only", WrapText::Right, "right"),
                ("Largest Only", WrapText::Largest, "largest"),
            ] {
                mi_check(ui, app, l, side == s, "arrange.wrapText", json!({"side": v}));
            }
        });
    });
}

fn rotate_menu(ui: &mut Ui, app: &mut WordApp, label: Option<&str>) {
    menu_button(ui, app, "rotate", label, "Rotate", false, "arrange.rotate", |ui, app| {
        mi(ui, app, "Rotate Right 90°", "arrange.rotate", json!({"direction": "right"}));
        mi(ui, app, "Rotate Left 90°", "arrange.rotate", json!({"direction": "left"}));
        mi(ui, app, "Flip Vertical", "arrange.rotate", json!({"direction": "flipVertical"}));
        mi(ui, app, "Flip Horizontal", "arrange.rotate", json!({"direction": "flipHorizontal"}));
    });
}

/// Size › Rotation: the selected object's exact angle (a drag of the field is one undo step).
fn rotation_field(ui: &mut Ui, app: &mut WordApp) {
    let deg0 = wordcraft_engine::cmd::objects::selected(&app.session).and_then(|(_, o)| o.frame().map(|(_, _, f)| f.spin().deg)).unwrap_or(0.0);
    ui.label(egui::RichText::new(tl!("Rotation:")).small());
    let mut deg = deg0;
    let r = ui.add(crate::numfield::NumField::new(&mut deg).speed(1.0).range(-360.0..=360.0).max_decimals(1).suffix("°"));
    if r.changed() {
        if r.dragged() && !r.drag_started() {
            app.session.join_next_undo();
        }
        let _ = app.run("arrange.rotation", json!({"degrees": deg}));
    }
}

/// Arrange › Group: Group (Shift+click objects to select several) and Ungroup.
fn group_menu(ui: &mut Ui, app: &mut WordApp, label: Option<&str>) {
    menu_button(ui, app, "group", label, "Group", false, "arrange.group", |ui, app| {
        mi(ui, app, "Group", "arrange.group", json!({}));
        mi(ui, app, "Ungroup", "arrange.ungroup", json!({}));
    });
}

fn references(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Table of Contents", app, |ui, app| {
        menu_button(ui, app, "toc", Some("Table of\nContents"), "Table of Contents", true, "references.toc", |ui, app| {
            mi(ui, app, "Automatic Table 1 (3 levels)", "references.toc", json!({"levels": 3}));
            mi(ui, app, "Automatic Table 2 (2 levels)", "references.toc", json!({"levels": 2, "title": "Table of Contents"}));
            ui.separator();
            mi(ui, app, "Custom Table of Contents…", "ui.dialog", json!({"name": "toc"}));
            mi(ui, app, "Remove Table of Contents", "references.removeToc", json!({}));
        });
        stack(ui, |ui| {
            menu_button(ui, app, "addText", Some("Add Text"), "Add Text", false, "references.addText", |ui, app| {
                mi(ui, app, "Do Not Show in TOC", "references.addText", json!({"level": 0}));
                for l in 1..=3u64 {
                    mi(ui, app, &format!("Level {l}"), "references.addText", json!({"level": l}));
                }
            });
            small(ui, app, "updateTable", Some("Update Table"), "Update Table", "references.updateToc", json!({}), false);
        });
    });
    group(ui, "Footnotes", app, |ui, app| {
        big(ui, app, "footnote", "Insert\nFootnote", "references.footnote", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "endnote", Some("Insert Endnote"), "Insert Endnote", "references.endnote", json!({}), false);
                small(ui, app, "nextFootnote", Some("Next Footnote"), "Next Footnote", "references.nextFootnote", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "showNotes", Some("Show Notes"), "Show Notes", "references.notes", json!({}), false);
                // Footnote and Endnote options (the group had a launcher for them).
                small(ui, app, "properties", None, "Footnote and Endnote", "references.noteOptions", json!({}), false);
            });
        });
    });
    group(ui, "Research", app, |ui, app| {
        big(ui, app, "researcher", "Researcher", "references.researcher", json!({}), false);
    });
    group(ui, "Citations & Bibliography", app, |ui, app| {
        // The document's sources to cite, then a new one (Create Source, #402).
        menu_button(ui, app, "citation", Some("Insert\nCitation"), "Insert Citation", true, "", |ui, app| {
            let sources: Vec<(String, String)> =
                app.session.doc.sources.iter().map(|s| (s.tag.clone(), format!("{} — {}", s.author, s.title))).collect();
            for (tag, label) in sources {
                mi_text(ui, app, &label, "references.citation", json!({"tag": tag}));
            }
            if !app.session.doc.sources.is_empty() {
                ui.separator();
            }
            mi(ui, app, "Add New Source…", "ui.dialog", json!({"name": "createSource"}));
        });
        stack(ui, |ui| {
            let current = app.session.bib_style.clone();
            crate::widgets::row(ui, |ui| {
                small(ui, app, "sources", None, "Manage Sources", "references.sources", json!({}), false);
                let tip = crate::i18n::fmt(tl!("Style: {style}"), &[("style", &current)]);
                menu_button(ui, app, "styles", None, &tip, false, "references.citationStyle", |ui, app| {
                    for st in &wordcraft_engine::cmd::citations::STYLES {
                        mi_check(ui, app, st.name, st.name == current, "references.citationStyle", json!({"style": st.name}));
                    }
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "bibliography", None, "Bibliography", "references.bibliography", json!({}), false);
            });
        });
    });
    group(ui, "Captions", app, |ui, app| {
        big(ui, app, "caption", "Insert\nCaption", "references.caption", json!({}), false);
        stack(ui, |ui| {
            small(
                ui,
                app,
                "tableOfFigures",
                Some("Insert Table of Figures"),
                "Insert Table of Figures",
                "references.tableOfFigures",
                json!({}),
                false,
            );
            small(
                ui,
                app,
                "tableOfTables",
                Some("Insert Table of Tables"),
                "Insert Table of Tables",
                "references.tableOfFigures",
                json!({"kind": "table"}),
                false,
            );
        });
        stack(ui, |ui| {
            small(ui, app, "update", Some("Update Table"), "Update Table of Figures", "references.updateFigures", json!({}), false);
            small(ui, app, "crossRef", Some("Cross-reference"), "Cross-reference", "insert.crossReference", json!({}), false);
        });
    });
    group(ui, "Index", app, |ui, app| {
        big(ui, app, "markEntry", "Mark\nEntry", "references.markEntry", json!({}), false);
        stack(ui, |ui| {
            small(ui, app, "index", Some("Insert Index"), "Insert Index", "references.index", json!({}), false);
            small(ui, app, "update", Some("Update Index"), "Update Index", "references.updateIndex", json!({}), false);
        });
    });
    group(ui, "Table of Authorities", app, |ui, app| {
        big(ui, app, "markCitation", "Mark\nCitation", "references.markCitation", json!({}), false);
        stack(ui, |ui| {
            small(
                ui,
                app,
                "tableOfAuthorities",
                Some("Insert Table of Authorities"),
                "Insert Table of Authorities",
                "references.tableOfAuthorities",
                json!({}),
                false,
            );
            small(ui, app, "update", Some("Update Table"), "Update Table", "references.updateAuthorities", json!({}), false);
        });
    });
}

fn mailings(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Create", app, |ui, app| {
        big(ui, app, "envelope", "Envelopes", "mailings.envelopes", json!({}), false);
        big(ui, app, "labels", "Labels", "mailings.labels", json!({}), false);
    });
    group(ui, "Start Mail Merge", app, |ui, app| {
        // The kinds of merge document the engine makes, the current one checked.
        menu_button(ui, app, "mailMerge", Some("Start Mail\nMerge"), "Start Mail Merge", true, "mailings.start", |ui, app| {
            let kind = app.session.merge.kind.clone();
            for (label, k) in [
                ("Letters", "letters"),
                ("E-mail Messages", "emails"),
                ("Envelopes", "envelopes"),
                ("Labels", "labels"),
                ("Directory", "directory"),
                ("Normal Word Document", "normal"),
            ] {
                let current = kind == k || (k == "normal" && kind.is_empty());
                mi_check(ui, app, label, current, "mailings.start", json!({"kind": k}));
            }
        });
        // The command needs data, so the button offers the two ways to give it (#240).
        menu_button(ui, app, "recipients", Some("Select\nRecipients"), "Select Recipients", true, "mailings.recipients", |ui, app| {
            mi(ui, app, "Type a New List…", "ui.dialog", json!({"name": "newRecipientList"}));
            mi(ui, app, "Use an Existing List…", "ui.openRecipientList", json!({}));
        });
        big(ui, app, "editRecipients", "Edit\nRecipient List", "mailings.editRecipients", json!({}), false);
    });
    group(ui, "Write & Insert Fields", app, |ui, app| {
        big(ui, app, "highlightFields", "Highlight\nMerge Fields", "mailings.highlightFields", json!({}), false);
        big(ui, app, "addressBlock", "Address\nBlock", "mailings.addressBlock", json!({}), false);
        big(ui, app, "greetingLine", "Greeting\nLine", "mailings.greetingLine", json!({}), false);
        big(ui, app, "mergeField", "Insert Merge\nField", "mailings.insertField", json!({}), false);
        stack(ui, |ui| {
            // The merge rules the engine knows; If and Skip Record If ask for their condition.
            menu_button(ui, app, "rules", Some("Rules"), "Rules", false, "mailings.rules", |ui, app| {
                mi(ui, app, "If…Then…Else…", "mailings.rules", json!({"rule": "IF"}));
                mi(ui, app, "Merge Record #", "mailings.rules", json!({"rule": "MERGEREC"}));
                mi(ui, app, "Next Record", "mailings.rules", json!({"rule": "NEXT"}));
                mi(ui, app, "Skip Record If…", "mailings.rules", json!({"rule": "SKIPIF"}));
            });
            small(ui, app, "matchFields", Some("Match Fields"), "Match Fields", "mailings.matchFields", json!({}), false);
        });
    });
    group(ui, "Preview Results", app, |ui, app| {
        big(ui, app, "preview", "Preview\nResults", "mailings.preview", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "previous", None, "Previous Record", "mailings.previous", json!({}), false);
                small(ui, app, "next", None, "Next Record", "mailings.next", json!({}), false);
            });
            small(ui, app, "find", Some("Find Recipient"), "Find Recipient", "mailings.findRecipient", json!({}), false);
            small(ui, app, "checkErrors", Some("Check for Errors"), "Check for Errors", "mailings.checkErrors", json!({}), false);
        });
    });
    group(ui, "Finish", app, |ui, app| {
        big(ui, app, "finish", "Finish &\nMerge", "mailings.finish", json!({}), false);
    });
}

/// Zotero (`docs/zotero.md`): the Zotero desktop app does the work in its own window.
fn zotero(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Citations", app, |ui, app| {
        big(ui, app, "citation", "Add/Edit\nCitation", "ui.zotero.addEditCitation", json!({}), false);
        big(ui, app, "addNote", "Add\nNote", "ui.zotero.addNote", json!({}), false);
        big(ui, app, "pastCitation", "Move Past\nCitation", "caret.pastCitation", json!({}), false);
    });
    group(ui, "Bibliography", app, |ui, app| {
        big(ui, app, "bibliography", "Add/Edit\nBibliography", "ui.zotero.addEditBibliography", json!({}), false);
    });
    group(ui, "Document", app, |ui, app| {
        big(ui, app, "update", "Refresh", "ui.zotero.refresh", json!({}), false);
        big(ui, app, "docPrefs", "Document\nPreferences", "ui.zotero.setDocPrefs", json!({}), false);
        big(ui, app, "unlinkCitations", "Unlink\nCitations", "ui.zotero.removeCodes", json!({}), false);
    });
}

fn review(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Proofing", app, |ui, app| {
        big(ui, app, "spelling", "Spelling &\nGrammar", "review.spelling", json!({}), false);
        stack(ui, |ui| {
            small(ui, app, "thesaurus", Some("Thesaurus"), "Thesaurus", "review.thesaurus", json!({}), false);
            small(ui, app, "wordCount", Some("Word Count"), "Word Count", "ui.dialog", json!({"name": "wordCount"}), false);
        });
    });
    group(ui, "Speech", app, |ui, app| {
        big(ui, app, "readAloud", "Read\nAloud", "review.readAloud", json!({}), false);
    });
    group(ui, "Accessibility", app, |ui, app| {
        big(ui, app, "accessibility", "Check\nAccessibility", "file.accessibility", json!({}), false);
    });
    group(ui, "Language", app, |ui, app| {
        big(ui, app, "translate", "Translate", "review.translate", json!({}), false);
        menu_button(ui, app, "language", Some("Language"), "Language", true, "", language_menu);
    });
    group(ui, "Comments", app, |ui, app| {
        big(ui, app, "newComment", "New\nComment", "review.newComment", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "deleteComment", Some("Delete"), "Delete Comment", "review.deleteComment", json!({}), false);
                small(ui, app, "prevComment", Some("Previous"), "Previous Comment", "review.previousComment", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "nextComment", Some("Next"), "Next Comment", "review.nextComment", json!({}), false);
            });
        });
        let shown = app.session.view.comments_pane;
        big(ui, app, "showComments", "Show\nComments", "view.commentsPane", json!({"value": !shown}), false);
    });
    group(ui, "Tracking", app, |ui, app| {
        let on = app.session.doc.settings.track_changes;
        let r = big(ui, app, "trackChanges", if on { "Track\nChanges ✓" } else { "Track\nChanges" }, "review.trackChanges", json!({}), false);
        let _ = r;
        stack(ui, |ui| {
            menu_button(
                ui,
                app,
                "markup",
                Some(if app.session.view.show_markup { "All Markup" } else { "No Markup" }),
                "Display for Review",
                false,
                "review.markup",
                |ui, app| {
                    mi(ui, app, "All Markup", "review.markup", json!({"value": "all"}));
                    mi(ui, app, "No Markup", "review.markup", json!({"value": "noMarkup"}));
                    // Track Changes Options (the group had a launcher for them).
                    ui.separator();
                    mi(ui, app, "Track Changes Options", "review.trackingOptions", json!({}));
                },
            );
            small(ui, app, "reviewingPane", Some("Reviewing Pane"), "Reviewing Pane", "review.changes", json!({}), false);
        });
    });
    group(ui, "Changes", app, |ui, app| {
        menu_button(ui, app, "accept", Some("Accept"), "Accept", true, "review.accept", |ui, app| {
            mi(ui, app, "Accept This Change", "review.accept", json!({}));
            mi(ui, app, "Accept All Changes", "review.acceptAll", json!({}));
        });
        menu_button(ui, app, "reject", Some("Reject"), "Reject", true, "review.reject", |ui, app| {
            mi(ui, app, "Reject Change", "review.reject", json!({}));
            mi(ui, app, "Reject All Changes", "review.rejectAll", json!({}));
        });
        stack(ui, |ui| {
            small(ui, app, "prevChange", Some("Previous"), "Previous Change", "review.previousChange", json!({}), false);
            small(ui, app, "nextChange", Some("Next"), "Next Change", "review.nextChange", json!({}), false);
        });
    });
    group(ui, "Compare", app, |ui, app| {
        big(ui, app, "compare", "Compare", "review.compare", json!({}), false);
    });
    group(ui, "Protect", app, |ui, app| {
        big(ui, app, "blockAuthors", "Block\nAuthors", "review.blockAuthors", json!({}), false);
        big(ui, app, "restrict", "Restrict\nEditing", "review.restrict", json!({}), false);
    });
    group(ui, "Ink", app, |ui, app| {
        let hidden = app.session.view.hide_ink;
        big_toggle(ui, app, "hideInk", "Hide\nInk", "review.hideInk", hidden);
    });
}

/// Developer › Controls: content controls.
fn developer(app: &mut WordApp, ui: &mut Ui) {
    let design = app.session.view.design_mode;
    group(ui, "Controls", app, |ui, app| {
        let rows: [[(&str, &str, &str); 3]; 3] = [
            [
                ("ccRichText", "Rich Text Content Control", "developer.richText"),
                ("ccPlainText", "Plain Text Content Control", "developer.plainText"),
                ("ccPicture", "Picture Content Control", "developer.picture"),
            ],
            [
                ("ccGallery", "Building Block Gallery Content Control", "developer.buildingBlock"),
                ("ccCheckBox", "Check Box Content Control", "developer.checkBox"),
                ("ccComboBox", "Combo Box Content Control", "developer.comboBox"),
            ],
            [
                ("ccDropDown", "Drop-Down List Content Control", "developer.dropDown"),
                ("ccDate", "Date Picker Content Control", "developer.datePicker"),
                ("ccRepeat", "Repeating Section Content Control", "developer.repeatingSection"),
            ],
        ];
        // Three rows of small buttons fill the group's height, like every other stack.
        stack(ui, |ui| {
            for row in rows {
                crate::widgets::row(ui, |ui| {
                    for (icon, tip, id) in row {
                        small(ui, app, icon, None, tip, id, json!({}), false);
                    }
                });
            }
        });
        stack(ui, |ui| {
            small(ui, app, "designMode", Some("Design Mode"), "Design Mode", "developer.designMode", json!({}), design);
            small(ui, app, "properties", Some("Properties"), "Properties", "developer.properties", json!({}), false);
        });
    });
}

fn view(app: &mut WordApp, ui: &mut Ui) {
    let v = app.session.view.clone();
    group(ui, "Views", app, |ui, app| {
        big(ui, app, "readMode", "Read\nMode", "view.readMode", json!({}), false);
        big(ui, app, "printLayout", "Print\nLayout", "view.printLayout", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "outline", Some("Outline"), "Outline", "view.outline", json!({}), false);
                small(ui, app, "draft", Some("Draft"), "Draft", "view.draft", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "webLayout", Some("Web Layout"), "Web Layout", "view.webLayout", json!({}), false);
            });
        });
    });
    group(ui, "Immersive", app, |ui, app| {
        big(ui, app, "focus", "Focus", "view.focus", json!({}), false);
        big(ui, app, "immersive", "Immersive\nReader", "view.immersive", json!({}), false);
    });
    group(ui, "Show", app, |ui, app| {
        stack(ui, |ui| {
            ui.horizontal(|ui| {
                small(ui, app, "ruler", Some("Ruler"), "Ruler", "view.ruler", json!({}), v.ruler);
                small(ui, app, "gridlines", Some("Gridlines"), "Gridlines", "view.gridlines", json!({}), v.gridlines);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            ui.horizontal(|ui| {
                small(ui, app, "textBoundaries", Some("Text Boundaries"), "Text Boundaries", "view.textBoundaries", json!({}), v.text_boundaries);
                let available = crate::canvas::can_hide_white_space(app);
                ui.add_enabled_ui(available, |ui| {
                    small(
                        ui,
                        app,
                        "hideWhiteSpace",
                        Some("Hide White Space"),
                        "Hide White Space",
                        "view.hideWhiteSpace",
                        json!({}),
                        v.hide_white_space && available,
                    )
                    .on_disabled_hover_text(tl!("Available in single-column Print Layout outside headers and footers."));
                });
            });
        });
        big_toggle(ui, app, "navPane", "Navigation\nPane", "view.navigationPane", v.nav_pane);
    });
    group(ui, "Zoom", app, |ui, app| {
        big(ui, app, "printPreview", "Print\nPreview", "ui.printPreview", json!({}), false);
        stack(ui, |ui| {
            ui.horizontal(|ui| {
                small(ui, app, "zoom", Some("Zoom"), "Zoom", "ui.dialog", json!({"name": "zoom"}), false);
                paragraph_separator(ui);
                small(ui, app, "zoomOut", None, "Zoom Out", "view.zoomOut", json!({}), false);
                small(ui, app, "zoom100", Some("100%"), "100%", "view.zoom100", json!({}), false);
                small(ui, app, "zoomIn", None, "Zoom In", "view.zoomIn", json!({}), false);
            });
            ui.horizontal(|ui| {
                small(ui, app, "onePage", Some("One Page"), "One Page", "view.onePage", json!({}), v.fit == "onePage");
                small(ui, app, "multiplePages", Some("Multiple Pages"), "Multiple Pages", "view.multiplePages", json!({}), v.multi_page);
            });
        });
        big_toggle(ui, app, "pageWidth", "Page\nWidth", "view.pageWidth", v.fit == "pageWidth");
    });
    group(ui, "Dark Mode", app, |ui, app| {
        big(ui, app, "darkMode", "Switch\nModes", "view.darkMode", json!({}), false);
        menu_button(ui, app, "interfaceTheme", Some("Interface\nTheme"), "Interface Theme", true, "", |ui, app| {
            for a in crate::theme::Appearance::ALL {
                if ui
                    .steady_scope(app.ui.theme == a, |ui| {
                        ui.add(egui::Button::selectable(app.ui.theme == a, tl!(a.label())).min_size(vec2(200.0, 0.0)))
                    })
                    .clicked()
                {
                    let _ = app.run("ui.theme", json!({"value": a.code()}));
                    ui.close();
                }
            }
        });
    });
    group(ui, "Window", app, |ui, app| {
        big(ui, app, "newWindow", "New\nWindow", "view.newWindow", json!({}), false);
        big(ui, app, "arrangeAll", "Arrange\nAll", "view.arrangeAll", json!({}), false);
        big(ui, app, "split", "Split", "view.split", json!({}), false);
        big(ui, app, "fullScreen", "Full\nScreen", "view.fullScreen", json!({}), v.full_screen);
    });
    group(ui, "Templates", app, |ui, app| {
        big(ui, app, "documentTemplate", "Templates", "tools.templates", json!({}), false);
    });
}

/// The Outlining tab (Outline view): outline levels, moving, expanding and collapsing, what is
/// shown, and closing the view.
fn outlining(app: &mut WordApp, ui: &mut Ui) {
    use wordcraft_engine::layout::outline::{BODY, para_level};
    let o = app.session.view.outline.clone();
    let level = app.session.doc.para_at(&app.session.sel.focus).map(|p| para_level(&app.session.doc, p)).unwrap_or(BODY);
    let name = |l: u8| if l >= BODY { tl!("Body Text").to_string() } else { crate::i18n::fmt(tl!("Level {n}"), &[("n", &l.to_string())]) };
    group(ui, "Outline Tools", app, |ui, app| {
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "outlineTop", None, "Promote to Heading 1", "outline.promoteToHeading1", json!({}), false);
                small(ui, app, "outlinePromote", None, "Promote", "outline.promote", json!({}), false);
                let levels: Vec<String> = (1..=BODY).map(name).collect();
                if let Some(v) = combo(ui, "outlineLevel", 100.0, &name(level), &levels, None)
                    && let Some(i) = levels.iter().position(|x| *x == v)
                {
                    let l = if i + 1 >= usize::from(BODY) { json!("body") } else { json!(i + 1) };
                    let _ = app.run("outline.level", json!({ "level": l }));
                }
                small(ui, app, "outlineDemote", None, "Demote", "outline.demote", json!({}), false);
                small(ui, app, "outlineBody", None, "Demote to Body Text", "outline.demoteToBody", json!({}), false);
            });
            crate::widgets::row(ui, |ui| {
                small(ui, app, "outlineUp", None, "Move Up", "outline.moveUp", json!({}), false);
                small(ui, app, "outlineDown", None, "Move Down", "outline.moveDown", json!({}), false);
                small(ui, app, "outlineExpand", None, "Expand", "outline.expand", json!({}), false);
                small(ui, app, "outlineCollapse", None, "Collapse", "outline.collapse", json!({}), false);
                ui.add_space(6.0);
                ui.label(egui::RichText::new(tl!("Show Level")).size(11.5));
                let mut shown: Vec<String> = vec![tl!("All Levels").to_string()];
                shown.extend((1..BODY).map(name));
                let current = shown.get(usize::from(o.show_level)).cloned().unwrap_or_default();
                if let Some(v) = combo(ui, "outlineShowLevel", 90.0, &current, &shown, None)
                    && let Some(i) = shown.iter().position(|x| *x == v)
                {
                    let l = if i == 0 { json!("all") } else { json!(i) };
                    let _ = app.run("outline.showLevel", json!({ "level": l }));
                }
            });
            crate::widgets::row(ui, |ui| {
                let mut f = !o.hide_formatting;
                if ui.checkbox(&mut f, tl!("Show Text Formatting")).changed() {
                    let _ = app.run("outline.showFormatting", json!({ "value": f }));
                }
                let mut first = o.first_line_only;
                if ui.checkbox(&mut first, tl!("Show First Line Only")).changed() {
                    let _ = app.run("outline.firstLineOnly", json!({ "value": first }));
                }
            });
        });
    });
    group(ui, "Close", app, |ui, app| {
        big(ui, app, "closeOutline", "Close Outline\nView", "outline.close", json!({}), false);
    });
}

fn help(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Help", app, |ui, app| {
        big(ui, app, "help", "Help", "ui.dialog", json!({"name": "about"}), false);
        if app.services.latest_release.is_some() {
            big(ui, app, "checkUpdates", "Check for\nUpdates", "ui.checkForUpdates", json!({}), false);
        }
    });
    group(ui, "Agents", app, |ui, app| {
        big(ui, app, "macros", "Commands", "ui.dialog", json!({"name": "commands"}), false);
    });
}

/// SmartArt Design: the selected graphic's items (Add Shape, Promote, Demote, Text Pane), its
/// layout (a gallery of previews), its colours and Reset.
fn smart_art_design(app: &mut WordApp, ui: &mut Ui) {
    use wordcraft_doc::smart_art::{SmartArtColors, SmartArtLayout};
    let Some(g) = wordcraft_engine::cmd::smart_art::selected_smart_art(&app.session) else { return };
    let theme = app.session.doc.settings.theme_colors.clone();
    // The Text Pane's current item, else the last.
    let current = app.smart_art_item.filter(|i| *i < g.items.len());
    group(ui, "Create Graphic", app, |ui, app| {
        big(ui, app, "addShape", "Add\nShape", "smartArt.addShape", current.map_or(json!({}), |i| json!({"after": i})), false);
        stack(ui, |ui| {
            let idx = current.map_or(json!({}), |i| json!({"index": i}));
            crate::widgets::row(ui, |ui| {
                small(ui, app, "promote", Some("Promote"), "Promote", "smartArt.promote", idx.clone(), false);
                small(ui, app, "demote", Some("Demote"), "Demote", "smartArt.demote", idx, false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "textPane", Some("Text Pane"), "Text Pane", "smartArt.textPane", json!({}), app.session.view.smart_art_pane);
            });
        });
    });
    group(ui, "Layouts", app, |ui, app| {
        let t = Tokens::get(ui.ctx());
        egui::Grid::new("smart_art_layout_gallery").spacing(vec2(2.0, 2.0)).show(ui, |ui| {
            for (k, layout) in SmartArtLayout::ALL.into_iter().enumerate() {
                let (r, resp) = ui.allocate_exact_size(vec2(46.0, 28.0), Sense::click());
                let on = g.layout == layout;
                let fill = if on {
                    t.checked
                } else if resp.hovered() {
                    t.hover
                } else {
                    t.ribbon
                };
                ui.painter().rect(r, 3.0, fill, Stroke::new(1.0, if on { t.accent } else { t.border }), egui::StrokeKind::Inside);
                crate::dialogs_smart_art::preview(ui.painter(), r.shrink(2.0), layout, g.colors, &theme, t.text_dim);
                if resp.on_hover_text(tl!(layout.label())).clicked() && !on {
                    let _ = app.run("smartArt.layout", json!({"layout": layout.id()}));
                }
                if k % 4 == 3 {
                    ui.end_row();
                }
            }
        });
    });
    group(ui, "SmartArt Styles", app, |ui, app| {
        menu_button(ui, app, "smartArtColors", Some("Change\nColors"), "Change Colors", true, "", |ui, app| {
            for colors in SmartArtColors::ALL {
                let on = g.colors == colors;
                let resp = ui.add(egui::Button::new(crate::dialogs_smart_art::colors_label(colors)).selected(on).min_size(vec2(200.0, 24.0)));
                let sw = Rect::from_min_size(pos2(resp.rect.right() - 74.0, resp.rect.center().y - 6.0), vec2(66.0, 12.0));
                crate::dialogs_smart_art::swatches(ui.painter(), sw, colors, &theme);
                if resp.clicked() {
                    let _ = app.run("smartArt.colors", json!({"variant": colors.id()}));
                    ui.close();
                }
            }
        });
    });
    group(ui, "Reset", app, |ui, app| {
        big(ui, app, "resetGraphic", "Reset\nGraphic", "smartArt.reset", json!({}), false);
    });
}

/// Chart Design: the selected chart's elements (title, legend, data labels), its data and type,
/// and its size.
fn chart_design(app: &mut WordApp, ui: &mut Ui) {
    let Some(chart) = wordcraft_engine::cmd::chart::selected_chart(&app.session) else { return };
    group(ui, "Chart Layouts", app, |ui, app| {
        menu_button(ui, app, "chartTitle", Some("Chart Title"), "Chart Title", true, "", |ui, app| {
            mi_check(ui, app, "None", chart.title.is_none(), "chart.title", json!({"text": null}));
            let text = chart.title.clone().unwrap_or_else(|| tl!("Chart Title").to_string());
            mi_check(ui, app, "Above Chart", chart.title.is_some(), "chart.title", json!({"text": text}));
        });
        menu_button(ui, app, "legend", Some("Legend"), "Legend", true, "", |ui, app| {
            mi_check(ui, app, "None", chart.legend.is_none(), "chart.legend", json!({"pos": "none"}));
            for (label, pos) in [
                ("Right", wordcraft_doc::chart::LegendPos::Right),
                ("Top", wordcraft_doc::chart::LegendPos::Top),
                ("Left", wordcraft_doc::chart::LegendPos::Left),
                ("Bottom", wordcraft_doc::chart::LegendPos::Bottom),
            ] {
                mi_check(ui, app, label, chart.legend == Some(pos), "chart.legend", json!({"pos": pos.id()}));
            }
        });
        menu_button(ui, app, "dataLabels", Some("Data Labels"), "Data Labels", true, "", |ui, app| {
            mi_check(ui, app, "None", !chart.data_labels, "chart.dataLabels", json!({"on": false}));
            mi_check(ui, app, "Show", chart.data_labels, "chart.dataLabels", json!({"on": true}));
        });
    });
    group(ui, "Data", app, |ui, app| {
        big(ui, app, "chartData", "Edit\nData", "chart.editData", json!({}), false);
    });
    group(ui, "Type", app, |ui, app| {
        big(ui, app, "changeChartType", "Change\nChart Type", "chart.type", json!({}), false);
    });
    group(ui, "Size", app, |ui, app| {
        let (w0, h0) =
            wordcraft_engine::cmd::objects::selected(&app.session).and_then(|(_, o)| o.frame().map(|(w, h, _)| (w, h))).unwrap_or((0.0, 0.0));
        stack(ui, |ui| {
            for (label, key, v0) in [("H:", "height", h0), ("W:", "width", w0)] {
                crate::widgets::row(ui, |ui| {
                    ui.label(egui::RichText::new(tl!(label)).small());
                    let (r, v) = size_box(ui, v0, 36.0, 2000.0);
                    if r.changed() {
                        if r.dragged() && !r.drag_started() {
                            app.session.join_next_undo();
                        }
                        let _ = app.run("picture.size", json!({key: v, "lockAspect": false}));
                    }
                });
            }
        });
    });
}

/// A width or height box for a picture, shape or chart: `pt` (points) shown and typed in the unit of
/// File › Options › Advanced (inches, centimetres…; `5 cm` typed in inches converts), as the table
/// boxes are. Returns the box and the value in points.
fn size_box(ui: &mut Ui, pt: f32, min_pt: f32, max_pt: f32) -> (egui::Response, f32) {
    let unit = crate::options::unit();
    let k = unit.pt_per_unit();
    let mut v = pt / k;
    let r = ui.add(crate::numfield::NumField::new(&mut v).speed(1.0 / k).range(min_pt / k..=max_pt / k).measure(unit).max_decimals(2));
    (r, v * k)
}

/// Shape Format: fill, outline and effects of the selected shape, and its arrangement.
fn shape_format(app: &mut WordApp, ui: &mut Ui) {
    let (stroke, glow_size) = match wordcraft_engine::cmd::objects::selected(&app.session) {
        Some((_, wordcraft_doc::para::InlineObject::Shape { stroke, effects, .. })) => (stroke, effects.glow.map(|g| g.size)),
        _ => (None, None),
    };
    let theme = app.session.doc.settings.theme_colors.clone();
    group(ui, "Shape Styles", app, |ui, app| {
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "shading", Some("Shape Fill"), "Shape Fill", false, "shape.fill", |ui, app| {
                    mi(ui, app, "No Color", "shape.fill", json!({"color": null}));
                    crate::colors::menu(ui, app, &theme, Target::cmd("shape.fill"));
                });
                menu_button(ui, app, "pen", Some("Shape Outline"), "Shape Outline", false, "shape.outline", |ui, app| {
                    mi(ui, app, "No Color", "shape.outline", json!({"color": null}));
                    crate::colors::menu(ui, app, &theme, Target::cmd("shape.outline"));
                    ui.separator();
                    ui.menu_button(tl!("Weight"), |ui| {
                        let color = stroke.unwrap_or(wordcraft_doc::Rgb::BLACK).hex();
                        for w in [0.25, 0.5, 0.75, 1.0, 1.5, 2.25, 3.0, 4.5, 6.0] {
                            mi(ui, app, &format!("{w} {}", tl!("pt")), "shape.outline", json!({"color": color, "width": w}));
                        }
                    });
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "shapeEffects", Some("Shape Effects"), "Shadow, glow and soft edges", false, "shape.effects", |ui, app| {
                    shape_effects_menu(ui, app, &theme, glow_size)
                });
            });
        });
    });
    alt_text_group(ui, app);
    if wordcraft_engine::cmd::canvas::selected_canvas(&app.session).is_some() {
        group(ui, "Drawing Canvas", app, |ui, app| {
            menu_button(ui, app, "canvas", Some("Drawing\nCanvas"), "Drawing Canvas", true, "", |ui, app| {
                mi(ui, app, "Fit", "canvas.fit", json!({}));
                mi(ui, app, "Expand", "canvas.expand", json!({}));
                ui.menu_button(tl!("Scale Drawing"), |ui| {
                    for pct in [50, 75, 125, 150, 200] {
                        mi(ui, app, &format!("{pct}%"), "canvas.scale", json!({"factor": pct as f32 / 100.0}));
                    }
                });
            });
        });
    }
    group(ui, "WordArt Styles", app, |ui, app| {
        menu_button(ui, app, "wordArt", Some("Styles"), "WordArt Styles", true, "", |ui, app| {
            crate::wordart_menus::style_gallery(ui, app, "wordArt.style")
        });
        stack(ui, |ui| {
            let outline = crate::wordart_menus::current_outline(app);
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "textFill", Some("Text Fill"), "Text Fill", false, "", |ui, app| {
                    crate::wordart_menus::fill_menu(ui, app, &theme)
                });
                menu_button(ui, app, "textOutline", Some("Text Outline"), "Text Outline", false, "", |ui, app| {
                    crate::wordart_menus::outline_menu(ui, app, &theme, outline)
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "transform", Some("Text Effects"), "Text Effects", false, "", |ui, app| {
                    crate::wordart_menus::effects_menu(ui, app, &theme, true)
                });
            });
        });
    });
    shape_text_group(app, ui);
    group(ui, "Arrange", app, |ui, app| {
        big(ui, app, "position", "Position", "arrange.position", json!({}), false);
        wrap_menu(ui, app);
        stack(ui, |ui| {
            rotate_menu(ui, app, Some("Rotate"));
            small(ui, app, "align", Some("Align"), "Align", "arrange.align", json!({}), false);
        });
    });
    group(ui, "Size", app, |ui, app| {
        let (w0, h0) =
            wordcraft_engine::cmd::objects::selected(&app.session).and_then(|(_, o)| o.frame().map(|(w, h, _)| (w, h))).unwrap_or((0.0, 0.0));
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                for (label, key, v0) in [("W:", "width", w0), ("H:", "height", h0)] {
                    ui.label(egui::RichText::new(tl!(label)).small());
                    let (r, v) = size_box(ui, v0, 4.0, 2000.0);
                    if r.changed() {
                        if r.dragged() && !r.drag_started() {
                            app.session.join_next_undo();
                        }
                        let _ = app.run("picture.size", json!({key: v, "lockAspect": false}));
                    }
                }
            });
            ui.add_space(2.0);
            crate::widgets::row(ui, |ui| rotation_field(ui, app));
        });
    });
}

/// Shape Format › Text: the text box's Text Direction and Align Text, and Create Link (Break Link
/// once it is linked to another box).
fn shape_text_group(app: &mut WordApp, ui: &mut Ui) {
    use wordcraft_doc::props::{TextVert, VAlign};
    let target = wordcraft_engine::cmd::objects::text_box_target(&app.session);
    let body = target.and_then(|(_, id)| app.session.doc.parts.get(&id).map(|p| p.body)).unwrap_or_default();
    group(ui, "Text", app, |ui, app| {
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "textDirection", Some("Text Direction"), "Text Direction", false, "", |ui, app| {
                    for (label, vert) in [
                        ("Horizontal", TextVert::Horz),
                        ("Rotate all text 90°", TextVert::Vert),
                        ("Rotate all text 270°", TextVert::Vert270),
                        ("Stacked", TextVert::Stacked),
                    ] {
                        mi_check(ui, app, label, body.vert == vert, "shape.textDirection", json!({"dir": vert.ooxml()}));
                    }
                });
                menu_button(ui, app, "alignText", Some("Align Text"), "Align Text", false, "", |ui, app| {
                    for (label, anchor) in [("Top", VAlign::Top), ("Middle", VAlign::Center), ("Bottom", VAlign::Bottom)] {
                        mi_check(
                            ui,
                            app,
                            label,
                            body.anchor == anchor,
                            "shape.alignText",
                            json!({"anchor": wordcraft_doc::props::anchor_ooxml(anchor)}),
                        );
                    }
                });
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                if body.next.is_some() {
                    small(ui, app, "textBoxUnlink", Some("Break Link"), "Break Link", "shape.breakLink", json!({}), false);
                } else {
                    let waiting = app.canvas.link_from.is_some();
                    small(ui, app, "textBoxLink", Some("Create Link"), "Create Link", "ui.linkTextBox", json!({}), waiting);
                }
            });
        });
    });
}

/// Picture/Shape Format › Accessibility: Alt Text opens the Alt Text pane.
fn alt_text_group(ui: &mut Ui, app: &mut WordApp) {
    group(ui, "Accessibility", app, |ui, app| {
        big(ui, app, "altText", "Alt\nText", "view.altTextPane", json!({"value": true}), false);
    });
}

/// Shape Format › Shape Effects: Shadow, Glow and Soft Edges, each with presets and a "No …" entry.
fn shape_effects_menu(ui: &mut Ui, app: &mut WordApp, theme: &[wordcraft_doc::Rgb], glow_size: Option<f32>) {
    ui.menu_button(tl!("Shadow"), |ui| {
        mi(ui, app, "No Shadow", "shape.effects", json!({"shadow": null}));
        ui.separator();
        for (id, label) in wordcraft_doc::effects::SHADOW_PRESETS {
            mi(ui, app, label, "shape.effects", json!({"shadow": id}));
        }
    });
    ui.menu_button(tl!("Glow"), |ui| {
        mi(ui, app, "No Glow", "shape.effects", json!({"glow": null}));
        ui.separator();
        for size in [5.0, 8.0, 11.0, 18.0] {
            mi(ui, app, &format!("{size} {}", tl!("pt")), "shape.effects", json!({"glow": size}));
        }
        ui.separator();
        ui.menu_button(tl!("Glow Colors"), |ui| {
            let target = Target::at("shape.effects", json!({"glow": {"color": null, "size": glow_size.unwrap_or(8.0)}}), "/glow/color");
            crate::colors::menu(ui, app, theme, target);
        });
    });
    ui.menu_button(tl!("Soft Edges"), |ui| {
        mi(ui, app, "No Soft Edges", "shape.effects", json!({"softEdge": null}));
        ui.separator();
        for r in [1.0, 2.5, 5.0, 10.0, 25.0, 50.0] {
            mi(ui, app, &format!("{r} {}", tl!("pt")), "shape.effects", json!({"softEdge": r}));
        }
    });
}

fn picture_format(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Adjust", app, |ui, app| {
        menu_button(ui, app, "picCorrections", Some("Corrections"), "Brightness, contrast and sharpness", true, "picture.corrections", |ui, app| {
            for (l, b, c) in
                [("Brighter +20%", 20.0, 0.0), ("Darker −20%", -20.0, 0.0), ("More Contrast +20%", 0.0, 20.0), ("Less Contrast −20%", 0.0, -20.0)]
            {
                mi(ui, app, l, "picture.corrections", json!({"brightness": b, "contrast": c}));
            }
            ui.separator();
            mi(ui, app, "Sharpen", "picture.corrections", json!({"sharpen": 40.0}));
            mi(ui, app, "Soften", "picture.corrections", json!({"sharpen": -40.0}));
        });
        menu_button(ui, app, "picColor", Some("Color"), "Saturation and recolor", true, "picture.color", |ui, app| {
            for (l, v) in [("Full Saturation 100%", 100.0), ("Muted 50%", 50.0), ("Gray 0%", 0.0), ("Vivid 200%", 200.0)] {
                mi(ui, app, l, "picture.color", json!({"mode": "saturation", "saturation": v}));
            }
            ui.separator();
            mi(ui, app, "Grayscale", "picture.color", json!({"mode": "grayscale"}));
            mi(ui, app, "Sepia", "picture.color", json!({"mode": "sepia"}));
            mi(ui, app, "Washout", "picture.color", json!({"mode": "washout"}));
        });
        menu_button(ui, app, "icons", Some("Icon Color"), "Recolor a one-color picture, such as an icon", true, "picture.iconColor", |ui, app| {
            let theme = app.session.doc.settings.theme_colors.clone();
            crate::colors::menu(ui, app, &theme, Target::cmd("picture.iconColor"));
        });
        menu_button(ui, app, "picTransparency", Some("Transparency"), "Picture transparency", true, "picture.transparency", |ui, app| {
            for v in [0.0, 15.0, 30.0, 50.0, 65.0, 80.0, 95.0] {
                mi(ui, app, &format!("{v:.0}%"), "picture.transparency", json!({"percent": v}));
            }
            ui.separator();
            // An exact value, dragged with a live preview: absolute from the original
            // pixels, one undo step for the whole drag (same pattern as Size › W/H).
            // The current value isn't stored anywhere, so the slider reopens where it
            // was last left for this picture.
            let (media, off) = match wordcraft_engine::cmd::objects::selected(&app.session) {
                Some((pos, wordcraft_doc::para::InlineObject::Image { media, .. })) => (media, pos.off),
                _ => (String::new(), 0),
            };
            let key = egui::Id::new(("picture_transparency", media, off));
            let mut v = ui.data(|d| d.get_temp::<f64>(key)).unwrap_or(0.0);
            let r = ui.add(crate::numfield::NumField::new(&mut v).speed(1.0).range(0.0..=100.0).suffix("%"));
            if r.changed() {
                ui.data_mut(|d| d.insert_temp(key, v));
                if r.dragged() && !r.drag_started() {
                    app.session.join_next_undo();
                }
                let _ = app.run("picture.transparency", json!({"percent": v}));
            }
        });
        stack(ui, |ui| {
            small(ui, app, "picChange", Some("Change"), "Change Picture", "ui.changePicture", json!({}), false);
            small(ui, app, "picReset", Some("Reset"), "Reset Picture", "picture.reset", json!({}), false);
        });
    });
    group(ui, "Picture Styles", app, |ui, app| {
        menu_button(ui, app, "picStyles", Some("Styles"), "Frames and effects", true, "picture.style", |ui, app| {
            for (l, s) in [
                ("Simple Frame", "simpleFrame"),
                ("Thick Frame", "thickFrame"),
                ("Rounded", "rounded"),
                ("Soft Edge", "softEdge"),
                ("Shadow", "shadow"),
            ] {
                mi(ui, app, l, "picture.style", json!({"style": s}));
            }
        });
        menu_button(ui, app, "picBorder", Some("Border"), "Picture border", true, "picture.border", |ui, app| {
            for (l, w) in [("Thin", 2), ("Medium", 4), ("Thick", 8)] {
                mi(ui, app, l, "picture.border", json!({"width": w}));
            }
        });
    });
    alt_text_group(ui, app);
    group(ui, "Arrange", app, |ui, app| {
        big(ui, app, "position", "Position", "arrange.position", json!({}), false);
        wrap_menu(ui, app);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                rotate_menu(ui, app, Some("Rotate"));
                small(ui, app, "align", Some("Align"), "Align", "arrange.align", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                group_menu(ui, app, Some("Group"));
            });
        });
    });
    group(ui, "Size", app, |ui, app| {
        let (w0, h0, crop0) = match wordcraft_engine::cmd::objects::selected(&app.session) {
            Some((_, wordcraft_doc::para::InlineObject::Image { w, h, crop, .. })) => (w, h, crop),
            _ => (0.0, 0.0, [0.0; 4]),
        };
        // `picture.size` locks the aspect ratio unless told otherwise, so one dimension suffices.
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                ui.label(egui::RichText::new(tl!("W:")).small());
                let (r, w) = size_box(ui, w0, 4.0, 2000.0);
                if r.changed() {
                    if r.dragged() && !r.drag_started() {
                        app.session.join_next_undo();
                    }
                    let _ = app.run("picture.size", json!({"width": w}));
                }
                ui.label(egui::RichText::new(tl!("H:")).small());
                let (r, h) = size_box(ui, h0, 4.0, 2000.0);
                if r.changed() {
                    if r.dragged() && !r.drag_started() {
                        app.session.join_next_undo();
                    }
                    let _ = app.run("picture.size", json!({"height": h}));
                }
                rotation_field(ui, app);
            });
            ui.add_space(2.0);
            crate::widgets::row(ui, |ui| {
                let cropping = crate::crop::active(app).is_some();
                small(ui, app, "crop", Some("Crop"), "Crop", "ui.crop", json!({}), cropping);
                for (i, side) in ["L", "T", "R", "B"].iter().enumerate() {
                    ui.label(egui::RichText::new(crate::i18n::tc("crop side", side)).small());
                    let mut v = crop0[i] * 100.0;
                    let r = ui.add(crate::numfield::NumField::new(&mut v).speed(0.5).range(0.0..=45.0).suffix("%"));
                    if r.changed() {
                        if r.dragged() && !r.drag_started() {
                            app.session.join_next_undo();
                        }
                        let mut c = crop0;
                        c[i] = (v / 100.0).clamp(0.0, 0.45);
                        let _ = app.run("picture.crop", json!({"left": c[0], "top": c[1], "right": c[2], "bottom": c[3]}));
                    }
                }
                if ui.button(egui::RichText::new(tl!("Reset")).small()).on_hover_text(tl!("Reset Crop")).clicked() {
                    let _ = app.run("picture.crop", json!({"left": 0.0, "top": 0.0, "right": 0.0, "bottom": 0.0}));
                }
            });
        });
    });
}

fn table_design(app: &mut WordApp, ui: &mut Ui) {
    let look = app.session.sel.focus.path.cell().and_then(|(tp, _, _)| app.session.doc.table(app.session.sel.focus.story, &tp).map(|t| t.props.look));
    group(ui, "Table Style Options", app, |ui, app| {
        let Some(l) = look else { return };
        // Two rows (the two-size ribbon's): the rows' options, then the columns'.
        egui::Grid::new("look").spacing(vec2(8.0, crate::widgets::ROW_GAP + 8.0)).show(ui, |ui| {
            for (row, items) in [
                [("Header Row", "headerRow", l.header_row), ("Total Row", "totalRow", l.total_row), ("Banded Rows", "bandedRows", l.banded_rows)],
                [
                    ("First Column", "firstColumn", l.first_column),
                    ("Last Column", "lastColumn", l.last_column),
                    ("Banded Columns", "bandedColumns", l.banded_columns),
                ],
            ]
            .iter()
            .enumerate()
            {
                let _ = row;
                for (label, key, val) in items {
                    let mut v = *val;
                    if ui.checkbox(&mut v, tl!(label)).changed() {
                        let _ = app.run("table.look", json!({ *key: v }));
                    }
                }
                ui.end_row();
            }
        });
    });
    group(ui, "Table Styles", app, |ui, app| {
        // The document's own styles first, so a new one shows without scrolling.
        let mut styles: Vec<(bool, String, String)> = app
            .session
            .doc
            .styles
            .styles
            .iter()
            .filter(|s| s.kind == wordcraft_doc::StyleKind::Table && !s.hidden)
            .map(|s| (s.builtin, s.id.clone(), crate::i18n::style_name(s)))
            .collect();
        styles.sort_by_key(|(builtin, _, _)| *builtin);
        let current_id = look.and_then(|_| {
            let (tp, _, _) = app.session.sel.focus.path.cell()?;
            app.session.doc.table(app.session.sel.focus.story, &tp)?.props.style.clone()
        });
        let current = current_id.as_ref().and_then(|id| {
            app.session
                .doc
                .styles
                .get(id)
                .filter(|st| st.kind == wordcraft_doc::StyleKind::Table)
                .map(wordcraft_engine::cmd::table_style::is_builtin_table_style)
        });
        table_style_gallery(ui, app, &styles, current_id.as_deref(), current);
        stack(ui, |ui| {
            let swatch = crate::colors::parse(&app.canvas.last_shading).map(|c| egui::Color32::from_rgb(c.0, c.1, c.2));
            let params = json!({"color": app.canvas.last_shading.clone()});
            split_labeled(ui, app, "shading", Some("Shading"), "Shading", "table.shading", params, false, swatch, |ui, app| {
                mi(ui, app, "No Color", "table.shading", json!({"color": null}));
                let theme = app.session.doc.settings.theme_colors.clone();
                crate::colors::menu(ui, app, &theme, Target::cmd("table.shading").remember(Remember::Shading));
            });
        });
    });
    group(ui, "Borders", app, |ui, app| {
        menu_button(ui, app, "borders", Some("Borders"), "Borders", true, "table.borders", border_items);
        big(ui, app, "borderPainter", "Border\nPainter", "table.borderPainter", json!({}), false);
    });
}

/// The Table Design gallery: a few tiles with arrows to page through them and "more", which
/// opens every table style in a panel grouped as Plain, Grid and List tables (the document's own
/// first), with New, Modify, Delete and Clear below. `current` is the caret table's style id and
/// `builtin` whether that style is one of ours (`None`: not in a table).
fn table_style_gallery(ui: &mut Ui, app: &mut WordApp, styles: &[(bool, String, String)], current_id: Option<&str>, builtin: Option<bool>) {
    let t = Tokens::get(ui.ctx());
    let tile = vec2(62.0, 52.0);
    // The panel's tiles are larger, like Word's, so a style's bands and borders read clearly.
    let panel_tile = vec2(92.0, 68.0);
    let visible = 5usize;
    let pitch = tile.x + 3.0;
    let key = egui::Id::new("table_gallery_start");
    let start = ui.data(|d| d.get_temp::<usize>(key)).unwrap_or(0).min(styles.len().saturating_sub(1));
    let (area, _) = ui.allocate_exact_size(vec2(visible as f32 * pitch + 22.0, CONTENT_H), Sense::hover());
    ui.painter().rect(area.shrink(1.0), 4.0, t.input, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    for (k, (_, id, name)) in styles.iter().skip(start).take(visible).enumerate() {
        let r = Rect::from_min_size(pos2(area.min.x + 3.0 + k as f32 * pitch, area.center().y - tile.y / 2.0), tile);
        if crate::previews::table_style_tile(ui, app, id, name, r, current_id == Some(id.as_str())).clicked() {
            let _ = app.run("table.style", json!({"style": id}));
        }
    }
    // Scroll arrows and "more".
    let ar = Rect::from_min_max(pos2(area.max.x - 17.0, area.min.y + 2.0), pos2(area.max.x - 2.0, area.max.y - 2.0));
    let thirds = ar.height() / 3.0;
    let mut more = None;
    for (i, (icon, delta)) in [("chevronUp", -1i32), ("chevronDown", 1), ("more", 0)].iter().enumerate() {
        let br = Rect::from_min_size(pos2(ar.min.x, ar.min.y + thirds * i as f32), vec2(ar.width(), thirds));
        let resp = ui.interact(br, ui.id().with(("table_gal_arrow", i)), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(br, 2.0, t.hover);
        }
        crate::icons::paint(ui.painter(), br.shrink(2.0), icon, t.icon, t.accent);
        let tip = match delta {
            0 => "More",
            d if *d < 0 => "Previous",
            _ => "Next",
        };
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!(tip)));
        if *delta == 0 {
            more = Some(resp);
        } else if resp.clicked() {
            let n = if *delta < 0 { start.saturating_sub(visible) } else { (start + visible).min(styles.len().saturating_sub(1)) };
            ui.data_mut(|d| d.insert_temp(key, n));
        }
    }
    let Some(more) = more else { return };
    let can_modify = builtin.is_some();
    // Built-in table styles can't be deleted.
    let can_delete = builtin == Some(false);
    egui::Popup::menu(&more).show(|ui| {
        let cols = 5usize;
        let gap = 4.0;
        ui.set_width(cols as f32 * (panel_tile.x + gap) + 8.0);
        let group_of = |builtin: bool, id: &str| match (builtin, id) {
            (false, _) => 0,
            (true, i) if i.starts_with("GridTable") => 2,
            (true, i) if i.starts_with("ListTable") => 3,
            _ => 1,
        };
        egui::ScrollArea::vertical().max_height(380.0).auto_shrink([false, true]).id_salt("table_styles_panel").show(ui, |ui| {
            for (g, title) in [(0, "Custom"), (1, "Plain Tables"), (2, "Grid Tables"), (3, "List Tables")] {
                let members: Vec<&(bool, String, String)> = styles.iter().filter(|(b, id, _)| group_of(*b, id) == g).collect();
                if members.is_empty() {
                    continue;
                }
                ui.label(egui::RichText::new(tl!(title)).font(crate::theme::semibold(12.0)));
                ui.add_space(2.0);
                for row in members.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(gap, gap);
                        for (_, id, name) in row {
                            let (r, _) = ui.allocate_exact_size(panel_tile, Sense::hover());
                            if crate::previews::table_style_tile(ui, app, id, name, r, current_id == Some(id.as_str())).clicked() {
                                let _ = app.run("table.style", json!({"style": id}));
                                ui.close();
                            }
                        }
                    });
                }
                ui.add_space(4.0);
            }
        });
        ui.separator();
        if ui.add_enabled(can_modify, egui::Button::new(tl!("Modify Table Style…")).min_size(vec2(200.0, 0.0))).clicked() {
            let _ = app.run("ui.dialog", json!({"name": "modifyTableStyle"}));
            ui.close();
        }
        if ui.add_enabled(can_delete, egui::Button::new(tl!("Delete Table Style")).min_size(vec2(200.0, 0.0))).clicked() {
            if let Err(e) = app.run("table.deleteStyle", json!({})) {
                app.status(e);
            }
            ui.close();
        }
        if ui.add_enabled(can_modify, egui::Button::new(tl!("Clear")).min_size(vec2(200.0, 0.0))).clicked() {
            if let Err(e) = app.run("table.style", json!({"style": "TableNormal"})) {
                app.status(e);
            }
            ui.close();
        }
        if ui.add(egui::Button::new(tl!("New Table Style…")).min_size(vec2(200.0, 0.0))).clicked() {
            let _ = app.run("ui.dialog", json!({"name": "newTableStyle"}));
            ui.close();
        }
    });
}

/// Table Layout › Cell Size: the caret cell's row height and column width, in inches. They show
/// the size on the page (a row without a set height shows the height its text gives it) and
/// editing runs `table.rowHeight` / `table.columnWidth`; dragging one is a single Undo.
fn cell_size_boxes(ui: &mut Ui, app: &mut WordApp) {
    let story = app.session.sel.focus.story;
    let Some((tp, r, c)) = app.session.sel.focus.path.cell() else { return };
    let Some(t) = app.session.doc.table(story, &tp) else { return };
    let row = t.rows.get(r).map(|x| x.props.clone()).unwrap_or_default();
    let stored_w = t.grid.get(t.grid_col(r, c)).copied();
    let laid = app.session.layout().pages.iter().flat_map(|pg| pg.items.iter()).find_map(|it| match it {
        wordcraft_layout::Placed::Cell { rect, table, row, cell, story: st, .. } if *table == tp && *row == r && *cell == c && *st == story => {
            Some(*rect)
        }
        _ => None,
    });
    // The row's own height: a vertically merged cell's box spans several rows, so take the
    // shortest box in this row.
    let row_h = app
        .session
        .layout()
        .pages
        .iter()
        .flat_map(|pg| pg.items.iter())
        .filter_map(|it| match it {
            wordcraft_layout::Placed::Cell { rect, table, row, story: st, .. } if *table == tp && *row == r && *st == story => Some(rect.h),
            _ => None,
        })
        .reduce(f32::min);
    let unit = crate::options::unit();
    let k = unit.pt_per_unit();
    let h0 = row.height.or(row_h).unwrap_or(0.0);
    let w0 = laid.map(|x| x.w).or(stored_w).unwrap_or(0.0);
    let exact = row.height_rule == wordcraft_doc::props::HeightRule::Exact;
    // The edited value in points, and whether it continues a drag (joins the previous Undo step).
    let boxed = |ui: &mut Ui, label: &str, pt: f32| -> Option<(f32, bool)> {
        let mut out = None;
        crate::widgets::row(ui, |ui| {
            ui.add_sized(vec2(48.0, 18.0), egui::Label::new(egui::RichText::new(tl!(label)).small()));
            let mut v = pt / k;
            let r =
                ui.add_sized(vec2(72.0, 18.0), crate::numfield::NumField::new(&mut v).speed(0.01).range(0.02..=22.0).measure(unit).max_decimals(2));
            if r.changed() {
                out = Some((v * k, r.dragged() && !r.drag_started()));
            }
        });
        ui.add_space(2.0);
        out
    };
    stack(ui, |ui| {
        if let Some((h, join)) = boxed(ui, "Height:", h0) {
            if join {
                app.session.join_next_undo();
            }
            let _ = app.run("table.rowHeight", json!({"height": h, "rule": if exact { "exact" } else { "atLeast" }}));
        }
        if let Some((w, join)) = boxed(ui, "Width:", w0) {
            if join {
                app.session.join_next_undo();
            }
            let _ = app.run("table.columnWidth", json!({"width": w}));
        }
    });
}

fn table_layout(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Table", app, |ui, app| {
        stack(ui, |ui| {
            let g = app.session.view.table_gridlines;
            crate::widgets::row(ui, |ui| {
                menu_button(ui, app, "selectTable", Some("Select"), "Select", false, "table.selectTable", |ui, app| {
                    pic_item(ui, app, "Select Cell", None, "sel_cell", "table.selectCell", json!({}));
                    pic_item(ui, app, "Select Column", None, "sel_column", "table.selectColumn", json!({}));
                    pic_item(ui, app, "Select Row", None, "sel_row", "table.selectRow", json!({}));
                    pic_item(ui, app, "Select Table", None, "sel_table", "table.selectTable", json!({}));
                });
                small(ui, app, "gridlines", Some("View Gridlines"), "View Gridlines", "table.viewGridlines", json!({}), g);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "properties", Some("Properties"), "Table Properties", "table.properties", json!({}), false);
            });
        });
    });
    group(ui, "Draw", app, |ui, app| {
        use crate::table_pen::TableTool;
        let tool = app.canvas.table_tool;
        crate::widgets::big_toggle(ui, app, "drawTable", "Draw\nTable", "table.draw", tool == Some(TableTool::Draw));
        crate::widgets::big_toggle(ui, app, "eraser", "Eraser", "table.eraser", tool == Some(TableTool::Erase));
    });
    group(ui, "Rows & Columns", app, |ui, app| {
        menu_button(ui, app, "deleteTable", Some("Delete"), "Delete", true, "table.deleteTable", |ui, app| {
            pic_item(ui, app, "Delete Cells", None, "del_cell", "table.deleteCells", json!({}));
            pic_item(ui, app, "Delete Columns", None, "del_column", "table.deleteColumn", json!({}));
            pic_item(ui, app, "Delete Rows", None, "del_row", "table.deleteRow", json!({}));
            pic_item(ui, app, "Delete Table", None, "del_table", "table.deleteTable", json!({}));
        });
        big(ui, app, "insertAbove", "Insert\nAbove", "table.insertRowAbove", json!({}), false);
        big(ui, app, "insertBelow", "Insert\nBelow", "table.insertRowBelow", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "insertLeft", None, "Insert Left", "table.insertColumnLeft", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "insertRight", None, "Insert Right", "table.insertColumnRight", json!({}), false);
            });
        });
    });
    group(ui, "Merge", app, |ui, app| {
        big(ui, app, "merge", "Merge\nCells", "table.merge", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "splitCells", None, "Split Cells", "table.split", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "splitTable", None, "Split Table", "table.splitTable", json!({}), false);
            });
        });
    });
    group(ui, "Cell Size", app, |ui, app| {
        menu_button(ui, app, "autofit", Some("AutoFit"), "AutoFit", true, "table.autofit", |ui, app| {
            pic_item(ui, app, "AutoFit Contents", None, "fit_contents", "table.autofit", json!({"mode": "contents"}));
            pic_item(ui, app, "AutoFit Window", None, "fit_window", "table.autofit", json!({"mode": "window"}));
            pic_item(ui, app, "Fixed Column Width", None, "fit_fixed", "table.autofit", json!({"mode": "fixed"}));
        });
        cell_size_boxes(ui, app);
        stack(ui, |ui| {
            small(ui, app, "distributeRows", Some("Distribute Rows"), "Distribute Rows", "table.distributeRows", json!({}), false);
            small(ui, app, "distributeCols", Some("Distribute Columns"), "Distribute Columns", "table.distributeColumns", json!({}), false);
        });
    });
    group(ui, "Alignment", app, |ui, app| {
        // The nine cell alignments, three rows in Word, are a menu of the two-size ribbon.
        menu_button(ui, app, "cellAlign_center", Some("Alignment"), "Cell Alignment", true, "table.cellAlign", |ui, app| {
            egui::Grid::new("cellalign").spacing(vec2(1.0, 1.0)).show(ui, |ui| {
                for row in [
                    [("topLeft", "Align Top Left"), ("topCenter", "Align Top Center"), ("topRight", "Align Top Right")],
                    [("centerLeft", "Align Center Left"), ("center", "Align Center"), ("centerRight", "Align Center Right")],
                    [("bottomLeft", "Align Bottom Left"), ("bottomCenter", "Align Bottom Center"), ("bottomRight", "Align Bottom Right")],
                ] {
                    for (v, tip) in row {
                        if small(ui, app, &format!("cellAlign_{v}"), None, tip, "table.cellAlign", json!({"value": v}), false).clicked() {
                            ui.close();
                        }
                    }
                    ui.end_row();
                }
            });
        });
        big(ui, app, "textDirection", "Text\nDirection", "table.textDirection", json!({}), false);
        big(ui, app, "cellMargins", "Cell\nMargins", "table.cellMargins", json!({}), false);
    });
    group(ui, "Data", app, |ui, app| {
        big(ui, app, "sort", "Sort", "table.sort", json!({}), false);
        stack(ui, |ui| {
            crate::widgets::row(ui, |ui| {
                small(ui, app, "repeatHeader", Some("Repeat Header Rows"), "Repeat Header Rows", "table.repeatHeader", json!({}), false);
                small(ui, app, "toText", Some("Convert to Text"), "Convert to Text", "table.toText", json!({}), false);
            });
            ui.add_space(crate::widgets::ROW_GAP);
            crate::widgets::row(ui, |ui| {
                small(ui, app, "formula", Some("Formula"), "Formula", "table.formula", json!({}), false);
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_engine::Session;

    fn png_data() -> String {
        let img = image::RgbaImage::from_fn(20, 10, |x, y| {
            if x > 2 && x < 17 && y > 2 && y < 7 { image::Rgba([200, 30, 30, 255]) } else { image::Rgba([255, 255, 255, 255]) }
        });
        let mut b = Vec::new();
        image::DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut b), image::ImageFormat::Png).unwrap();
        wordcraft_engine::cmd::insert::base64_encode(&b)
    }

    /// The Editing menu (top right) offers View only: choosing it makes the document read only (the
    /// button says so, typing does nothing, edit buttons grey out); Editing brings it back.
    #[test]
    fn editing_menu_offers_view_only() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("text.insert", json!({"text": "Hello"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 300.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let text = |h: &egui_kittest::Harness<WordApp>| h.state().session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        // The mode button is the "Editing" one next to Comments (the Editing group has the same name).
        h.query_all_by_label("Editing").last().unwrap().click();
        h.run_steps(4);
        h.get_by_label("View only — no changes or suggestions").click();
        h.run_steps(4);
        assert!(h.state().session.view_only, "the menu entry turns view only on");
        assert!(h.query_by_label("View only").is_some(), "the mode button says View only");
        assert!(h.state_mut().run("text.insert", json!({"text": "!"})).is_err(), "typing is refused");
        assert_eq!(text(&h), "Hello");
        h.query_all_by_label("View only").last().unwrap().click();
        h.run_steps(4);
        h.get_by_label("Editing — edit the document directly").click();
        h.run_steps(4);
        assert!(!h.state().session.view_only, "Editing turns it off again");
        h.state_mut().run("text.insert", json!({"text": "!"})).unwrap();
        assert_eq!(text(&h), "Hello!");
    }

    /// The title bar has no Discord community button (this fork removed it), and no Discord link is
    /// left anywhere: not on the Help tab, not in File, not as a command.
    #[test]
    fn title_bar_has_no_discord_button() {
        use egui_kittest::kittest::Queryable;
        let app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 300.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        // Left to right the title bar ends: the search box, [the Discord button], the Account circle.
        // The control next to Account is the wide search box (260 wide); the removed button was an
        // 84-wide control between them.
        let account = h.get_by_label("Account").rect();
        let next_to_account = h
            .query_all_by_role(egui::accesskit::Role::Unknown)
            .map(|n| n.rect())
            .filter(|r| r.min.y < 40.0 && r.max.x <= account.min.x)
            .max_by(|a, b| a.max.x.total_cmp(&b.max.x))
            .expect("the title bar has controls left of Account");
        assert!(next_to_account.width() > 200.0, "a button sits between the search box and Account: {next_to_account:?}");
        h.state_mut().ui.tab = "Help".into();
        h.run_steps(4);
        assert!(h.query_by_label_contains("Discord").is_none(), "the Help tab has no Discord button");
        h.state_mut().ui.backstage = true;
        h.run_steps(4);
        assert!(h.query_by_label_contains("Discord").is_none(), "File has no Discord item");
        assert!(h.state_mut().run("ui.discord", json!({})).is_err(), "ui.discord is no longer a command");
    }

    /// The View tab's Show group has a Text Boundaries toggle, on by default; clicking it turns the
    /// outline around the page's text area off and on again.
    #[test]
    fn view_tab_has_a_text_boundaries_toggle() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("ui.tab", json!({"tab": "View"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 300.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let ribbon_button = |label: &str| h.get_all_by_label(label).map(|node| node.rect()).min_by(|a, b| a.min.y.total_cmp(&b.min.y)).unwrap();
        let ruler = ribbon_button("Ruler");
        let gridlines = ribbon_button("Gridlines");
        let hide_white_space = ribbon_button("Hide White Space");
        let text_boundaries = ribbon_button("Text Boundaries");
        let navigation = ribbon_button("Navigation Pane");
        let zoom_in = ribbon_button("Zoom In");
        let zoom_out = ribbon_button("Zoom Out");
        let one_page = ribbon_button("One Page");
        let multiple_pages = ribbon_button("Multiple Pages");
        let page_width = ribbon_button("Page Width");

        // Show, in the two-size ribbon's two rows: Ruler and Gridlines, then Text Boundaries and
        // Hide White Space, left-aligned.
        assert!((gridlines.min.y - ruler.min.y).abs() < 0.5);
        assert!((gridlines.min.x - ruler.max.x).abs() <= 2.0, "the first row has no column-sized gap");
        assert!(text_boundaries.min.y > ruler.max.y - 1.0 && (hide_white_space.min.y - text_boundaries.min.y).abs() < 0.5, "two rows");
        assert!((text_boundaries.min.x - ruler.min.x).abs() < 0.5 && hide_white_space.min.x >= text_boundaries.max.x - 1.0);
        assert!(navigation.min.x >= gridlines.max.x && navigation.min.x >= hide_white_space.max.x && navigation.height() > ruler.height() * 2.0);
        assert!((zoom_in.min.y - zoom_out.min.y).abs() < 0.5);
        assert!((one_page.min.y - multiple_pages.min.y).abs() < 0.5);
        assert!(one_page.min.y > zoom_in.min.y);
        assert!(page_width.min.x >= multiple_pages.max.x && page_width.height() > one_page.height() * 2.0);
        assert!(h.state().session.view.text_boundaries, "on by default");
        h.get_by_label("Text Boundaries").click();
        h.run_steps(3);
        assert!(!h.state().session.view.text_boundaries, "the button turns it off");
        h.get_by_label("Text Boundaries").click();
        h.run_steps(3);
        assert!(h.state().session.view.text_boundaries, "and on again");
    }

    /// Table Design › Table Styles: "More" opens a panel of every style in groups, with New,
    /// Modify, Delete and Clear below; picking a tile styles the caret's table.
    #[test]
    fn table_styles_more_panel_lists_groups_and_applies_a_style() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
        app.run("ui.tab", json!({"tab": "Table Design"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h.get_by_label("More").click();
        h.run_steps(4);
        for group in ["Plain Tables", "Grid Tables", "List Tables"] {
            assert!(h.query_by_label(group).is_some(), "{group} group");
        }
        for action in ["Modify Table Style…", "Delete Table Style", "Clear", "New Table Style…"] {
            assert!(h.query_by_label(action).is_some(), "{action}");
        }
        let tile = h.get_all_by_label("Grid Table 4 – Blue").last().unwrap().rect().center();
        h.hover_at(tile);
        h.step();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: tile, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
            h.step();
        }
        h.run_steps(3);
        let state = h.state();
        let (tp, _, _) = state.session.sel.focus.path.cell().unwrap();
        let style = state.session.doc.table(state.session.sel.focus.story, &tp).unwrap().props.style.clone();
        assert_eq!(style.as_deref(), Some("GridTable4AccentBlue"));
        // Clear applies the plain Normal Table style.
        h.state_mut().run("table.style", json!({"style": "TableNormal"})).expect("Clear");
        let state = h.state();
        let (tp, _, _) = state.session.sel.focus.path.cell().unwrap();
        let style = state.session.doc.table(state.session.sel.focus.story, &tp).unwrap().props.style.clone();
        assert_eq!(style.as_deref(), Some("TableNormal"));
    }

    #[test]
    fn show_group_rows_share_edges_in_thai() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.ui.language = "th".into();
        app.run("ui.tab", json!({"tab": "View"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 300.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let rect = |label: &str| h.get_all_by_label(label).map(|n| n.rect()).min_by(|a, b| a.min.y.total_cmp(&b.min.y)).unwrap();
        let ruler = rect("ไม้บรรทัด");
        let white_space = rect("ซ่อนพื้นที่ว่าง");
        let grid = rect("เส้นตาราง");
        let boundaries = rect("ขอบเขตข้อความ");
        let nav = rect("บานหน้าต่างนำทาง");
        assert!((grid.min.y - ruler.min.y).abs() < 0.5 && (grid.min.x - ruler.max.x).abs() <= 2.0);
        assert!(boundaries.min.y > ruler.max.y - 1.0 && (white_space.min.y - boundaries.min.y).abs() < 0.5, "two rows");
        assert!((boundaries.min.x - ruler.min.x).abs() < 0.5 && white_space.min.x >= boundaries.max.x - 1.0);
        assert!(nav.min.x >= grid.max.x && nav.min.x >= white_space.max.x && nav.height() > ruler.height() * 2.0);
    }

    /// The Home tab's Paragraph group has a Thai Distribution button after Justify (and it sets
    /// that alignment, which lays Thai out with its extra space shared between characters).
    #[test]
    fn home_tab_has_a_thai_distribution_button_after_justify() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("text.insert", json!({"text": "สวัสดีครับ"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 300.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let justify = h.get_by_label("Justify").rect();
        let thai = h.get_by_label("Thai Distribution").rect();
        let sort = h.get_by_label("Sort").rect();
        assert!(thai.min.x > justify.max.x - 1.0 && (thai.center().y - justify.center().y).abs() < 1.0, "{justify:?} {thai:?}");
        let outdent = h.get_by_label("Decrease Indent").rect();
        let spacing = h.get_by_label("Line and Paragraph Spacing").rect();
        assert!((sort.center().y - outdent.center().y).abs() < 1.0, "sort and indent share the first row: {sort:?} {outdent:?}");
        assert!(
            (spacing.center().y - justify.center().y).abs() < 1.0 && justify.min.y > sort.max.y,
            "alignment and spacing share the second row: {justify:?} {spacing:?} {sort:?}"
        );
        h.get_by_label("Thai Distribution").click();
        h.run_steps(4);
        let align = h.state().session.doc.para_at(&wordcraft_doc::Pos::body(0, 0)).and_then(|p| p.props.align);
        assert_eq!(align, Some(wordcraft_doc::Align::ThaiDistribute));
    }

    /// The style set gallery fills exactly its own box and the next control starts after a gap,
    /// so Colors and Fonts can't overlap the thumbnails (#638).
    #[test]
    fn design_style_set_gallery_stays_in_its_box() {
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let mut seen = None;
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 200.0)).build_ui(|ui| {
            ui.horizontal_top(|ui| {
                let (gallery, used) = style_set_box(&mut app, ui);
                seen = Some((gallery, used, ui.cursor().min.x));
            });
        });
        h.run_steps(2);
        drop(h);
        let (gallery, used, next_x) = seen.unwrap();
        assert!((gallery.width() - crate::previews::STYLE_SET_GALLERY_W).abs() < 0.01);
        assert!(used.right() <= gallery.right() + 0.01, "{used:?} {gallery:?}");
        assert!(next_x >= gallery.right() + 8.0 - 0.01, "{next_x} {gallery:?}");
    }

    /// #658: menu buttons grey out (and don't open) while the command their menu is about is
    /// disabled: the Table Layout menus outside a table; menus with no command stay on.
    #[test]
    fn menu_buttons_follow_their_command() {
        use crate::widgets::menu_enabled;
        let mut a = crate::WordApp::new(Session::new(wordcraft_doc::Document::from_text("Text")), crate::Services::default());
        for id in ["table.selectTable", "table.deleteTable", "table.autofit"] {
            assert!(!menu_enabled(&a, id), "{id} is off outside a table");
        }
        assert!(menu_enabled(&a, ""), "a menu without a command is always on");
        assert!(menu_enabled(&a, "insert.header"));
        a.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
        for id in ["table.selectTable", "table.deleteTable", "table.autofit"] {
            assert!(menu_enabled(&a, id), "{id} is on inside a table");
        }
    }

    /// Every menu picture (Layout's Text Direction, Orientation, Columns, Breaks, Line Numbers and
    /// Margins; the table menus' Borders, Select, Delete and AutoFit) draws something: a misspelt
    /// name would leave an empty space before the label.
    #[test]
    fn layout_menu_pictures_all_draw() {
        let names = [
            "dirHorizontal",
            "dirDown",
            "dirUp",
            "portrait",
            "landscape",
            "cols_one",
            "cols_two",
            "cols_three",
            "cols_left",
            "cols_right",
            "colsMore",
            "brk_page",
            "brk_column",
            "brk_textWrapping",
            "brk_nextPage",
            "brk_continuous",
            "brk_evenPage",
            "brk_oddPage",
            "ln_none",
            "ln_continuous",
            "ln_restartPage",
            "ln_restartSection",
            "lnOptions",
            "mg_normal",
            "mg_narrow",
            "mg_moderate",
            "mg_wide",
            "mg_mirrored",
            "mg_office2003",
            "mg_custom",
            "tb_all",
            "tb_outside",
            "tb_inside",
            "tb_none",
            "tb_top",
            "tb_bottom",
            "tb_left",
            "tb_right",
            "tb_insideH",
            "tb_insideV",
            "tb_diagDown",
            "tb_diagUp",
            "hline",
            "gridlines",
            "sel_cell",
            "sel_column",
            "sel_row",
            "sel_table",
            "del_cell",
            "del_column",
            "del_row",
            "del_table",
            "fit_contents",
            "fit_window",
            "fit_fixed",
        ];
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
        for name in names {
            let out = ctx.run_ui(egui::RawInput::default(), |ui| {
                let r = Rect::from_min_size(pos2(10.0, 10.0), vec2(24.0, 20.0));
                layout_picture(ui.painter(), r, name, egui::Color32::BLACK, egui::Color32::BLUE);
            });
            assert!(out.shapes.len() >= 2, "{name} draws its picture");
            out.drop_without_applying_deltas();
        }
    }

    /// Insert › Shapes shows each shape's picture beside its name; choosing one inserts it.
    #[test]
    fn shapes_menu_items_insert_their_shape() {
        use egui_kittest::kittest::Queryable;
        let mut a = crate::WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.run("ui.tab", json!({"tab": "Insert"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
            |ui, app: &mut crate::WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Shapes").click();
        h.run_steps(3);
        h.get_by_label("Oval").click();
        h.run_steps(3);
        assert!(has_shape_selected(&h.state().session), "choosing Oval inserted a shape and selected it");
    }

    /// #1206: Margins, Orientation, Size and Columns mark the caret section's setting, as Word
    /// does. Every value differs from a new document's, so a default can't satisfy it.
    #[test]
    fn page_setup_menus_mark_the_current_setting() {
        use egui_kittest::kittest::{By, NodeT, Queryable};
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::from_text("Text")), crate::Services::default());
        app.run("layout.margins", json!({"preset": "narrow"})).unwrap();
        app.run("layout.orientation", json!({"value": "landscape"})).unwrap();
        app.run("layout.size", json!({"name": "A4"})).unwrap();
        app.run("layout.columns", json!({"preset": "right"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(400.0, 1200.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                margins_items(ui, app);
                orientation_items(ui, app);
                size_menu(ui, app);
                columns_items(ui, app);
            },
            app,
        );
        h.run_steps(2);
        let checked: Vec<String> = h
            .query_all(By::new().predicate(|n| n.toggled() == Some(egui::accesskit::Toggled::True)))
            .filter_map(|n| n.accesskit_node().label())
            .collect();
        let first_words: Vec<&str> = checked.iter().filter_map(|l| l.trim_start_matches('✓').split_whitespace().next()).collect();
        assert_eq!(first_words, ["Narrow", "Landscape", "A4", "Right"], "{checked:?}");
    }

    /// #860 review: a tag the language list doesn't have shows by language and region, not as
    /// the tag; Portuguese (Angola) isn't Brazil's, and no name shows twice in the menu.
    #[test]
    fn other_regions_show_by_name_once() {
        assert_eq!(language_name(Some("fr-CA")), "French (Canada)");
        assert_eq!(language_name(Some("en-AU")), "English (Australia)");
        assert_eq!(language_name(Some("de_AT")), "German (Austria)");
        assert_eq!(language_name(Some("pt-AO")), "Portuguese (Angola)");
        assert_eq!(language_name(Some("sr-Latn-BA")), "Serbian (Bosnia and Herzegovina)");
        assert_eq!(language_name(Some("en-ZZ")), "English (ZZ)");
        assert_eq!(language_name(Some("xx-YY")), "xx-YY");
        let exact = |cur: &'static str| move |t: &str| t.eq_ignore_ascii_case(cur);
        for tag in ["fr-CA", "pt-AO", "en-AU"] {
            let c = language_checks(tag, exact(tag));
            let name = c.extra.expect("shown above the list");
            assert!(!PROOFING_LANGUAGES.iter().any(|(_, n)| *n == name), "{name} is listed once");
            assert!(!c.listed.contains(&true), "{tag}: nothing in the list checked");
        }
        // A tag named like a list entry checks that entry instead of showing it again.
        let c = language_checks("PT-br-x-test", |_| false);
        assert_eq!(c.extra, None);
        assert_eq!(c.listed, PROOFING_LANGUAGES.map(|(t, _)| t == "pt-BR"));
    }

    #[test]
    fn czech_has_a_name_in_the_language_menu() {
        assert!(PROOFING_LANGUAGES.contains(&("cs-CZ", "Czech")));
        for tag in ["cs", "cs-CZ", "CS-cz", "ces", "CZE"] {
            assert_eq!(language_name(Some(tag)), "Czech", "{tag}");
        }
    }

    #[test]
    fn portuguese_has_a_name_in_the_language_menu() {
        assert!(PROOFING_LANGUAGES.iter().any(|(t, _)| *t == "pt-BR"));
        assert_eq!(language_name(Some("pt-BR")), "Portuguese (Brazil)");
        assert_eq!(language_name(Some("pt")), "Portuguese (Brazil)");
        assert_eq!(language_name(Some("PT-BR".to_string().as_str())), "Portuguese (Brazil)");
    }

    #[test]
    fn turkish_has_a_name_in_the_language_menu() {
        assert!(PROOFING_LANGUAGES.iter().any(|(t, _)| *t == "tr-TR"));
        assert_eq!(language_name(Some("tr-TR")), "Turkish");
        assert_eq!(language_name(Some("tr")), "Turkish");
        assert_eq!(language_name(Some("TR-TR")), "Turkish");
    }

    #[test]
    fn picture_format_tab_appears_when_picture_selected() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        assert!(!has_picture_selected(&s));
        assert!(!contextual_tabs(&s).contains(&"Picture Format"));
        s.run("insert.picture", &json!({"data": png_data()})).unwrap();
        assert!(has_picture_selected(&s));
        assert!(contextual_tabs(&s).contains(&"Picture Format"));
        s.run("select.collapse", &json!({"end": true})).unwrap();
        s.run("text.insert", &json!({"text": "x"})).unwrap();
        assert!(!has_picture_selected(&s));
    }

    /// The width and height boxes of a picture follow File › Options › Advanced › Measurement
    /// units (not a fixed `pt`), as the table's do; a typed length in another unit converts.
    #[test]
    fn picture_size_boxes_show_the_options_unit() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.session.run("insert.picture", &json!({"data": png_data()})).unwrap();
        app.session.prefs.units = Some(wordcraft_geom::Unit::Centimeters);
        crate::options::sync(&app);
        app.run("ui.tab", json!({"tab": "Picture Format"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        let values: Vec<String> = h.get_all_by_role(egui::accesskit::Role::TextInput).filter_map(|n| n.value()).collect();
        assert!(values.iter().any(|v| v.ends_with("cm")), "a size box in cm: {values:?}");
        assert!(!values.iter().any(|v| v.ends_with("pt")), "no size box in points: {values:?}");
    }

    #[test]
    fn shape_format_tab_appears_when_shape_selected() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("insert.shape", &json!({"kind": "rectangle"})).unwrap();
        assert!(contextual_tabs(&s).contains(&"Shape Format"));
        assert!(!contextual_tabs(&s).contains(&"Picture Format"));
        s.run("select.collapse", &json!({"end": true})).unwrap();
        s.run("text.insert", &json!({"text": "x"})).unwrap();
        assert!(!contextual_tabs(&s).contains(&"Shape Format"));
    }

    /// Groups have no launchers (owner, 2026-11-13); every dialog or pane they opened is still
    /// in the ribbon, at the end of a related menu or as a button of its group.
    #[test]
    fn launcher_dialogs_have_other_ways_in() {
        // The ribbon's own code, without these tests.
        let src = include_str!("ribbon.rs").split("#[cfg(test)]").next().unwrap();
        for id in ["edit.clipboardPane", "references.noteOptions", "review.trackingOptions", "\"pageSetup\"", "para.dialog"] {
            assert!(src.contains(id), "{id} is not reachable from the ribbon");
        }
        let gallery = include_str!("previews.rs");
        assert!(gallery.contains("view.stylesPane"), "the Styles gallery's More button opens the Styles pane");
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("layout.pageSetup", json!({})).unwrap();
    }

    #[test]
    fn smart_art_picker_inserts_and_design_tab_follows_the_selection() {
        let mut app = crate::WordApp::new(Session::new(wordcraft_doc::Document::new()), Default::default());
        // Insert › SmartArt without a layout opens the picker.
        let _ = app.run("insert.smartArt", json!({}));
        assert_eq!(app.dialog.as_ref().map(crate::dialogs::Dialog::name), Some("insertSmartArt"));
        app.dialog = None;
        app.run("insert.smartArt", json!({"layout": "cycle"})).unwrap();
        assert!(contextual_tabs(&app.session).contains(&"SmartArt Design"));
        app.run("smartArt.textPane", json!({})).unwrap();
        assert!(app.session.view.smart_art_pane);
        app.session.run("select.collapse", &json!({"end": true})).unwrap();
        app.session.run("text.insert", &json!({"text": "x"})).unwrap();
        assert!(!contextual_tabs(&app.session).contains(&"SmartArt Design"));
    }

    #[test]
    fn chart_design_tab_appears_when_chart_selected_and_buttons_open_dialogs() {
        let mut app = crate::WordApp::new(Session::new(wordcraft_doc::Document::new()), Default::default());
        // Insert › Chart without a type opens the gallery; choosing inserts.
        let _ = app.run("insert.chart", json!({}));
        assert_eq!(app.dialog.as_ref().map(crate::dialogs::Dialog::name), Some("insertChart"));
        app.dialog = None;
        app.run("insert.chart", json!({"type": "doughnut"})).unwrap();
        assert!(contextual_tabs(&app.session).contains(&"Chart Design"));
        let _ = app.run("chart.editData", json!({}));
        assert_eq!(app.dialog.as_ref().map(crate::dialogs::Dialog::name), Some("chartData"));
        let _ = app.run("chart.type", json!({}));
        assert_eq!(app.dialog.as_ref().map(crate::dialogs::Dialog::name), Some("changeChartType"));
        app.session.run("select.collapse", &json!({"end": true})).unwrap();
        app.session.run("text.insert", &json!({"text": "x"})).unwrap();
        assert!(!contextual_tabs(&app.session).contains(&"Chart Design"));
    }

    /// No group's content runs into its label: View › Show stacked four check boxes in a column
    /// made for three, so "Hide White Space" was drawn under "Show". Every tab, every language.
    #[test]
    fn ribbon_groups_fit_above_their_labels() {
        let app = WordApp::new(Session::new(wordcraft_engine::sample::sample_document()), crate::Services::default());
        // Wide enough that no tab hides groups behind its scroll arrows.
        let mut h = egui_kittest::Harness::builder().with_size(vec2(2560.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        let mut tall = Vec::new();
        for lang in crate::i18n::Lang::all() {
            h.state_mut().run("ui.language", json!({"value": lang.code()})).unwrap();
            for tab in TABS.iter().filter(|t| **t != "File") {
                h.state_mut().run("ui.tab", json!({"tab": tab})).unwrap();
                h.run_steps(2);
                crate::widgets::TALL_GROUPS.with(|g| g.borrow_mut().clear());
                h.run_steps(1);
                let found = crate::widgets::TALL_GROUPS.with(|g| g.take());
                tall.extend(found.into_iter().map(|g| format!("{} {tab} › {g}", lang.code())));
            }
        }
        tall.sort();
        tall.dedup();
        assert!(tall.is_empty(), "groups taller than the ribbon: {tall:#?}");
    }

    /// The tabs that appear for an object (a table, picture, shape, chart, SmartArt, equation)
    /// fit the two-row ribbon too, in every language.
    #[test]
    fn contextual_groups_fit_above_their_labels() {
        let picture = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/app-icon/khiankrub-192.png");
        let cases: [(&str, Value, &[&str]); 6] = [
            ("insert.table", json!({"rows": 3, "cols": 3}), &["Table Design", "Table Layout"]),
            ("insert.picture", json!({"path": picture}), &["Picture Format"]),
            ("insert.shape", json!({"kind": "rectangle"}), &["Shape Format"]),
            ("insert.chart", json!({"type": "column"}), &["Chart Design"]),
            ("insert.smartArt", json!({"layout": "process"}), &["SmartArt Design"]),
            ("insert.equation", json!({}), &["Equation"]),
        ];
        let mut tall = Vec::new();
        for (cmd, params, tabs) in cases {
            let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
            app.run(cmd, params).unwrap();
            let mut h = egui_kittest::Harness::builder().with_size(vec2(2560.0, 900.0)).build_ui_state(
                |ui, app: &mut WordApp| {
                    let ctx = ui.ctx().clone();
                    app.logic(&ctx);
                    app.ui(ui);
                },
                app,
            );
            for lang in crate::i18n::Lang::all() {
                h.state_mut().run("ui.language", json!({"value": lang.code()})).unwrap();
                for tab in tabs.iter() {
                    assert!(contextual_tabs(&h.state().session).contains(tab), "{cmd} shows {tab}");
                    h.state_mut().run("ui.tab", json!({"tab": tab})).unwrap();
                    h.run_steps(2);
                    crate::widgets::TALL_GROUPS.with(|g| g.borrow_mut().clear());
                    h.run_steps(1);
                    tall.extend(crate::widgets::TALL_GROUPS.with(|g| g.take()).into_iter().map(|g| format!("{} {tab} › {g}", lang.code())));
                }
            }
        }
        tall.sort();
        tall.dedup();
        assert!(tall.is_empty(), "groups taller than the ribbon: {tall:#?}");
    }

    /// Going into a header brings up the Header & Footer tab (its groups fit in every language);
    /// Close Header and Footer takes it away and goes back to the tab before.
    #[test]
    fn header_footer_tab_comes_and_goes_with_the_header() {
        let app = WordApp::new(Session::new(wordcraft_doc::Document::from_text("Text")), crate::Services::default());
        let mut h = egui_kittest::Harness::builder().with_size(vec2(2560.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.state_mut().run("ui.tab", json!({"tab": "Review"})).unwrap();
        h.run_steps(2);
        assert!(!contextual_tabs(&h.state().session).contains(&"Header & Footer"));
        h.state_mut().run("insert.editHeader", json!({})).unwrap();
        h.run_steps(2);
        assert!(contextual_tabs(&h.state().session).contains(&"Header & Footer"));
        assert_eq!(h.state().ui.tab, "Header & Footer");
        let mut tall = Vec::new();
        for lang in crate::i18n::Lang::all() {
            h.state_mut().run("ui.language", json!({"value": lang.code()})).unwrap();
            h.run_steps(2);
            crate::widgets::TALL_GROUPS.with(|g| g.borrow_mut().clear());
            h.run_steps(1);
            tall.extend(crate::widgets::TALL_GROUPS.with(|g| g.take()).into_iter().map(|g| format!("{} {g}", lang.code())));
        }
        assert!(tall.is_empty(), "groups taller than the ribbon: {tall:#?}");
        h.state_mut().run("insert.closeHeader", json!({})).unwrap();
        h.run_steps(2);
        assert!(!contextual_tabs(&h.state().session).contains(&"Header & Footer"));
        assert_eq!(h.state().ui.tab, "Review");
    }

    #[test]
    fn fit_tabs_keeps_the_kept_tabs_and_drops_from_the_end() {
        let tabs = [(40.0, true), (50.0, false), (50.0, false), (50.0, false), (60.0, true)];
        assert_eq!(fit_tabs(&tabs, 250.0, 20.0), [true; 5]);
        // 250 doesn't fit 240 with room for »: the last plain tabs give way, the kept ones stay.
        assert_eq!(fit_tabs(&tabs, 240.0, 20.0), [true, true, true, false, true]);
        // 200 would fit 210, but not with the » button too.
        assert_eq!(fit_tabs(&tabs, 210.0, 20.0), [true, true, false, false, true]);
        assert_eq!(fit_tabs(&tabs, 150.0, 20.0), [true, false, false, false, true]);
        // Nothing but the kept tabs fits: they stay anyway.
        assert_eq!(fit_tabs(&tabs, 10.0, 20.0), [true, false, false, false, true]);
        // Hostile numbers don't panic.
        let _ = fit_tabs(&[(f32::NAN, false), (f32::INFINITY, false)], f32::NAN, 20.0);
    }

    /// #860: Review › Language (and the status bar) reach every proofing language, not just the
    /// short list: Set Proofing Language… opens the Language dialog (French, Set As Default),
    /// Language Preferences… opens Options › Language, and French text shows by name, checked.
    #[test]
    fn language_menu_reaches_every_language() {
        use egui_kittest::kittest::{By, NodeT, Queryable};
        let item = |label: &'static str| By::new().label_contains(label);
        let app = WordApp::new(Session::new(wordcraft_doc::Document::from_text("Bonjour")), crate::Services::default());
        let mut h =
            egui_kittest::Harness::builder().with_size(vec2(400.0, 400.0)).build_ui_state(|ui, app: &mut WordApp| language_menu(ui, app), app);
        h.run();
        assert!(h.query(item("French (France)")).is_none(), "the short list only, on an English document");
        assert!(!h.get(item("Set Proofing Language…")).accesskit_node().is_disabled(), "never greyed out on a fresh document");
        assert!(!h.get(item("English (United States)")).accesskit_node().is_disabled());
        h.get(item("Set Proofing Language…")).click();
        h.run();
        assert_eq!(h.state().dialog.as_ref().map(crate::dialogs::Dialog::name), Some("language"));
        h.state_mut().dialog = None;
        // French for the whole document: the menu and the status bar name it.
        h.state_mut().run("review.language", json!({"lang": "fr-FR", "default": true})).unwrap();
        h.run();
        assert_eq!(caret_language(h.state()), "French (France)");
        assert!(h.query(item("French (France)")).is_some(), "the current language is listed (only while it is the current one)");
        h.get(item("Language Preferences…")).click();
        h.run();
        assert!(h.state().ui.backstage, "File › Options opened");
        assert_eq!(h.state().ui.options_pane, "language");
        assert_eq!(language_name(Some("de-DE")), "German (Germany)");
    }

    #[test]
    fn resolve_tab_falls_back_when_contextual_tab_expires() {
        let all = ["Home", "Table Design", "Picture Format"];
        assert_eq!(resolve_tab("Picture Format", &all), "Picture Format");
        assert_eq!(resolve_tab("Picture Format", &["Home"]), "Home");
        assert_eq!(resolve_tab("Table Design", &["Home"]), "Home");
        assert_eq!(resolve_tab("Home", &["Home"]), "Home");
    }
}
