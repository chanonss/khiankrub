//! Modify Style (Home › Styles, Manage Styles, the Styles pane, Table of Contents › Modify…): a
//! window with a list of pages on the left — General, Font, Indents & Spacing, Alignment — and a
//! preview of the style as the boxes leave it. OK sends everything as one `styles.modify`.
//!
//! The Font page sets Latin and Thai (complex script) text side by side: Word keeps a separate
//! font, size, bold and italic for each, and a Thai heading showed one size in the size box and
//! another on the page when only the Latin ones could be seen.

use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};
use wordcraft_doc::props::{LineSpacing, Underline, VertAlign};
use wordcraft_doc::{Align, CharProps, ParaProps, StyleKind, TextColor};

use crate::WordApp;
use crate::dialogs::buttons;
use crate::options::MeasureExt;
use crate::theme::semibold;
use crate::widgets::SteadyUi;

/// The pages, in order: (id, name). A character style has only the first two.
const PAGES: [(&str, &str); 4] = [("general", "General"), ("font", "Font"), ("indents", "Indents & Spacing"), ("alignment", "Alignment")];

const ALIGNS: [(&str, &str); 6] = [
    ("left", "Left"),
    ("center", "Center"),
    ("right", "Right"),
    ("justify", "Justify"),
    ("distribute", "Distributed"),
    ("thaiDistribute", "Thai Distribution"),
];

/// What the Modify Style window shows. Indents are in the interface unit; spacing in points.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleForm {
    pub page: String,
    pub id: String,
    pub character: bool,
    pub name: String,
    /// Style ids; empty for none.
    pub based_on: String,
    pub next: String,
    // Font: Latin text, then Thai (complex script) text.
    pub font: String,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub font_cs: String,
    pub size_cs: f32,
    pub bold_cs: bool,
    pub italic_cs: bool,
    pub color: String,
    pub underline: bool,
    pub strike: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub small_caps: bool,
    pub caps: bool,
    /// Character spacing, points.
    pub spacing: f32,
    // Indents & Spacing.
    pub left: f32,
    pub right: f32,
    /// Signed first-line indent: positive first line, negative hanging.
    pub first: f32,
    pub before: f32,
    pub after: f32,
    /// `multiple` (lines), `atLeast` or `exactly` (points).
    pub line_rule: String,
    pub line: f32,
    pub contextual: bool,
    // Alignment.
    pub align: String,
    pub rtl: bool,
    /// Paragraph styles to base on and follow with: (id, name).
    #[serde(skip)]
    pub styles: Vec<(String, String)>,
    /// The form as it opened: OK sends only what changed, so values the style inherits (from
    /// the style it's based on) stay inherited.
    #[serde(skip)]
    pub start: Option<Box<StyleForm>>,
}

fn align_name(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
        Align::Justify => "justify",
        Align::Distribute => "distribute",
        Align::ThaiDistribute => "thaiDistribute",
    }
}

fn align_of(s: &str) -> Align {
    match s {
        "center" => Align::Center,
        "right" => Align::Right,
        "justify" => Align::Justify,
        "distribute" => Align::Distribute,
        "thaiDistribute" => Align::ThaiDistribute,
        _ => Align::Left,
    }
}

impl StyleForm {
    /// The form for style `id` as it resolves now; `None` for no such style or a table style
    /// (edited from Table Design).
    pub fn read(app: &WordApp, id: &str) -> Option<StyleForm> {
        let sheet = &app.session.doc.styles;
        let st = sheet.get(id)?;
        let character = match st.kind {
            StyleKind::Paragraph => false,
            StyleKind::Character => true,
            _ => return None,
        };
        let rc = sheet.resolve_char(
            if character { None } else { Some(id) },
            &CharProps { style: if character { Some(id.into()) } else { None }, ..Default::default() },
        );
        let rp = sheet.resolve_para(&ParaProps { style: Some(id.into()), ..Default::default() });
        let k = crate::options::unit().pt_per_unit();
        let (line_rule, line) = match rp.line_spacing {
            LineSpacing::Multiple(x) => ("multiple", x),
            LineSpacing::AtLeast(x) => ("atLeast", x),
            LineSpacing::Exactly(x) => ("exactly", x),
        };
        let kind = if character { StyleKind::Character } else { StyleKind::Paragraph };
        let mut styles: Vec<(String, String)> =
            sheet.styles.iter().filter(|s| s.kind == kind && !s.hidden && s.id != id).map(|s| (s.id.clone(), s.name.clone())).collect();
        styles.sort_by(|a, b| a.1.cmp(&b.1));
        let mut f = StyleForm {
            page: "general".into(),
            id: id.into(),
            character,
            name: st.name.clone(),
            based_on: st.based_on.clone().unwrap_or_default(),
            next: st.next.clone().unwrap_or_default(),
            font: rc.font.clone(),
            size: rc.size,
            bold: rc.bold,
            italic: rc.italic,
            font_cs: rc.font_cs.clone(),
            size_cs: rc.size_cs,
            bold_cs: rc.bold_cs,
            italic_cs: rc.italic_cs,
            color: match rc.color {
                TextColor::Rgb(c) => c.hex(),
                TextColor::Auto => "auto".into(),
            },
            underline: rc.underline != Underline::None,
            strike: rc.strike,
            superscript: rc.vert_align == VertAlign::Superscript,
            subscript: rc.vert_align == VertAlign::Subscript,
            small_caps: rc.small_caps,
            caps: rc.caps,
            spacing: rc.spacing,
            left: rp.indent_left / k,
            right: rp.indent_right / k,
            first: rp.indent_first / k,
            before: rp.space_before,
            after: rp.space_after,
            line_rule: line_rule.into(),
            line,
            contextual: rp.contextual_spacing,
            align: align_name(rp.align).into(),
            rtl: rp.bidi,
            styles,
            start: None,
        };
        f.start = Some(Box::new(f.clone()));
        Some(f)
    }

    /// Whether `pick` differs from the opening form (or there is none to compare with).
    fn changed<T: PartialEq>(&self, pick: impl Fn(&StyleForm) -> T) -> bool {
        self.start.as_deref().is_none_or(|s| pick(s) != pick(self))
    }

    /// The character formatting the form changes in the style. A Thai value goes along when its
    /// Latin one changed, so the engine doesn't copy the Latin one over it.
    pub fn chr(&self) -> CharProps {
        let c = |pick: fn(&StyleForm) -> String| self.changed(pick);
        let thai = |latin: bool, thai: bool| latin || thai;
        let font_cs = thai(self.changed(|f| f.font.clone()), self.changed(|f| f.font_cs.clone()));
        let size_cs = thai(self.changed(|f| f.size.to_bits()), self.changed(|f| f.size_cs.to_bits()));
        let bold_cs = thai(self.changed(|f| f.bold), self.changed(|f| f.bold_cs));
        let italic_cs = thai(self.changed(|f| f.italic), self.changed(|f| f.italic_cs));
        let vert = self.changed(|f| (f.superscript, f.subscript));
        CharProps {
            font: Some(self.font.clone()).filter(|f| !f.trim().is_empty() && c(|f| f.font.clone())),
            size: self.changed(|f| f.size.to_bits()).then_some(self.size.clamp(1.0, 1638.0)),
            bold: self.changed(|f| f.bold).then_some(self.bold),
            italic: self.changed(|f| f.italic).then_some(self.italic),
            font_cs: Some(self.font_cs.clone()).filter(|f| !f.trim().is_empty() && font_cs),
            size_cs: size_cs.then_some(self.size_cs.clamp(1.0, 1638.0)),
            bold_cs: bold_cs.then_some(self.bold_cs),
            italic_cs: italic_cs.then_some(self.italic_cs),
            color: if !c(|f| f.color.clone()) {
                None
            } else if self.color.trim().eq_ignore_ascii_case("auto") {
                Some(TextColor::Auto)
            } else {
                wordcraft_doc::Rgb::parse(&self.color).map(TextColor::Rgb)
            },
            underline: self.changed(|f| f.underline).then_some(if self.underline { Underline::Single } else { Underline::None }),
            strike: self.changed(|f| f.strike).then_some(self.strike),
            vert_align: vert.then_some(if self.superscript {
                VertAlign::Superscript
            } else if self.subscript {
                VertAlign::Subscript
            } else {
                VertAlign::Baseline
            }),
            small_caps: self.changed(|f| f.small_caps).then_some(self.small_caps),
            caps: self.changed(|f| f.caps).then_some(self.caps),
            spacing: self.changed(|f| f.spacing.to_bits()).then_some(self.spacing.clamp(-100.0, 100.0)),
            ..Default::default()
        }
    }

    /// The paragraph formatting the form gives the style (none for a character style).
    pub fn para(&self) -> Option<ParaProps> {
        if self.character {
            return None;
        }
        let k = crate::options::unit().pt_per_unit();
        let line = match self.line_rule.as_str() {
            "atLeast" => LineSpacing::AtLeast(self.line.clamp(0.0, 1584.0)),
            "exactly" => LineSpacing::Exactly(self.line.clamp(0.7, 1584.0)),
            _ => LineSpacing::Multiple(self.line.clamp(0.06, 132.0)),
        };
        let ch = |pick: fn(&StyleForm) -> u32| self.changed(pick);
        Some(ParaProps {
            align: self.changed(|f| f.align.clone()).then_some(align_of(&self.align)),
            bidi: self.changed(|f| f.rtl).then_some(self.rtl),
            indent_left: ch(|f| f.left.to_bits()).then_some(self.left * k),
            indent_right: ch(|f| f.right.to_bits()).then_some(self.right * k),
            indent_first: ch(|f| f.first.to_bits()).then_some(self.first * k),
            space_before: ch(|f| f.before.to_bits()).then_some(self.before.clamp(0.0, 1584.0)),
            space_after: ch(|f| f.after.to_bits()).then_some(self.after.clamp(0.0, 1584.0)),
            line_spacing: self.changed(|f| (f.line_rule.clone(), f.line.to_bits())).then_some(line),
            contextual_spacing: self.changed(|f| f.contextual).then_some(self.contextual),
            ..Default::default()
        })
    }

    /// `styles.modify` parameters for everything in the form.
    pub fn params(&self) -> Value {
        let mut v = json!({
            "style": self.id,
            "name": self.name,
            "basedOn": if self.based_on.is_empty() { Value::Null } else { json!(self.based_on) },
            "chr": serde_json::to_value(self.chr()).unwrap_or(Value::Null),
        });
        if let Some(p) = self.para() {
            v["para"] = serde_json::to_value(p).unwrap_or(Value::Null);
            if !self.next.is_empty() {
                v["next"] = json!(self.next);
            }
        }
        v
    }
}

/// The Modify Style window. Returns true to close.
pub fn body(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) -> bool {
    // A fixed area, so the window keeps its size from page to page (a separator here grew to the
    // whole screen's height).
    const PAGE_H: f32 = 330.0;
    ui.allocate_ui_with_layout(vec2(760.0, PAGE_H), egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.set_height(PAGE_H);
        // The pages.
        ui.vertical(|ui| {
            ui.set_width(150.0);
            for (id, name) in PAGES.iter().take(if f.character { 2 } else { PAGES.len() }) {
                if ui.steady_label(f.page == *id, tl!(name)).clicked() {
                    f.page = id.to_string();
                }
            }
        });
        let x = ui.cursor().min.x + 4.0;
        let top = ui.cursor().min.y;
        ui.painter().vline(x, top..=top + PAGE_H, ui.visuals().widgets.noninteractive.bg_stroke);
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.set_width(590.0);
            match f.page.as_str() {
                "font" => font_page(app, ui, f),
                "indents" if !f.character => indents_page(ui, f),
                "alignment" if !f.character => alignment_page(ui, f),
                _ => general_page(ui, f),
            }
        });
    });
    ui.separator();
    ui.label(egui::RichText::new(tl!("Preview")).small().weak());
    let key = serde_json::to_string(&f.params()).unwrap_or_default();
    let (chr, para, id) = (f.chr(), f.para(), f.id.clone());
    crate::previews::style_edit_preview(
        app,
        ui,
        &id,
        f.character,
        &key,
        &|sheet| {
            if let Some(st) = sheet.get_mut(&id) {
                st.chr.overlay(&chr);
                if let Some(p) = &para {
                    st.para.overlay(p);
                }
            }
        },
        vec2(720.0, 64.0),
    );
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if ok && let Err(e) = app.run("styles.modify", f.params()) {
        app.status(e);
    }
    ok || cancel
}

fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
}

fn general_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Style");
    egui::Grid::new("ms_general").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Name:"));
        ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(300.0));
        ui.end_row();
        let named = |id: &str| f.styles.iter().find(|(s, _)| s == id).map(|(_, n)| n.clone());
        ui.label(tl!("Based on:"));
        let shown = named(&f.based_on).unwrap_or_else(|| tl!("(no style)").to_string());
        egui::ComboBox::from_id_salt("ms_based").selected_text(shown).width(300.0).show_ui(ui, |ui| {
            ui.steady_value(&mut f.based_on, String::new(), tl!("(no style)"));
            for (id, name) in &f.styles {
                ui.steady_value(&mut f.based_on, id.clone(), name);
            }
        });
        ui.end_row();
        if !f.character {
            ui.label(tl!("Style for following paragraph:"));
            let shown = if f.next.is_empty() || f.next == f.id { f.name.clone() } else { named(&f.next).unwrap_or_else(|| f.next.clone()) };
            egui::ComboBox::from_id_salt("ms_next").selected_text(shown).width(300.0).show_ui(ui, |ui| {
                ui.steady_value(&mut f.next, f.id.clone(), &f.name);
                for (id, name) in &f.styles {
                    ui.steady_value(&mut f.next, id.clone(), name);
                }
            });
            ui.end_row();
        }
        ui.label(tl!("Style type:"));
        ui.label(if f.character { tl!("Character") } else { tl!("Paragraph") });
        ui.end_row();
    });
}

fn font_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    let fams = app.previews.families();
    let prev = app.previews.font_preview_fn();
    let mut column = |ui: &mut Ui, title: &str, salt: &str, font: &mut String, size: &mut f32, bold: &mut bool, italic: &mut bool| {
        ui.vertical(|ui| {
            heading(ui, title);
            egui::Grid::new(salt).num_columns(2).spacing(vec2(8.0, 8.0)).show(ui, |ui| {
                ui.label(tl!("Font:"));
                let shown = app.previews.font_shown(font);
                if let Some(pick) = crate::widgets::font_combo(ui, &format!("{salt}_font"), 190.0, &shown, &fams, Some(&*prev))
                    && pick != crate::widgets::ADD_FONT
                    && pick != crate::widgets::LOAD_SYSTEM_FONTS
                {
                    *font = app.previews.font_stored(&pick);
                }
                ui.end_row();
                ui.label(tl!("Size:"));
                ui.add(crate::numfield::NumField::new(size).range(1.0..=1638.0).speed(0.5));
                ui.end_row();
                ui.label("");
                ui.horizontal(|ui| {
                    ui.checkbox(bold, tl!("Bold"));
                    ui.checkbox(italic, tl!("Italic"));
                });
                ui.end_row();
            });
        });
    };
    ui.horizontal_top(|ui| {
        column(ui, "Latin text", "ms_latin", &mut f.font, &mut f.size, &mut f.bold, &mut f.italic);
        ui.add_space(16.0);
        column(ui, "Thai text (complex script)", "ms_thai", &mut f.font_cs, &mut f.size_cs, &mut f.bold_cs, &mut f.italic_cs);
    });
    if ui.small_button(tl!("Thai text like Latin text")).on_hover_text(tl!("Copy the Latin font, size, bold and italic to Thai text")).clicked() {
        f.font_cs = f.font.clone();
        f.size_cs = f.size;
        f.bold_cs = f.bold;
        f.italic_cs = f.italic;
    }
    heading(ui, "Effects");
    egui::Grid::new("ms_effects").num_columns(3).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
        ui.checkbox(&mut f.underline, tl!("Underline"));
        ui.checkbox(&mut f.strike, tl!("Strikethrough"));
        ui.checkbox(&mut f.small_caps, tl!("Small caps"));
        ui.end_row();
        if ui.checkbox(&mut f.superscript, tl!("Superscript")).changed() && f.superscript {
            f.subscript = false;
        }
        if ui.checkbox(&mut f.subscript, tl!("Subscript")).changed() && f.subscript {
            f.superscript = false;
        }
        ui.checkbox(&mut f.caps, tl!("All caps"));
        ui.end_row();
    });
    egui::Grid::new("ms_color").num_columns(2).spacing(vec2(8.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Font color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.color, "Automatic", "auto");
        ui.end_row();
        ui.label(tl!("Character spacing (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.spacing).speed(0.1).range(-20.0..=20.0));
        ui.end_row();
    });
}

fn indents_page(ui: &mut Ui, f: &mut StyleForm) {
    let unit = crate::options::unit();
    fn len(v: &mut f32, unit: wordcraft_geom::Unit) -> crate::numfield::NumField<'_> {
        crate::numfield::NumField::new(v).speed(0.01).range(-22.0..=22.0).measure(unit).max_decimals(2)
    }
    heading(ui, "Indentation");
    egui::Grid::new("ms_indent").num_columns(4).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Left:"));
        ui.add(len(&mut f.left, unit));
        ui.label(tl!("Right:"));
        ui.add(len(&mut f.right, unit));
        ui.end_row();
        // As in Word: Special (none, First line, Hanging) and By; hanging is a negative first line.
        ui.label(tl!("Special:"));
        let special = if f.first > 0.0 {
            "first"
        } else if f.first < 0.0 {
            "hanging"
        } else {
            "none"
        };
        let label = |s: &str| match s {
            "first" => tl!("First line"),
            "hanging" => tl!("Hanging"),
            _ => tl!("(none)"),
        };
        egui::ComboBox::from_id_salt("ms_special").selected_text(label(special)).show_ui(ui, |ui| {
            for s in ["none", "first", "hanging"] {
                if ui.steady_label(s == special, label(s)).clicked() {
                    let by = if f.first == 0.0 { 36.0 / unit.pt_per_unit() } else { f.first.abs() };
                    f.first = match s {
                        "first" => by,
                        "hanging" => -by,
                        _ => 0.0,
                    };
                }
            }
        });
        ui.label(tl!("By:"));
        let mut by = f.first.abs();
        if ui.add_enabled(special != "none", len(&mut by, unit)).changed() {
            f.first = if special == "hanging" { -by.abs() } else { by.abs() };
        }
        ui.end_row();
    });
    heading(ui, "Spacing");
    egui::Grid::new("ms_spacing").num_columns(4).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Before (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.before).range(0.0..=1584.0).speed(0.5));
        ui.label(tl!("Line spacing:"));
        let rule = |s: &str| match s {
            "atLeast" => tl!("At least"),
            "exactly" => tl!("Exactly"),
            _ => tl!("Multiple"),
        };
        let before = f.line_rule.clone();
        egui::ComboBox::from_id_salt("ms_line_rule").selected_text(rule(&f.line_rule)).show_ui(ui, |ui| {
            for s in ["multiple", "atLeast", "exactly"] {
                ui.steady_value(&mut f.line_rule, s.to_string(), rule(s));
            }
        });
        if f.line_rule != before {
            // Lines for Multiple, points for the others: a sensible start for each.
            f.line = if f.line_rule == "multiple" { 1.0 } else { 12.0 };
        }
        ui.end_row();
        ui.label(tl!("After (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.after).range(0.0..=1584.0).speed(0.5));
        ui.label(if f.line_rule == "multiple" { tl!("Lines:") } else { tl!("Points:") });
        ui.add(crate::numfield::NumField::new(&mut f.line).range(0.0..=1584.0).speed(0.05).max_decimals(2));
        ui.end_row();
    });
    ui.checkbox(&mut f.contextual, tl!("Don't add space between paragraphs of the same style"));
}

fn alignment_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Alignment");
    for (v, l) in ALIGNS {
        ui.radio_value(&mut f.align, v.to_string(), tl!(l));
    }
    heading(ui, "Text direction");
    ui.horizontal(|ui| {
        ui.radio_value(&mut f.rtl, false, tl!("Left-to-right"));
        ui.radio_value(&mut f.rtl, true, tl!("Right-to-left"));
    });
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wordcraft_engine::Session;

    use super::StyleForm;
    use crate::{Services, WordApp};

    fn app() -> WordApp {
        WordApp::new(Session::new(wordcraft_doc::Document::from_text("หัวข้อ")), Services::default())
    }

    /// The Font page shows Latin and Thai text apart: Heading 1 with Latin 18 pt and Thai 24 pt
    /// (Modify Style before this changed only the Latin size) shows both; setting the Thai size
    /// to 18 fixes the heading, and only what changed is sent (the Latin font stays inherited).
    #[test]
    fn the_font_page_sets_thai_text_apart() {
        let mut a = app();
        if let Some(h) = a.session.doc.styles.get_mut("Heading1") {
            h.chr.size = Some(18.0);
            h.chr.size_cs = Some(24.0);
        }
        let mut f = StyleForm::read(&a, "Heading1").unwrap();
        assert_eq!((f.size, f.size_cs), (18.0, 24.0), "both sizes show");
        f.size_cs = 18.0;
        let sent = f.params();
        assert_eq!(sent["chr"].get("sizeCs").and_then(|v| v.as_f64()), Some(18.0));
        assert!(sent["chr"].get("font").is_none() && sent["chr"].get("size").is_none(), "unchanged values aren't sent: {sent}");
        assert!(sent.get("para").is_some_and(|p| p.as_object().is_some_and(|o| o.is_empty())), "no paragraph change: {sent}");
        a.run("styles.modify", sent).unwrap();
        let h = a.session.doc.styles.resolve_char(Some("Heading1"), &Default::default());
        assert_eq!((h.size, h.size_cs), (18.0, 18.0));
    }

    /// Indents & Spacing, Alignment and General: a hanging indent, line spacing exactly 20 pt,
    /// Thai Distribution, right-to-left and another base style all reach the style; a style can't
    /// be based on one based on it.
    #[test]
    fn indents_alignment_and_base_reach_the_style() {
        let mut a = app();
        let mut f = StyleForm::read(&a, "Heading2").unwrap();
        let k = crate::options::unit().pt_per_unit();
        f.left = 72.0 / k;
        f.first = -18.0 / k;
        f.line_rule = "exactly".into();
        f.line = 20.0;
        f.align = "thaiDistribute".into();
        f.rtl = true;
        f.based_on = "Normal".into();
        f.next = "Heading3".into();
        a.run("styles.modify", f.params()).unwrap();
        let rp = a.session.doc.styles.resolve_para(&wordcraft_doc::ParaProps { style: Some("Heading2".into()), ..Default::default() });
        assert_eq!((rp.indent_left, rp.indent_first), (72.0, -18.0));
        assert_eq!(rp.line_spacing, wordcraft_doc::props::LineSpacing::Exactly(20.0));
        assert_eq!((rp.align, rp.bidi), (wordcraft_doc::Align::ThaiDistribute, true));
        let st = a.session.doc.styles.get("Heading2").unwrap();
        assert_eq!((st.based_on.as_deref(), st.next.as_deref()), (Some("Normal"), Some("Heading3")));
        // Heading 3 is based on Heading 2 here: Heading 2 can't be based on Heading 3.
        a.run("styles.modify", json!({"style": "Heading3", "basedOn": "Heading2"})).unwrap();
        assert!(a.run("styles.modify", json!({"style": "Heading2", "basedOn": "Heading3"})).is_err());
        // A character style has only General and Font, and sends no paragraph formatting.
        let c = StyleForm::read(&a, "Strong").unwrap();
        assert!(c.character && c.params().get("para").is_none());
    }
}
