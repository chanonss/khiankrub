//! File tab (Backstage): Home/New, Open, Info, Save As, Print, Export, Close, Options, About.

use egui::{Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::theme::{Tokens, medium, regular, semibold};
use crate::{WordApp, icons};

const PAGES: [(&str, &str); 11] = [
    ("home", "Home"),
    ("new", "New"),
    ("open", "Open"),
    ("info", "Info"),
    ("recover", "Recover"),
    ("save", "Save"),
    ("saveAs", "Save As"),
    ("print", "Print"),
    ("export", "Export"),
    ("close", "Close"),
    ("options", "Options"),
];

/// The File menu's pages, by id and label. In the browser (`web`) one Save page offers both
/// places, this device and Google Drive, so there is no separate Save As; and there is no Close,
/// which has no window to close there.
fn pages(web: bool) -> Vec<(&'static str, &'static str)> {
    PAGES
        .iter()
        .filter(|(id, _)| !(web && matches!(*id, "save" | "close")))
        .map(|&(id, label)| if web && id == "saveAs" { (id, "Save") } else { (id, label) })
        .collect()
}

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // No document open (#1129): only the pages that open or create one, and Options.
    let has_document = app.has_document();
    if !has_document && !works_without_document(&app.ui.backstage_page) {
        app.ui.backstage_page = "home".into();
    }
    // The web's start screen (nothing open yet): a page of its own, like an online word
    // processor's home, instead of File's sidebar. File › Home inside the editor stays as it is.
    if !has_document && app.ui.backstage_page == "home" && app.services.browser_recovery.is_some() {
        web_home(app, ui);
        return;
    }
    let mac = app.integrated_titlebar;
    egui::Panel::left("backstage_nav")
        .exact_size(200.0)
        // Fixed width, as in Word: without this egui shows a resize handle that does nothing (#502).
        .resizable(false)
        // The navy of the title bar and tab strip (darker in the dark theme), so File matches them.
        .frame(egui::Frame::NONE.fill(Tokens::get(ui.ctx()).title_bar).inner_margin(egui::Margin {
            left: 0,
            right: 0,
            top: if mac { 0 } else { 12 },
            bottom: 12,
        }))
        .show(ui, |ui| {
            if mac {
                // macOS (#222): the Backstage covers the title bar, so its top band is the title
                // bar (beside the traffic lights): it drags the window.
                let (_, drag) = ui.allocate_exact_size(vec2(200.0, app.mac_titlebar.height), Sense::click_and_drag());
                if drag.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            }
            // Escape in the built-in file browser closes only the browser.
            if !app.file_dialog_open() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
            ui.add_space(if mac { 12.0 } else { 6.0 });
            for (id, label) in pages(cfg!(target_arch = "wasm32")) {
                // No document open (#1129): Info, Save, Save As, Print and Export are greyed out.
                let on = has_document || works_without_document(id);
                let (r, resp) = ui.add_enabled_ui(on, |ui| ui.allocate_exact_size(vec2(200.0, 38.0), Sense::click())).inner;
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, on, tl!(label)));
                let active = app.ui.backstage_page == id;
                if active {
                    ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(46));
                } else if on && resp.hovered() {
                    ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(26));
                }
                let ink = if on { egui::Color32::WHITE } else { egui::Color32::from_white_alpha(110) };
                nav_icon(ui, r, page_icon(id), ink);
                ui.painter().text(
                    pos2(r.min.x + NAV_TEXT_X, r.center().y),
                    Align2::LEFT_CENTER,
                    tl!(label),
                    if active { semibold(14.0) } else { medium(14.0) },
                    ink,
                );
                if resp.clicked() {
                    match id {
                        // A document without a file goes to the Save As page (as in Word) instead
                        // of `file.save`, which would also open the native Save As dialog.
                        // In the browser a save is a download, so it saves right away.
                        "save" if app.session.path.is_none() && app.services.download.is_none() => {
                            app.ui.backstage_page = "saveAs".into();
                        }
                        "save" => {
                            let r = app.run("file.save", json!({}));
                            // A Google Drive document's save goes back to Drive and finishes there.
                            if r.is_ok_and(|v| {
                                v.get("saved").and_then(serde_json::Value::as_bool) == Some(true)
                                    || v.get("downloaded").is_some()
                                    || v.get("googleDrive").is_some()
                            }) {
                                app.ui.backstage = false;
                                app.canvas.want_focus = true;
                            } else {
                                app.ui.backstage_page = "saveAs".into();
                            }
                        }
                        "saveAs" => {
                            app.ui.backstage_page = "saveAs".into();
                        }
                        // Not a page: `file.close` asks Save / Don't Save / Cancel first when there
                        // are unsaved changes, and Cancel leaves the Backstage where it was.
                        "close" => {
                            let _ = app.run("file.close", json!({}));
                        }
                        _ => app.ui.backstage_page = id.into(),
                    }
                }
            }
            // Back to the document: after Options, set apart by a rule, as it opens no page.
            ui.add_space(8.0);
            let (rule, _) = ui.allocate_exact_size(vec2(200.0, 1.0), Sense::hover());
            ui.painter().hline(rule.min.x + 22.0..=rule.max.x - 22.0, rule.center().y, Stroke::new(1.0, egui::Color32::from_white_alpha(46)));
            ui.add_space(8.0);
            let (r, resp) = ui.allocate_exact_size(vec2(200.0, 38.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Back")));
            if resp.hovered() {
                ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(26));
            }
            nav_icon(ui, r, "back", egui::Color32::WHITE);
            ui.painter().text(pos2(r.min.x + NAV_TEXT_X, r.center().y), Align2::LEFT_CENTER, tl!("Back"), medium(14.0), egui::Color32::WHITE);
            if resp.clicked() {
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                let (r, resp) = ui.allocate_exact_size(vec2(200.0, 32.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tl!("About")));
                if resp.hovered() {
                    ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(26));
                }
                nav_icon(ui, r, "info", egui::Color32::WHITE);
                ui.painter().text(pos2(r.min.x + NAV_TEXT_X, r.center().y), Align2::LEFT_CENTER, tl!("About"), regular(13.0), egui::Color32::WHITE);
                if resp.clicked() {
                    app.dialog = crate::dialogs::Dialog::open("about", app);
                }
            });
        });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(40, 30))).show(ui, |ui| {
        // The Print page fills the pane and scrolls its settings and its preview itself (#1156).
        if app.ui.backstage_page == "print" {
            export_page(app, ui);
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| match app.ui.backstage_page.as_str() {
            "home" => home_page(app, ui),
            "new" => new_page(app, ui),
            "open" => open_page(app, ui),
            "saveAs" => save_as_page(app, ui),
            "info" => info_page(app, ui),
            "recover" => recovery_page(app, ui),
            "print" | "export" => export_page(app, ui),
            "options" => crate::options::page(app, ui),
            _ => home_page(app, ui),
        });
    });
}

/// Recovery is a separate copy. Leaving this page never discards it.
/// It follows the theme like the other File pages; its text was forced white on a black page,
/// which left the buttons' labels white on white in the light theme.
fn recovery_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Recover documents");
    ui.label(tl!("Restore opens an unsaved copy. Your original file stays unchanged."));
    ui.add_space(20.0);
    if app.session.password.is_some() {
        ui.label(tl!("Recovery is disabled while this document has a password."));
    }
    if let Some(error) = app.session.recovery_error() {
        ui.label(error);
    }
    if !app.session.recovery_ready() {
        ui.label(tl!("Local recovery is unavailable or still loading."));
    }
    let documents = app.session.recovery_documents();
    if documents.is_empty() && app.session.recovery_ready() {
        ui.label(tl!("No recovery copies."));
    }
    let busy = app.session.recovery_busy();
    egui::Grid::new("recovery_documents").min_col_width(180.0).spacing(vec2(24.0, 16.0)).show(ui, |ui| {
        if !documents.is_empty() {
            ui.strong(tl!("Document"));
            ui.strong(tl!("Recovery copy"));
            ui.end_row();
        }
        for doc in documents {
            ui.label(&doc.name);
            ui.label(&doc.saved_at);
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(tl!("Restore")).clicked() {
                        let _ = app.run("file.recover", json!({"restore": doc.id}));
                    }
                    if ui.button(tl!("Discard…")).clicked() {
                        let _ = app.run("ui.recoveryDiscard", json!({"id": doc.id}));
                    }
                });
            });
            ui.end_row();
        }
    });
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui.button(tl!("Continue without restoring")).clicked() {
            let _ = app.run("ui.backstage", json!({"value": false}));
            app.canvas.want_focus = true;
        }
        if ui.add_enabled(!busy && app.session.recovery_ready(), egui::Button::new(tl!("Refresh"))).clicked() {
            let _ = app.run("file.recover", json!({"refresh": true}));
        }
    });
}

/// The Backstage pages that work while no document is open (#1129).
fn works_without_document(page: &str) -> bool {
    matches!(page, "home" | "new" | "open" | "options")
}

/// Where a File menu entry's label starts: after its icon ([`nav_icon`]).
const NAV_TEXT_X: f32 = 50.0;

/// The icon before a File menu entry's label, in the label's colour (one colour on the blue).
fn nav_icon(ui: &Ui, row: Rect, icon: &str, ink: egui::Color32) {
    let r = Rect::from_center_size(pos2(row.min.x + 32.0, row.center().y), vec2(18.0, 18.0));
    crate::icons::paint(ui.painter(), r, icon, ink, ink);
}

/// The icon of each File menu page.
fn page_icon(id: &str) -> &'static str {
    match id {
        "home" => "homeFolder",
        "new" => "blankPage",
        "open" => "open",
        "info" => "info",
        "recover" => "recover",
        "save" => "save",
        // On the web the Save page (id "saveAs") is the only save entry, so it shows Save's icon.
        "saveAs" if cfg!(target_arch = "wasm32") => "save",
        "saveAs" => "saveAs",
        "print" => "print",
        "export" => "share",
        "close" => "closeFile",
        _ => "options",
    }
}

fn heading(ui: &mut Ui, s: &str) {
    ui.label(egui::RichText::new(tl!(s)).font(semibold(26.0)));
    ui.add_space(16.0);
}

/// `new_tab`: the web start screen opens the template in a new tab (`./?new=<template>`), so the
/// start screen stays where it is; elsewhere the template replaces the current document.
fn template_tile(ui: &mut Ui, app: &mut WordApp, label: &str, template: &str, new_tab: bool) {
    let t = Tokens::get(ui.ctx());
    ui.vertical(|ui| {
        // The label sits right under the thumbnail whatever row gap the gallery uses.
        ui.spacing_mut().item_spacing.y = 0.0;
        let (r, resp) = ui.allocate_exact_size(vec2(150.0, 194.0), Sense::click());
        ui.painter().rect(
            r,
            2.0,
            egui::Color32::WHITE,
            Stroke::new(if resp.hovered() { 2.0 } else { 1.0 }, if resp.hovered() { t.accent } else { t.border_strong }),
            egui::StrokeKind::Inside,
        );
        let ppp = ui.ctx().pixels_per_point();
        let key = format!("tpl:{template}:{ppp}");
        let region = app.session.region();
        let tex = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(egui::Id::new(&key))).or_else(|| {
            let mut s = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
            // The blank tile shows the region's paper (#838).
            s.system_region = region;
            let _ = s.run("file.new", &json!({"template": template}));
            let l = s.export_layout();
            let page = l.pages.first()?;
            let img = wordcraft_render::render_page(&s.doc, page, 150.0 / page.w * ppp, &crate::canvas::screen_render_options());
            let px = img.to_straight();
            let h = ui.ctx().load_texture(
                &key,
                egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &px),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(&key), h.clone()));
            Some(h)
        });
        if let Some(h) = tex {
            ui.painter().image(h.id(), r.shrink(1.0), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
        // The caption opens the template too, as in Word.
        let caption = ui.add(egui::Label::new(egui::RichText::new(crate::i18n::tc("template", label)).font(regular(12.5))).sense(Sense::click()));
        if resp.clicked() || caption.clicked() {
            if new_tab {
                app.canvas.open_url = Some(format!("./?new={template}"));
            } else {
                let _ = app.run("file.new", json!({"template": template}));
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
        }
    });
}

/// The built-in templates in gallery order: display label and the `file.new` template id.
const TEMPLATES: [(&str, &str); 5] =
    [("Blank document", "blank"), ("Studio handbook (sample)", "sample"), ("Letter", "letter"), ("Résumé", "resume"), ("Report", "report")];

/// How many templates Home shows before "More templates" (the rest are on New).
const HOME_TEMPLATES: usize = 4;

/// The templates whose English or translated label, or id, contains `query` (case-insensitive;
/// the id lets "resume" find "Résumé"); all of them for an empty query.
fn matching_templates(query: &str) -> Vec<(&'static str, &'static str)> {
    let q = query.trim().to_lowercase();
    TEMPLATES
        .into_iter()
        .filter(|(label, id)| {
            q.is_empty() || id.contains(&q) || label.to_lowercase().contains(&q) || crate::i18n::tc("template", label).to_lowercase().contains(&q)
        })
        .collect()
}

/// Home: a greeting, Blank plus a short row of templates with a link to New, then Recent.
fn home_page(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let hour = wordcraft_engine::cmd::now_local_iso().get(11..13).and_then(|h| h.parse::<u32>().ok()).unwrap_or(9);
    heading(
        ui,
        if hour < 12 {
            "Good morning"
        } else if hour < 18 {
            "Good afternoon"
        } else {
            "Good evening"
        },
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 0.0);
        for (label, template) in TEMPLATES.into_iter().take(HOME_TEMPLATES) {
            template_tile(ui, app, label, template, false);
        }
    });
    ui.add_space(8.0);
    let more = ui.add(egui::Label::new(egui::RichText::new(tl!("More templates →")).font(medium(13.5)).color(t.accent)).sense(Sense::click()));
    if more.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.ui.backstage_page = "new".into();
    }
    ui.add_space(24.0);
    // The browser has no files to reopen: Home lists the drafts kept in it instead.
    if app.services.browser_recovery.is_some() {
        drafts_list(app, ui, false);
    } else {
        open_list(app, ui);
    }
}

/// The web start screen: a top bar (KhianKrub, Open, Google Drive, Options, back to the KrubOffice
/// apps), a greeting, every template, then the drafts kept in this browser.
fn web_home(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("web_home_bar")
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(20, 10)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // KhianKrub's mark: a gold ข on navy, as on its icon.
                let (r, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
                ui.painter().rect_filled(r, 8.0, egui::Color32::from_rgb(0x1e, 0x3a, 0x8a));
                krub_icons::glass_letter(ui.painter(), r.center() + vec2(0.0, 1.0), "ข", semibold(22.0), egui::Color32::from_rgb(0xf2, 0xb7, 0x05));
                ui.add_space(4.0);
                // The name gives way to the buttons on narrow screens (an iPad in portrait).
                if ui.available_width() > 560.0 {
                    ui.label(egui::RichText::new(tl!("KhianKrub")).font(semibold(17.0)));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    app_switcher(ui);
                    if ui.button(egui::RichText::new(tl!("Options")).font(medium(13.5))).clicked() {
                        app.ui.backstage_page = "options".into();
                    }
                    if app.services.google_drive.is_some() && crate::widgets::icon_text_button(ui, "cloud", "Google Drive", medium(13.5)).clicked() {
                        app.ui.backstage_page = "open".into();
                    }
                    if crate::widgets::icon_text_button(ui, "open", tl!("Open file"), medium(13.5)).clicked() {
                        let _ = app.run("ui.openFileDialog", json!({}));
                    }
                });
            });
        });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // A centred column, as wide as the template row, on wide windows.
            let width = ui.available_width().min(900.0);
            let side = ((ui.available_width() - width) / 2.0).max(0.0) + 24.0;
            ui.add_space(28.0);
            ui.horizontal_top(|ui| {
                ui.add_space(side);
                ui.vertical(|ui| {
                    ui.set_max_width(width - 48.0);
                    ui.label(egui::RichText::new(tl!("Hello! What would you like to write?")).font(semibold(24.0)));
                    ui.label(
                        egui::RichText::new(tl!("Pick a template, or pick up a draft where you left off.")).font(regular(13.5)).color(t.text_dim),
                    );
                    ui.add_space(18.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
                        for (label, template) in TEMPLATES {
                            template_tile(ui, app, label, template, true);
                        }
                    });
                    ui.add_space(28.0);
                    drafts_list(app, ui, true);
                    ui.add_space(28.0);
                });
            });
        });
    });
}

/// The suite's app switcher in the web start screen's top bar: a 3 x 3 dot button that opens a menu of
/// the KrubOffice apps (this one marked, the others live links that open in a new tab) and a link to kruboffice.app.
fn app_switcher(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("KrubOffice apps")));
    let resp = resp.on_hover_text(tl!("KrubOffice apps"));
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    for (dx, dy) in (0..9).map(|i| ((i % 3) as f32 - 1.0, (i / 3) as f32 - 1.0)) {
        ui.painter().circle_filled(pos2(r.center().x + dx * 6.0, r.center().y + dy * 6.0), 1.8, t.icon);
    }
    egui::Popup::menu(&resp).align(egui::RectAlign::BOTTOM_END).show(|ui| {
        ui.set_min_width(268.0);
        ui.label(egui::RichText::new("KrubOffice").font(regular(11.5)).color(t.text_dim));
        let (navy, teal, coral) =
            (egui::Color32::from_rgb(0x1e, 0x3a, 0x8a), egui::Color32::from_rgb(0x0f, 0x6e, 0x56), egui::Color32::from_rgb(0x9a, 0x3a, 0x1f));
        app_switcher_row(ui, "ข", navy, tl!("KhianKrub"), tl!("Documents"), Some(tl!("Here")), false);
        if app_switcher_row(ui, "ค", teal, tl!("KhitKrub"), tl!("Spreadsheets"), None, true) {
            ui.ctx().open_url(egui::OpenUrl::new_tab("https://khitkrub.app/"));
            ui.close();
        }
        if app_switcher_row(ui, "พ", coral, tl!("PhutKrub"), tl!("Presentations"), None, true) {
            ui.ctx().open_url(egui::OpenUrl::new_tab("https://phutkrub.app/"));
            ui.close();
        }
        ui.separator();
        if ui.button(tl!("See all apps at kruboffice.app")).clicked() {
            ui.ctx().open_url(egui::OpenUrl::same_tab("https://kruboffice.app/"));
            ui.close();
        }
    });
}

/// One app in the switcher: its tile (a gold Thai letter on the app's colour), name and kind. `badge`
/// marks the app you are in; an app that is neither this one nor `live` (not out yet) draws faded.
/// True when a live app other than this one is clicked.
fn app_switcher_row(ui: &mut Ui, letter: &str, tile: egui::Color32, name: &str, kind: &str, badge: Option<&str>, live: bool) -> bool {
    let t = Tokens::get(ui.ctx());
    let faded = badge.is_none() && !live;
    let fade = if faded { 0.45 } else { 1.0 };
    let sense = if live { Sense::click() } else { Sense::hover() };
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(268.0), 44.0), sense);
    let kind_of_widget = if live { egui::WidgetType::Button } else { egui::WidgetType::Label };
    resp.widget_info(|| egui::WidgetInfo::labeled(kind_of_widget, true, format!("{name}, {kind}")));
    if badge.is_some() || resp.hovered() {
        ui.painter().rect_filled(r, 8.0, t.hover);
    }
    let tile_rect = Rect::from_min_size(pos2(r.min.x + 6.0, r.min.y + 7.0), vec2(30.0, 30.0));
    ui.painter().rect_filled(tile_rect, 8.0, tile.gamma_multiply(fade));
    let gold = egui::Color32::from_rgb(0xf2, 0xb7, 0x05).gamma_multiply(fade);
    ui.painter().text(tile_rect.center() + vec2(0.0, 1.0), Align2::CENTER_CENTER, letter, semibold(17.0), gold);
    let x = tile_rect.max.x + 10.0;
    let name_color = if faded { t.text_dim } else { t.text };
    ui.painter().text(pos2(x, r.min.y + 15.0), Align2::LEFT_CENTER, name, medium(13.5), name_color);
    ui.painter().text(pos2(x, r.min.y + 31.0), Align2::LEFT_CENTER, kind, regular(11.5), t.text_dim);
    if let Some(badge) = badge {
        let galley = ui.painter().layout_no_wrap(badge.to_string(), regular(11.0), egui::Color32::WHITE);
        let pill = Rect::from_center_size(pos2(r.max.x - 10.0 - galley.size().x / 2.0 - 8.0, r.center().y), galley.size() + vec2(16.0, 4.0));
        ui.painter().rect_filled(pill, pill.height() / 2.0, t.accent);
        ui.painter().galley(pill.center() - galley.size() / 2.0, galley, egui::Color32::WHITE);
    }
    live && resp.clicked()
}

/// Web Home: the recovery drafts kept in this browser. Click a row to keep writing; its ⋮ (or a
/// right-click) downloads a copy or discards it, after asking.
/// `new_tab`: on the web start screen, keeping writing opens the draft in a new tab
/// (`./?draft=<id>`) and leaves the start screen as it is.
fn drafts_list(app: &mut WordApp, ui: &mut Ui, new_tab: bool) {
    let t = Tokens::get(ui.ctx());
    let Some(recovery) = &app.services.browser_recovery else { return };
    let drafts = (recovery.drafts)(&app.ui.language);
    let width = ui.available_width().min(700.0);
    ui.label(egui::RichText::new(tl!("Drafts on this device")).font(semibold(16.0)));
    let note = if app.services.google_drive.is_some() {
        tl!("Kept only in this browser on this device. Save to Google Drive or download a copy to keep it anywhere else.")
    } else {
        tl!("Kept only in this browser on this device. Download a copy to keep it anywhere else.")
    };
    ui.label(egui::RichText::new(note).font(regular(12.0)).color(t.text_dim));
    if (recovery.fragile)() {
        let warning = tl!(
            "This browser may remove drafts by itself: when it runs short of space, after a long time without a visit, or when a private window closes."
        );
        ui.label(egui::RichText::new(warning).font(regular(12.0)).color(t.text_dim));
    }
    ui.add_space(6.0);
    if !app.ui.local_recovery {
        ui.label(egui::RichText::new(tl!("Recovery is off. Turn it on in File › Options › General.")).color(t.text_dim));
        return;
    }
    let Some(drafts) = drafts else {
        ui.label(egui::RichText::new(tl!("Loading drafts…")).color(t.text_dim));
        return;
    };
    if drafts.is_empty() {
        ui.label(
            egui::RichText::new(tl!("Documents you write here are kept in this browser, so you can pick up where you left off.")).color(t.text_dim),
        );
        return;
    }
    let confirm_id = egui::Id::new("backstage_discard_draft");
    let confirm = ui.ctx().data(|d| d.get_temp::<String>(confirm_id));
    let mut action = None;
    for draft in &drafts {
        // Every row is a draft until it is discarded, saved elsewhere or not.
        let label = format!("{} ({})", draft.name, tl!("draft"));
        let (r, resp) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
        let more_rect = Rect::from_center_size(pos2(r.max.x - 20.0, r.center().y), vec2(26.0, 26.0));
        let more = ui.interact(more_rect, resp.id.with("more"), Sense::click());
        more.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("More options")));
        if ui.rect_contains_pointer(r) {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        if more.hovered() {
            ui.painter().rect_filled(more_rect, 4.0, t.pressed);
        }
        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(22.0, 22.0)), "printLayout", t.icon, t.accent);
        let name_width = width - 56.0;
        let name = ui.painter().layout_no_wrap(label.clone(), medium(13.0), t.text);
        let clip = Rect::from_min_size(pos2(r.min.x + 40.0, r.min.y), vec2(name_width.max(0.0), r.height()));
        ui.painter().with_clip_rect(clip).galley(pos2(r.min.x + 40.0, r.min.y + 13.0 - name.size().y / 2.0), name, t.text);
        ui.painter().text(pos2(r.min.x + 40.0, r.min.y + 29.0), Align2::LEFT_CENTER, &draft.saved, regular(11.0), t.text_dim);
        // Three dots, drawn: the ⋮ menu.
        for dy in [-5.0, 0.0, 5.0] {
            ui.painter().circle_filled(pos2(more_rect.center().x, more_rect.center().y + dy), 1.6, t.icon);
        }
        let menu = |ui: &mut Ui, action: &mut Option<(crate::DraftAction, String)>| {
            if ui.button(tl!("Keep writing")).clicked() {
                *action = Some((crate::DraftAction::Open, draft.id.clone()));
                ui.close();
            }
            if ui.button(tl!("Download a copy")).clicked() {
                *action = Some((crate::DraftAction::Download, draft.id.clone()));
                ui.close();
            }
            ui.separator();
            if ui.button(tl!("Discard draft…")).clicked() {
                ui.ctx().data_mut(|d| d.insert_temp(confirm_id, draft.id.clone()));
                ui.close();
            }
        };
        // Opens leftwards from the ⋮ at the row's end, so it stays inside the window.
        egui::Popup::menu(&more).align(egui::RectAlign::BOTTOM_END).show(|ui| menu(ui, &mut action));
        resp.context_menu(|ui| menu(ui, &mut action));
        if resp.clicked() && !more.clicked() {
            action = Some((crate::DraftAction::Open, draft.id.clone()));
        }
        if confirm.as_deref() == Some(draft.id.as_str()) {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(40.0);
                ui.label(tl!("Permanently discard this recovery draft and its previous copy?"));
                if ui.button(tl!("Discard draft")).clicked() {
                    action = Some((crate::DraftAction::Discard, draft.id.clone()));
                    ui.ctx().data_mut(|d| d.remove::<String>(confirm_id));
                }
                if ui.button(tl!("Cancel")).clicked() {
                    ui.ctx().data_mut(|d| d.remove::<String>(confirm_id));
                }
            });
        }
    }
    if let Some((what, id)) = action {
        if new_tab && matches!(what, crate::DraftAction::Open) {
            app.canvas.open_url = Some(format!("./?draft={}", url_component(&id)));
        } else if let Some(recovery) = &app.services.browser_recovery {
            (recovery.act)(what, &id);
        }
    }
}

/// `text` as a URL query value: letters, digits and `-_.~` stay, every other byte becomes `%XX`.
fn url_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// New: the whole template gallery with a search box; no Recent list (that is on Home and Open).
fn new_page(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    heading(ui, "New");
    let id = egui::Id::new("backstage_template_search");
    let mut query = ui.ctx().data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let search = ui.add(egui::TextEdit::singleline(&mut query).hint_text(tl!("Search for templates")).desired_width(360.0));
    if search.changed() {
        ui.ctx().data_mut(|d| d.insert_temp(id, query.clone()));
    }
    ui.add_space(18.0);
    let found = matching_templates(&query);
    if found.is_empty() {
        ui.label(egui::RichText::new(tl!("No templates match your search.")).color(t.text_dim));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
        for (label, template) in found {
            template_tile(ui, app, label, template, false);
        }
    });
}

/// Recent documents: click one to open it. Each row has an ✕ (shown while pointed at) and a
/// right-click menu to take it off the list; Clear all empties the list. None of these touch the
/// files themselves.
fn open_list(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width().min(700.0);
    ui.allocate_ui_with_layout(vec2(width, 24.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.label(egui::RichText::new(tl!("Recent")).font(semibold(16.0)));
        if !app.ui.recent.is_empty() && app.ui.recent_shown() > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(tl!("Clear all")).on_hover_text(tl!("Take every document off this list. The files stay where they are.")).clicked() {
                    let _ = app.run("ui.recentClear", json!({}));
                }
            });
        }
    });
    ui.add_space(6.0);
    if !app.ui.remember_recent {
        ui.label(
            egui::RichText::new(tl!("KhianKrub doesn't remember the documents you open. Turn it on in File › Options › Advanced.")).color(t.text_dim),
        );
        return;
    }
    if app.ui.recent.is_empty() {
        ui.label(egui::RichText::new(tl!("Documents you open will show up here.")).color(t.text_dim));
    }
    for p in app.ui.recent.iter().take(app.ui.recent_shown()).cloned().collect::<Vec<_>>() {
        let name = std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(p.clone());
        let (r, resp) = ui.allocate_exact_size(vec2(width, 40.0), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name));
        // The ✕ at the row's end, while the row is pointed at.
        let x_rect = Rect::from_center_size(pos2(r.max.x - 20.0, r.center().y), vec2(26.0, 26.0));
        let hovered = ui.rect_contains_pointer(r);
        let x_resp = hovered.then(|| {
            let x = ui.interact(x_rect, resp.id.with("remove"), Sense::click());
            x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Remove from list")));
            x.on_hover_text(tl!("Remove from list"))
        });
        if hovered {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(22.0, 22.0)), "printLayout", t.icon, t.accent);
        ui.painter().text(pos2(r.min.x + 40.0, r.min.y + 13.0), Align2::LEFT_CENTER, &name, medium(13.0), t.text);
        ui.painter().text(pos2(r.min.x + 40.0, r.min.y + 29.0), Align2::LEFT_CENTER, &p, regular(11.0), t.text_dim);
        let mut remove = false;
        if let Some(x) = &x_resp {
            if x.hovered() {
                ui.painter().rect_filled(x_rect, 4.0, t.pressed);
            }
            icons::paint(ui.painter(), x_rect.shrink(6.0), "close", t.icon, t.accent);
            remove = x.clicked();
        }
        resp.context_menu(|ui| {
            if ui.button(tl!("Open")).clicked() {
                let _ = app.run("file.open", json!({"path": p}));
                app.ui.backstage = false;
                app.canvas.want_focus = true;
                ui.close();
            }
            if ui.button(tl!("Remove from list")).clicked() {
                let _ = app.run("ui.recentRemove", json!({"path": p}));
                ui.close();
            }
            ui.separator();
            if ui.button(tl!("Clear all")).clicked() {
                let _ = app.run("ui.recentClear", json!({}));
                ui.close();
            }
        });
        if remove {
            let _ = app.run("ui.recentRemove", json!({"path": p}));
        } else if resp.clicked() {
            let _ = app.run("file.open", json!({"path": p}));
            app.ui.backstage = false;
            app.canvas.want_focus = true;
        }
    }
}

fn open_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Open");
    places(ui, |ui| {
        place_card(ui, "device", tl!("This device"), None, |ui| {
            #[cfg(target_arch = "wasm32")]
            let (about, browse) = (tl!("Choose a file stored on this computer or iPad."), tl!("Browse files on this device…"));
            #[cfg(not(target_arch = "wasm32"))]
            let (about, browse) = (tl!("Choose a file stored on this computer."), tl!("Browse…"));
            card_text(ui, about);
            if crate::widgets::icon_text_button(ui, "open", browse, medium(14.0)).clicked() {
                let _ = app.run("ui.openFileDialog", json!({}));
            }
            // Linux and the BSDs (#576): the system dialog needs a file chooser portal or zenity; when it
            // doesn't show up, WordCraft's own browser is always one click away.
            #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
            if ui
                .button(egui::RichText::new(tl!("Browse in KhianKrub…")).font(medium(14.0)))
                .on_hover_text(tl!("Open a file with KhianKrub's own browser when the system file dialog doesn't appear."))
                .clicked()
            {
                let _ = app.run("ui.fileBrowser", json!({}));
            }
        });
        drive_card(app, ui, false);
    });
    ui.add_space(18.0);
    open_list(app, ui);
}

/// Save As: one card per place. It has no recent-documents list: clicking one of those opens it,
/// which is not what Save As is for. A document opened from Google Drive shows Drive first.
fn save_as_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, if cfg!(target_arch = "wasm32") { "Save" } else { "Save As" });
    let from_drive = app.services.google_drive.as_ref().is_some_and(|d| (d.linked_name)(app.session.document_id()).is_some());
    places(ui, |ui| {
        if from_drive {
            drive_card(app, ui, true);
        }
        place_card(ui, "device", tl!("This device"), None, |ui| {
            #[cfg(target_arch = "wasm32")]
            let (about, icon, browse) = (tl!("Download a copy to this computer or iPad."), "download", tl!("Download a copy…"));
            #[cfg(not(target_arch = "wasm32"))]
            let (about, icon, browse) = (tl!("Choose where to save on this computer."), "saveAs", tl!("Browse…"));
            card_text(ui, about);
            if crate::widgets::icon_text_button(ui, icon, browse, medium(14.0)).clicked() {
                app.save_as_dialog();
            }
        });
        if !from_drive {
            drive_card(app, ui, true);
        }
    });
}

/// The places to open from or save to, side by side, wrapping to a column when narrow.
fn places(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
        add(ui);
    });
}

const PLACE_CARD_WIDTH: f32 = 330.0;

/// A place card: an icon on a tinted tile, a title with an optional status chip, then `body`.
fn place_card(ui: &mut Ui, icon: &str, title: &str, chip: Option<&str>, body: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::new().stroke(Stroke::new(1.0, t.border_strong)).corner_radius(10.0).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
        ui.set_width(PLACE_CARD_WIDTH);
        ui.set_min_height(160.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let (tile, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
            ui.painter().rect_filled(tile, 8.0, t.accent.linear_multiply(0.22));
            icons::paint(ui.painter(), tile.shrink(8.0), icon, t.icon, t.accent);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).font(semibold(16.0)));
                if let Some(chip) = chip {
                    egui::Frame::new()
                        .fill(t.accent.linear_multiply(0.22))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(6, 1))
                        .show(ui, |ui| ui.label(egui::RichText::new(chip).font(regular(12.0))));
                }
            });
            body(ui);
        });
    });
}

fn card_text(ui: &mut Ui, text: impl Into<String>) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(text.into()).font(regular(13.0)).color(t.text_dim));
}

/// The Google Drive card on Open (`save` false) or Save As (`save` true).
fn drive_card(app: &mut WordApp, ui: &mut Ui, save: bool) {
    let Some(state) = app.services.google_drive.as_ref().map(|drive| (drive.status)()) else { return };
    let connected = state == "Connected";
    let connect = |app: &WordApp| {
        if let Some(drive) = &app.services.google_drive {
            (drive.connect)();
        }
    };
    place_card(ui, "cloud", "Google Drive", connected.then(|| tl!("Connected")), |ui| {
        if !connected {
            card_text(
                ui,
                if save { tl!("Save to your Google Drive.") } else { tl!("Open a Word document from your Google Drive and save it back there.") },
            );
            card_text(ui, tl!(state));
            if matches!(state, "Not connected" | "Click Connect to sign in") && ui.button(tl!("Connect Google Drive")).clicked() {
                connect(app);
            }
            // The sign-in window may not have opened (a blocked pop-up) or was closed unanswered:
            // start it again rather than leaving the page stuck until a reload.
            if state == "Signing in…" && ui.button(tl!("Try signing in again")).clicked() {
                connect(app);
            }
            return;
        }
        if save {
            drive_save(app, ui);
        } else {
            drive_open(app, ui);
        }
        if let Some(drive) = &app.services.google_drive
            && let Some(disconnect) = &drive.disconnect
            && ui.button(tl!("Disconnect Google Drive")).clicked()
        {
            disconnect();
        }
    });
}

fn drive_open(app: &mut WordApp, ui: &mut Ui) {
    let Some(drive) = &app.services.google_drive else { return };
    card_text(ui, tl!("Open a Word document from your Google Drive and save it back there."));
    if crate::widgets::icon_text_button(ui, "cloud", tl!("Choose from Google Drive…"), medium(14.0)).clicked() {
        (drive.open)(app.session.document_id(), app.session.rev());
    }
    if let Some(files) = (drive.files)() {
        ui.label(tl!("Files available to KhianKrub (up to 100):"));
        if files.is_empty() {
            ui.label(tl!("No available Word documents found."));
        }
        for (id, name) in files {
            if ui.button(name).clicked() {
                (drive.select_file)(id, app.session.document_id(), app.session.rev());
            }
        }
    }
}

fn drive_save(app: &mut WordApp, ui: &mut Ui) {
    let Some(drive) = &app.services.google_drive else { return };
    let docx = |app: &mut WordApp, name: &str| {
        if !name.to_ascii_lowercase().ends_with(".docx") {
            app.status(tl!("Google Drive copies currently support .docx documents only."));
            return None;
        }
        let password = app.session.password.as_ref().map(wordcraft_engine::Password::as_str);
        let opts = wordcraft_engine::cmd::file::export_options(&app.session, &json!({}));
        wordcraft_engine::io::save_bytes_with(name, &app.session.doc, password, &opts).map_err(|error| app.status(error)).ok()
    };
    let linked = (drive.linked_name)(app.session.document_id());
    match &linked {
        Some(name) => {
            card_text(ui, crate::i18n::fmt(tl!("Opened from Google Drive: {name}"), &[("name", name)]));
            card_text(ui, tl!("Replaces the file on Google Drive. Drive keeps earlier versions for a while."));
        }
        None => card_text(ui, tl!("Each save creates a new file. Changes remain unsaved in the editor.")),
    }
    let mut over = false;
    let mut copy = false;
    ui.horizontal_wrapped(|ui| {
        if linked.is_some() {
            over = crate::widgets::icon_text_button(ui, "save", tl!("Save to this file on Google Drive"), medium(14.0)).clicked();
        }
        copy = ui.button(egui::RichText::new(tl!("Save a new copy to Google Drive")).font(medium(14.0))).clicked();
    });
    if let (true, Some(linked)) = (over, &linked) {
        if let Err(error) = app.save_to_drive(linked) {
            app.status(error);
        }
    } else if copy {
        let name = app.default_save_name();
        if let Some(bytes) = docx(app, &name)
            && let Some(drive) = &app.services.google_drive
        {
            (drive.save_copy)(name, bytes);
        }
    }
}

fn info_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Info");
    let info = app.session.run("file.info", &json!({})).unwrap_or_default();
    ui.columns(2, |cols| {
        let ui = &mut cols[0];
        ui.label(egui::RichText::new(tl!("Properties")).font(semibold(15.0)));
        // The fields edit a copy of the document's properties that is read afresh every frame,
        // so each change goes into the document as it is typed (#260): waiting for the field to
        // lose focus dropped the keystrokes, and leaving File with Escape never loses it at all.
        // The keystrokes of one visit to a field are a single undo step.
        let mut props = app.session.doc.core.clone();
        let mut edited = None;
        let mut focused = None;
        egui::Grid::new("props").num_columns(2).spacing(vec2(12.0, 6.0)).show(ui, |ui| {
            for (l, v) in [("Title", &mut props.title), ("Subject", &mut props.subject), ("Author", &mut props.creator), ("Keywords", &mut props.keywords), ("Category", &mut props.category)] {
                let label = ui.label(tl!(l));
                let r = ui.add(egui::TextEdit::singleline(v).id_salt(("info-prop", l))).labelled_by(label.id);
                if r.changed() {
                    edited = Some(l);
                }
                if r.has_focus() {
                    focused = Some(l);
                }
                ui.end_row();
            }
        });
        if let Some(field) = edited {
            if app.info_editing == Some((field, app.session.rev())) {
                app.session.join_next_undo();
            }
            let ok = app.run("file.properties", json!({"title": props.title, "subject": props.subject, "author": props.creator, "keywords": props.keywords, "category": props.category})).is_ok();
            app.info_editing = ok.then(|| (field, app.session.rev()));
        }
        if focused != app.info_editing.map(|(f, _)| f) {
            app.info_editing = None;
        }
        // Protect Document › Encrypt with Password (#55).
        ui.add_space(18.0);
        ui.label(egui::RichText::new(tl!("Protect Document")).font(semibold(15.0)));
        let encrypted = info.get("encrypted").and_then(|v| v.as_bool()).unwrap_or(false);
        ui.label(if encrypted {
            tl!("A password is required to open this document.")
        } else {
            tl!("Anyone can open this document. Encrypt it with a password to keep it private.")
        });
        ui.horizontal(|ui| {
            let label = if encrypted { tl!("Change Password…") } else { tl!("Encrypt with Password…") };
            if ui.button(egui::RichText::new(label).font(medium(13.5))).clicked() {
                let _ = app.run("file.encrypt", json!({}));
            }
            if encrypted && ui.button(egui::RichText::new(tl!("Remove Password")).font(medium(13.5))).clicked() {
                let _ = app.run("file.encrypt", json!({"password": null}));
            }
        });
        let ui = &mut cols[1];
        ui.label(egui::RichText::new(tl!("Statistics")).font(semibold(15.0)));
        for (l, k) in [("Pages", "pages"), ("Words", "words"), ("Paragraphs", "paragraphs"), ("Sections", "sections"), ("Comments", "comments")] {
            ui.label(format!("{}: {}", tl!(l), info.get(k).map(|v| v.to_string()).unwrap_or_default()));
        }
        let path = info.get("path").and_then(|v| v.as_str()).unwrap_or(tl!("Not saved yet"));
        ui.label(crate::i18n::fmt(tl!("Location: {path}"), &[("path", path)]));
    });
}

fn export_page(app: &mut WordApp, ui: &mut Ui) {
    if app.ui.backstage_page == "print" {
        heading(ui, "Print");
        // Print Markup (a print setting, as in Word): the comments beside each page.
        let mut markup = app.session.prefs.print_markup;
        if ui
            .checkbox(&mut markup, tl!("Print Markup"))
            .on_hover_text(tl!(
                "Comments beside each page and tracked changes in PDFs, prints and images; a comment list at the end of web pages, Markdown and text."
            ))
            .changed()
        {
            let _ = app.run("file.printMarkup", json!({"value": markup}));
        }
        ui.add_space(12.0);
        crate::print_page::show(app, ui);
        return;
    }
    heading(ui, "Export");
    ui.label(tl!("Save a copy in another format. Printing goes through a PDF you can print from any viewer."));
    ui.add_space(12.0);
    crate::print_page::export_formats(app, ui);
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use serde_json::json;

    use super::{matching_templates, url_component};
    use crate::{Services, WordApp};

    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_app() -> (WordApp, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "wordcraft-recovery-ui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let id = "0123456789abcdef0123456789abcdef";
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("snapshot-00000000000000000001.json"),
            serde_json::to_vec(&json!({
                "version": 1, "generation": 1, "info": {"id": id, "name": "Synthetic draft", "savedAt": "2026-10-11T10:42:00Z"},
                "document": wordcraft_doc::Document::from_text("Recovered synthetic work")
            }))
            .unwrap(),
        )
        .unwrap();
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        app.session.enable_recovery(root.clone(), std::sync::Arc::new(|| {})).unwrap();
        (app, root)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_harness(app: WordApp) -> egui_kittest::Harness<'static, WordApp> {
        egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn settle_recovery(h: &mut egui_kittest::Harness<'_, WordApp>) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            h.run_steps(2);
            if h.state().session.recovery_ready() && !h.state().session.recovery_busy() {
                break;
            }
            assert!(std::time::Instant::now() < end);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // Worker completion is a state boundary; egui needs subsequent frames to finish
        // font installation/layout and expose the resulting buttons to AccessKit.
        h.run_steps(4);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_page_keeps_dismissed_copies_and_confirms_discard() {
        let (app, root) = recovery_app();
        let mut h = recovery_harness(app);
        settle_recovery(&mut h);
        assert!(h.state().ui.backstage && h.state().ui.backstage_page == "recover");
        h.get_by_label("Continue without restoring").click();
        h.run_steps(4);
        assert!(!h.state().ui.backstage);
        assert_eq!(h.state().session.recovery_documents().len(), 1);
        h.state_mut().run("file.recover", json!({})).unwrap();
        h.run_steps(4);
        h.get_by_label("Discard…").click();
        h.run_steps(4);
        assert!(matches!(h.state().dialog, Some(crate::dialogs::Dialog::DiscardRecovery { .. })));
        h.get_by_label("Cancel").click();
        h.run_steps(4);
        assert_eq!(h.state().session.recovery_documents().len(), 1);
        h.get_by_label("Discard…").click();
        h.run_steps(4);
        h.get_by_label("Discard").click();
        settle_recovery(&mut h);
        assert!(h.state().session.recovery_documents().is_empty());
        assert!(h.query_by_label("No recovery copies.").is_some());
        drop(h);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn recovery_restore_uses_save_changes_and_resets_autosave_permission() {
        let (app, root) = recovery_app();
        let mut h = recovery_harness(app);
        settle_recovery(&mut h);
        h.state_mut().execute("text.insert", json!({"text": "Current unsaved work"})).unwrap();
        h.state_mut().autosave_path = Some(root.join("must-not-write.docx"));
        h.state_mut().run("file.recover", json!({})).unwrap();
        h.run_steps(4);
        assert_eq!(h.state().session.recovery_documents().len(), 1);
        h.get_by_label("Restore").click();
        h.run_steps(4);
        assert!(matches!(h.state().dialog, Some(crate::dialogs::Dialog::SaveChanges { .. })));
        h.get_by_label("Cancel").click();
        h.run_steps(4);
        assert_eq!(h.state().session.doc.plain_text(wordcraft_doc::StoryRef::Body), "Current unsaved work");
        h.get_by_label("Restore").click();
        h.run_steps(4);
        h.get_by_label("Don't Save").click();
        settle_recovery(&mut h);
        assert_eq!(h.state().session.doc.plain_text(wordcraft_doc::StoryRef::Body), "Recovered synthetic work");
        assert!(h.state().session.dirty && h.state().session.path.is_none());
        assert!(h.state().autosave_path.is_none());
        assert!(!h.state().ui.backstage);
        assert!(!root.join("must-not-write.docx").exists());
        drop(h);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn template_search_matches_labels_case_insensitively() {
        let ids = |q: &str| matching_templates(q).into_iter().map(|(_, id)| id).collect::<Vec<_>>();
        assert_eq!(ids(""), ["blank", "sample", "letter", "resume", "report"]);
        assert_eq!(ids("  LETTER "), ["letter"]);
        assert_eq!(ids("re"), ["resume", "report"]);
        assert_eq!(ids("résumé"), ["resume"]);
        assert!(ids("no such template").is_empty());
    }

    /// Home and New are different pages (#141): Home has the short template row, the link to New
    /// and Recent; New has every template and no Recent list.
    #[test]
    fn home_and_new_show_different_sections() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label("Recent").is_some(), "Home lists recent documents");
        assert!(h.query_by_label("Letter").is_some(), "Home shows the first templates");
        assert!(h.query_by_label("Report").is_none(), "Home shows only a short row of templates");
        h.get_by_label("More templates →").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "new", "the link opens New");
        assert!(h.query_by_label("Report").is_some(), "New shows every template");
        assert!(h.query_by_label("Recent").is_none(), "New has no Recent list");
    }

    /// Web Home lists the drafts kept in the browser in place of Recent: clicking one asks the
    /// host to open it, the ⋮ menu downloads or discards it (after asking), and storage that is
    /// still loading or empty says so.
    #[test]
    fn web_home_lists_browser_drafts_instead_of_recent() {
        use crate::{BrowserDraft, BrowserRecoveryServices, DraftAction};
        use std::cell::RefCell;
        use std::rc::Rc;
        let drafts = |list: Option<Vec<BrowserDraft>>, fragile: bool| {
            let asked: Rc<RefCell<Vec<(DraftAction, String)>>> = Rc::default();
            let log = asked.clone();
            let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
            a.ui.language = "en".into();
            a.services.browser_recovery = Some(BrowserRecoveryServices {
                status: Box::new(|| "Recovery ready on this device"),
                has_drafts: Box::new(|| false),
                fragile: Box::new(move || fragile),
                open_draft: Box::new(|| None),
                manage: Box::new(|| {}),
                drafts: Box::new(move |_| list.clone()),
                act: Box::new(move |what, id| log.borrow_mut().push((what, id.to_string()))),
            });
            assert!(a.start_screen());
            let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
                |ui, app: &mut WordApp| {
                    let ctx = ui.ctx().clone();
                    app.logic(&ctx);
                    app.ui(ui);
                },
                a,
            );
            h.run_steps(4);
            (h, asked)
        };
        let two = vec![
            BrowserDraft { id: "a-1".into(), name: "Monthly report".into(), saved: "29/10/2026, 14:32".into() },
            BrowserDraft { id: "b-2".into(), name: "Letter to Somchai".into(), saved: "28/10/2026, 09:10".into() },
        ];
        let (mut h, asked) = drafts(Some(two.clone()), false);
        assert!(h.query_by_label("Drafts on this device").is_some());
        assert!(h.query_by_label("Recent").is_none(), "the web has no recent files to reopen");
        assert!(h.query_by_label("Letter").is_some(), "the templates stay");
        assert!(h.query_by_label("Hello! What would you like to write?").is_some(), "the web start screen, not File's Home");
        assert!(h.query_by_label("Report").is_some(), "every template, not a short row");
        assert!(h.query_by_label("Open file").is_some() && h.query_by_label("KrubOffice apps").is_some(), "the top bar");
        assert!(h.query_by_label("Print").is_none(), "no File sidebar");
        // Keeping writing opens the draft in a new tab and leaves this start screen as it is.
        let mut opened = Vec::new();
        assert!(h.query_by_label("Monthly report (draft)").is_some(), "each draft says it is one");
        assert!(h.query_by_label_contains("may remove drafts by itself").is_none(), "this browser keeps them");
        h.get_by_label("Letter to Somchai (draft)").click();
        for _ in 0..2 {
            h.step();
            opened.extend(opened_urls(&h));
        }
        assert_eq!(opened, [("./?draft=b-2".to_string(), true)]);
        assert!(asked.borrow().is_empty(), "this tab does not restore the draft");
        assert!(h.query_by_label("Hello! What would you like to write?").is_some());
        h.get_all_by_label("More options").next().unwrap().click();
        h.run_steps(2);
        h.get_by_label("Download a copy").click();
        h.run_steps(2);
        assert_eq!(asked.borrow().last(), Some(&(DraftAction::Download, "a-1".to_string())));
        h.get_all_by_label("More options").next().unwrap().click();
        h.run_steps(2);
        h.get_by_label("Discard draft…").click();
        h.run_steps(2);
        assert_eq!(asked.borrow().len(), 1, "discarding asks first (only the download was asked)");
        h.get_by_label("Discard draft").click();
        h.run_steps(2);
        assert_eq!(asked.borrow().last(), Some(&(DraftAction::Discard, "a-1".to_string())));
        assert!(!h.state().has_document(), "nothing opens until the host restores the draft");

        let (h, _) = drafts(Some(two), true);
        assert!(h.query_by_label_contains("may remove drafts by itself").is_some(), "a browser that may clear them says so");
        let (h, _) = drafts(None, false);
        assert!(h.query_by_label("Loading drafts…").is_some());
        let (h, _) = drafts(Some(Vec::new()), false);
        assert!(h.query_by_label_contains("pick up where you left off").is_some());
    }

    /// The OpenUrl commands (url, new tab) the last frame asked the browser for.
    fn opened_urls<S>(h: &egui_kittest::Harness<'_, S>) -> Vec<(String, bool)> {
        h.output()
            .platform_output
            .commands
            .iter()
            .filter_map(|c| match c {
                egui::OutputCommand::OpenUrl(o) => Some((o.url.clone(), o.new_tab)),
                _ => None,
            })
            .collect()
    }

    /// The web start screen opens a template in a new tab (`./?new=<template>`) and keeps itself.
    #[test]
    fn web_home_templates_open_in_a_new_tab() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.ui.language = "en".into();
        a.services.browser_recovery = Some(crate::BrowserRecoveryServices {
            status: Box::new(|| "Recovery ready on this device"),
            has_drafts: Box::new(|| false),
            fragile: Box::new(|| false),
            open_draft: Box::new(|| None),
            manage: Box::new(|| {}),
            drafts: Box::new(|_| Some(Vec::new())),
            act: Box::new(|_, _| {}),
        });
        assert!(a.start_screen());
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        for (label, id) in [("Letter", "letter"), ("Blank document", "blank")] {
            let mut opened = Vec::new();
            h.get_by_label(label).click();
            for _ in 0..2 {
                h.step();
                opened.extend(opened_urls(&h));
            }
            assert_eq!(opened, [(format!("./?new={id}"), true)], "{label}");
            assert!(!h.state().has_document(), "{label}: the start screen stays; the new tab holds the document");
        }
    }

    /// The top bar's app switcher lists the suite, marks this app and links to kruboffice.app.
    #[test]
    fn web_home_app_switcher_lists_the_suite() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.ui.language = "en".into();
        a.services.browser_recovery = Some(crate::BrowserRecoveryServices {
            status: Box::new(|| "Recovery ready on this device"),
            has_drafts: Box::new(|| false),
            fragile: Box::new(|| false),
            open_draft: Box::new(|| None),
            manage: Box::new(|| {}),
            drafts: Box::new(|_| Some(Vec::new())),
            act: Box::new(|_, _| {}),
        });
        assert!(a.start_screen());
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label("All apps").is_none(), "the text button is gone");
        assert!(h.query_by_label_contains("Coming soon").is_none(), "nothing is coming soon any more");
        h.get_by_label("KrubOffice apps").click();
        h.run_steps(3);
        assert!(h.query_by_label("KhianKrub, Documents").is_some());
        // The other two apps are out: live links that open in a new tab.
        let mut apps = Vec::new();
        for (label, url) in [("KhitKrub, Spreadsheets", "https://khitkrub.app/"), ("PhutKrub, Presentations", "https://phutkrub.app/")] {
            assert_eq!(h.get_all_by_label(label).count(), 1, "{label}");
            h.get_by_label(label).click();
            for _ in 0..2 {
                h.step();
                apps.extend(opened_urls(&h));
            }
            assert_eq!(apps.pop(), Some((url.to_string(), true)), "{label} opens in a new tab");
            h.get_by_label("KrubOffice apps").click();
            h.run_steps(3);
        }
        let mut opened = Vec::new();
        h.get_by_label("See all apps at kruboffice.app").click();
        for _ in 0..2 {
            h.step();
            opened.extend(opened_urls(&h));
        }
        assert_eq!(opened, [("https://kruboffice.app/".to_string(), false)], "the hub opens in this tab");
        assert!(!h.state().has_document());
    }

    #[test]
    fn draft_ids_are_escaped_in_the_url() {
        assert_eq!(url_component("a-1_b.c~"), "a-1_b.c~");
        assert_eq!(url_component("a b/&?=ก"), "a%20b%2F%26%3F%3D%E0%B8%81");
    }

    /// #260: File › Info's Title and Author fields dropped every keystroke (they edited a copy
    /// re-read each frame and only wrote it back when the field lost focus), so a save had no
    /// title or author. Typing now lands in the document as it happens, one undo step per field,
    /// and the browser's Save downloads it with the save stamps advanced (#262).
    #[test]
    fn info_fields_keep_typed_properties_through_a_web_save() {
        use egui::accesskit::Role;
        let got: std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>> = Default::default();
        let sink = got.clone();
        let services = Services {
            download: Some(Box::new(move |n: &str, b: &[u8]| {
                sink.borrow_mut().push((n.to_string(), b.to_vec()));
                Ok(())
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("XYZ")), services);
        a.session.author = "GAMMA".into();
        a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        for (field, text) in [("Title", "ALPHA"), ("Author", "BETA")] {
            h.get_by_role_and_label(Role::TextInput, field).click();
            h.run_steps(2);
            for c in text.chars() {
                h.get_by_role_and_label(Role::TextInput, field).type_text(&c.to_string());
                h.run_steps(2);
            }
            assert_eq!(h.get_by_role_and_label(Role::TextInput, field).value().as_deref(), Some(text), "{field} shows what was typed");
        }
        let core = h.state().session.doc.core.clone();
        assert_eq!((core.title.as_str(), core.creator.as_str()), ("ALPHA", "BETA"));
        assert!(h.state().session.dirty, "the property edits are unsaved changes");

        // Save (web: a download) writes them into docProps/core.xml, with fresh save stamps.
        let a = h.state_mut();
        let created = a.session.doc.core.created.clone();
        a.run("file.save", json!({})).unwrap();
        a.run("file.save", json!({})).unwrap();
        let (name, bytes) = got.borrow_mut().pop().unwrap();
        let back = wordcraft_engine::io::open_bytes(&name, &bytes).unwrap();
        assert_eq!((back.core.title.as_str(), back.core.creator.as_str()), ("ALPHA", "BETA"));
        assert_eq!(back.core.last_modified_by, "GAMMA");
        assert_eq!(back.core.revision, core.revision + 2, "each save advances the revision");
        assert!(created.is_empty() && !back.core.created.is_empty(), "the first save sets the creation time");
        assert!(!back.core.modified.is_empty());
        assert!(!a.session.dirty);

        // Each field's typing is one undo step.
        a.session.run("edit.undo", &json!({})).unwrap();
        assert_eq!((a.session.doc.core.title.as_str(), a.session.doc.core.creator.as_str()), ("ALPHA", ""));
        a.session.run("edit.undo", &json!({})).unwrap();
        assert_eq!(a.session.doc.core.title, "");
    }

    #[test]
    fn save_as_page_shows_browse_and_no_open_list() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "saveAs"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label("Browse…").is_some(), "Save As has Browse button");
        assert!(h.query_by_label("Recent").is_none(), "Save As has no list that opens documents");
    }

    /// File tab, then Back (after Options) to the document: the Backstage closes, the page gets the
    /// keyboard back, and AccessKit (which kittest runs underneath) accepts every frame. The old "<"
    /// at the top of the menu crashed there ("Focused ID … is not in the node list"): a click on the
    /// menu's chrome refocused the page before it was drawn (fixed in `refocus_after_chrome_click`).
    /// The Back button takes focus itself, so this test passes with or without that guard.
    #[test]
    fn leaving_the_backstage_with_back_keeps_accessibility_focus_valid() {
        use egui_kittest::kittest::Queryable;
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.integrated_titlebar = true;
        a.run("text.insert", json!({"text": "Hello"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("File").click();
        h.run_steps(4);
        assert!(h.state().ui.backstage, "the File tab opens the Backstage");
        let options = h.get_by_label("Options").rect();
        let back = h.get_by_label("Back").rect();
        assert!(back.min.y > options.max.y, "Back comes after Options: {options:?} {back:?}");
        h.get_by_label("Back").click();
        h.run_steps(6);
        assert!(!h.state().ui.backstage, "Back returns to the document");
        assert!(h.state().canvas.focused, "the document has the keyboard again");
    }

    #[test]
    fn backstage_save_navigates_to_save_as_when_unnamed() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs", "unnamed Save opens Save As page");
        assert!(h.state().ui.backstage, "backstage remains open");
    }

    /// Save on a document without a file shows the Save As page only: no native dialog as well.
    #[test]
    fn backstage_save_when_unnamed_opens_no_file_dialog() {
        let shown = std::rc::Rc::new(std::cell::Cell::new(0));
        let count = shown.clone();
        let services = Services {
            file_dialog: Some(Box::new(move |_| {
                count.set(count.get() + 1);
                std::sync::mpsc::channel::<crate::FileDialogAnswer>().1
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs");
        assert_eq!(shown.get(), 0, "no native Save As dialog");
        assert!(!h.state().file_dialog_open());
    }

    #[test]
    fn backstage_save_closes_when_saved() {
        let got: std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>> = Default::default();
        let sink = got.clone();
        let services = Services {
            download: Some(Box::new(move |n: &str, b: &[u8]| {
                sink.borrow_mut().push((n.to_string(), b.to_vec()));
                Ok(())
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert!(!h.state().ui.backstage, "successful save closes backstage");
    }

    /// #1124: the side menu had no Close. It sits after Export and closes the document, asking
    /// first when there are unsaved changes; Cancel leaves the Backstage as it was.
    #[test]
    fn backstage_close_asks_then_closes() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("text.insert", json!({"text": "Draft"})).unwrap();
        a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        let export = h.get_by_label("Export").rect().min.y;
        assert!(h.get_by_label("Close").rect().min.y > export, "Close comes after Export");
        h.get_by_label("Close").click();
        h.run_steps(4);
        assert!(matches!(h.state().dialog, Some(crate::dialogs::Dialog::SaveChanges { .. })), "unsaved changes ask first");
        assert!(!h.state().quit_requested);

        h.state_mut().run("ui.saveChanges", json!({"answer": "cancel"})).unwrap();
        h.run_steps(4);
        assert!(!h.state().quit_requested, "Cancel keeps the document open");
        assert!(h.state().ui.backstage && h.state().ui.backstage_page == "info", "Cancel leaves the Backstage as it was");

        h.get_by_label("Close").click();
        h.run_steps(4);
        h.state_mut().run("ui.saveChanges", json!({"answer": "dontSave"})).unwrap();
        assert!(h.state().quit_requested, "Don't Save closes");
    }

    /// A document without unsaved changes closes without a prompt.
    #[test]
    fn backstage_close_on_a_clean_document_closes_at_once() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Close").click();
        h.run_steps(1);
        assert!(h.state().dialog.is_none(), "no prompt");
        assert!(h.state().quit_requested);
    }

    #[test]
    fn google_drive_open_and_save_copy_are_explicit_actions() {
        use std::{cell::RefCell, rc::Rc};
        let opened = Rc::new(RefCell::new(Vec::new()));
        let saved = Rc::new(RefCell::new(Vec::<(String, Vec<u8>)>::new()));
        let open_calls = opened.clone();
        let save_calls = saved.clone();
        let services = Services {
            google_drive: Some(crate::DriveServices {
                status: Box::new(|| "Connected"),
                disconnect: None,
                connect: Box::new(|| {}),
                open: Box::new(move |id, rev| open_calls.borrow_mut().push((id, rev))),
                files: Box::new(|| None),
                select_file: Box::new(|_, _, _| {}),
                save_copy: Box::new(move |name, bytes| save_calls.borrow_mut().push((name, bytes))),
                linked_name: Box::new(|_| None),
                save_over: Box::new(|_, _, _| {}),
            }),
            ..Services::default()
        };
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        app.run("ui.backstage", json!({"value": true, "page": "open"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        assert!(opened.borrow().is_empty() && saved.borrow().is_empty());
        h.get_by_label("Choose from Google Drive…").click();
        h.run_steps(4);
        assert_eq!(opened.borrow().as_slice(), &[(h.state().session.document_id(), h.state().session.rev())]);
        h.get_by_label("Save As").click();
        h.run_steps(4);
        h.get_by_label("Save a new copy to Google Drive").click();
        h.run_steps(4);
        let copies = saved.borrow();
        assert_eq!(copies.len(), 1);
        assert!(copies[0].0.ends_with(".docx"));
        assert!(copies[0].1.starts_with(b"PK"));
    }

    /// A document opened from Google Drive can be saved back over that file from Save As; the
    /// click sends the document's id and revision with its DOCX bytes.
    #[test]
    fn a_document_opened_from_google_drive_saves_back_over_it() {
        use std::{cell::RefCell, rc::Rc};
        let replaced = Rc::new(RefCell::new(Vec::<(u64, u64, Vec<u8>)>::new()));
        let calls = replaced.clone();
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        let linked = app.session.document_id();
        app.services.google_drive = Some(crate::DriveServices {
            status: Box::new(|| "Connected"),
            disconnect: None,
            connect: Box::new(|| {}),
            open: Box::new(|_, _| {}),
            files: Box::new(|| None),
            select_file: Box::new(|_, _, _| {}),
            save_copy: Box::new(|_, _| {}),
            linked_name: Box::new(move |document| (document == linked).then(|| "report.docx".to_string())),
            save_over: Box::new(move |document, revision, bytes| calls.borrow_mut().push((document, revision, bytes))),
        });
        app.run("ui.backstage", json!({"value": true, "page": "saveAs"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        assert!(replaced.borrow().is_empty(), "nothing is sent before the click");
        h.get_by_label("Save to this file on Google Drive").click();
        h.run_steps(4);
        let calls = replaced.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!((calls[0].0, calls[0].1), (h.state().session.document_id(), h.state().session.rev()));
        assert!(calls[0].2.starts_with(b"PK"));
    }

    /// Save (the File menu, Ctrl+S and the toolbar all run `file.save`) of a document opened from
    /// Google Drive replaces that Drive file instead of downloading; other documents still download.
    #[test]
    fn save_goes_back_to_google_drive_for_a_drive_document() {
        use std::{cell::RefCell, rc::Rc};
        let replaced = Rc::new(RefCell::new(Vec::<(u64, u64)>::new()));
        let downloads = Rc::new(RefCell::new(Vec::<String>::new()));
        let (calls, got) = (replaced.clone(), downloads.clone());
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        let linked = app.session.document_id();
        app.services.download = Some(Box::new(move |name: &str, _: &[u8]| {
            got.borrow_mut().push(name.to_string());
            Ok(())
        }));
        app.services.google_drive = Some(crate::DriveServices {
            status: Box::new(|| "Connected"),
            disconnect: None,
            connect: Box::new(|| {}),
            open: Box::new(|_, _| {}),
            files: Box::new(|| None),
            select_file: Box::new(|_, _, _| {}),
            save_copy: Box::new(|_, _| {}),
            linked_name: Box::new(move |document| (document == linked).then(|| "report.docx".to_string())),
            save_over: Box::new(move |document, revision, bytes| {
                assert!(bytes.starts_with(b"PK"));
                calls.borrow_mut().push((document, revision));
            }),
        });
        app.session.path = Some("report.docx".into());
        let v = app.run("file.save", json!({})).unwrap();
        assert_eq!(v["googleDrive"], "report.docx");
        assert_eq!(replaced.borrow().as_slice(), &[(app.session.document_id(), app.session.rev())]);
        assert!(downloads.borrow().is_empty(), "nothing is downloaded");
        // A document that didn't come from Drive downloads as before.
        app.run("file.new", json!({})).unwrap();
        app.run("file.save", json!({})).unwrap();
        assert_eq!(replaced.borrow().len(), 1);
        assert_eq!(downloads.borrow().len(), 1);
    }

    /// A Google sign-in that never answers (a blocked pop-up, a window closed unanswered) can be
    /// started again from the Open page instead of staying on "Signing in…" until a reload.
    #[test]
    fn a_stuck_google_sign_in_can_be_tried_again() {
        use std::{cell::Cell, rc::Rc};
        let connects = Rc::new(Cell::new(0));
        let count = connects.clone();
        let services = Services {
            google_drive: Some(crate::DriveServices {
                status: Box::new(|| "Signing in…"),
                disconnect: None,
                connect: Box::new(move || count.set(count.get() + 1)),
                open: Box::new(|_, _| {}),
                files: Box::new(|| None),
                select_file: Box::new(|_, _, _| {}),
                save_copy: Box::new(|_, _| {}),
                linked_name: Box::new(|_| None),
                save_over: Box::new(|_, _, _| {}),
            }),
            ..Services::default()
        };
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        app.run("ui.backstage", json!({"value": true, "page": "open"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h.get_by_label("Try signing in again").click();
        h.run_steps(2);
        assert_eq!(connects.get(), 1, "the button starts a new sign-in");
    }

    /// The web's File menu has one Save page (both places) and no Save As or Close; the desktop
    /// keeps all three.
    #[test]
    fn the_web_file_menu_has_one_save_and_no_close() {
        let web: Vec<_> = super::pages(true);
        let ids: Vec<&str> = web.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, ["home", "new", "open", "info", "recover", "saveAs", "print", "export", "options"]);
        assert_eq!(web.iter().find(|(id, _)| *id == "saveAs").map(|(_, l)| *l), Some("Save"), "the save page reads Save");
        assert!(!web.iter().any(|(_, l)| *l == "Close" || *l == "Save As"));
        let desktop: Vec<&str> = super::pages(false).iter().map(|(_, l)| *l).collect();
        for label in ["Save", "Save As", "Close"] {
            assert!(desktop.contains(&label), "the desktop keeps {label}");
        }
    }

    #[test]
    fn backstage_save_as_sidebar_opens_save_as_page() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save As").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs");
    }
}
