//! Define New Multilevel List (Home › Paragraph › Multilevel List) and Track Changes Options
//! (Review › Tracking). Each fills a form from the session and ends in one command
//! (`list.define`, `review.trackingOptions`), so agents get the same result without the dialog.

use crate::widgets::SteadyUi;
use egui::{Sense, Ui, vec2};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};
use wordcraft_doc::section::NumFormat;
use wordcraft_engine::cmd::lists::{LEVELS, caret_list, level_json, levels_of};

use crate::WordApp;
use crate::dialogs::buttons;
use crate::options::MeasureExt;
use crate::theme::{Tokens, semibold};

/// Number styles a level can have: OOXML name and a sample.
const STYLES: [(&str, &str); 13] = [
    ("decimal", "1, 2, 3, …"),
    ("lowerLetter", "a, b, c, …"),
    ("upperLetter", "A, B, C, …"),
    ("lowerRoman", "i, ii, iii, …"),
    ("upperRoman", "I, II, III, …"),
    ("decimalZero", "01, 02, 03, …"),
    ("ordinal", "1st, 2nd, 3rd, …"),
    ("cardinalText", "One, Two, Three, …"),
    ("thaiNumbers", "๑, ๒, ๓, …"),
    ("thaiLetters", "ก, ข, ค, …"),
    ("thaiCounting", "หนึ่ง, สอง, สาม, …"),
    ("bullet", "•"),
    ("none", ""),
];

/// One level of the Define New Multilevel List form. Positions are in the interface unit.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelForm {
    pub text: String,
    pub format: String,
    pub start: u32,
    pub restart: bool,
    /// 1-based level it restarts after.
    pub restart_after: u8,
    /// Linked paragraph style id, empty for none.
    pub style: String,
    pub align: String,
    pub aligned_at: f32,
    pub indent_at: f32,
    pub follow: String,
    pub tab_on: bool,
    pub tab_at: f32,
    pub font: String,
    pub bold: bool,
    pub italic: bool,
    pub legal: bool,
}

impl LevelForm {
    fn read(i: usize, v: &Value) -> LevelForm {
        let k = crate::options::unit().pt_per_unit();
        let s = |key: &str| v.get(key).and_then(Value::as_str).unwrap_or("").to_string();
        let f = |key: &str| v.get(key).and_then(Value::as_f64).map(|x| x as f32 / k).filter(|x| x.is_finite());
        let restart_after = v.get("restartAfter").and_then(Value::as_u64);
        LevelForm {
            text: s("text"),
            format: s("format"),
            start: v.get("start").and_then(Value::as_u64).unwrap_or(1).min(32_767) as u32,
            restart: restart_after != Some(0),
            restart_after: restart_after.filter(|r| *r > 0).map(|r| r.min(8) as u8).unwrap_or(i as u8).max(1),
            style: s("style"),
            align: s("align"),
            aligned_at: f("alignedAt").unwrap_or(0.0),
            indent_at: f("indentAt").unwrap_or(0.0),
            follow: s("follow"),
            tab_on: f("tabAt").is_some(),
            tab_at: f("tabAt").or(f("indentAt")).unwrap_or(0.0),
            font: s("font"),
            bold: v.get("bold").and_then(Value::as_bool).unwrap_or(false),
            italic: v.get("italic").and_then(Value::as_bool).unwrap_or(false),
            legal: v.get("legal").and_then(Value::as_bool).unwrap_or(false),
        }
    }

    /// The level as `list.define` takes it.
    pub fn params(&self, i: usize) -> Value {
        let k = crate::options::unit().pt_per_unit();
        json!({
            "format": self.format,
            "text": self.text,
            "start": self.start,
            "restartAfter": if !self.restart { json!(0) } else if i > 0 && (self.restart_after as usize) < i { json!(self.restart_after) } else { Value::Null },
            "style": if self.style.is_empty() { Value::Null } else { json!(self.style) },
            "align": self.align,
            "alignedAt": self.aligned_at * k,
            "indentAt": self.indent_at * k,
            "follow": self.follow,
            "tabAt": if self.tab_on && self.follow == "tab" { json!(self.tab_at * k) } else { Value::Null },
            "font": if self.font.trim().is_empty() { Value::Null } else { json!(self.font.trim()) },
            "bold": self.bold,
            "italic": self.italic,
            "legal": self.legal,
        })
    }
}

/// A multilevel list saved by name in My Lists: its levels as `list.define` takes them (positions
/// in points, so a change of measurement unit doesn't change it).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedList {
    pub name: String,
    pub levels: Value,
}

/// At most this many saved lists; longer names are cut.
pub const MAX_SAVED_LISTS: usize = 50;
const MAX_LIST_NAME: usize = 64;

/// The saved lists in the settings file: entries without a name or levels are dropped.
pub fn lenient_saved_lists<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<SavedList>, D::Error> {
    let v = Value::deserialize(d)?;
    Ok(v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| serde_json::from_value::<SavedList>(x.clone()).ok())
                .filter(|l| !l.name.trim().is_empty() && l.levels.is_array())
                .take(MAX_SAVED_LISTS)
                .collect()
        })
        .unwrap_or_default())
}

impl SavedList {
    /// How its first three levels number, as the menu shows it (`1. 1.1. 1.1.1.`).
    pub fn sample(&self) -> String {
        let levels = (0..LEVELS).map(|i| LevelForm::read(i, self.levels.get(i).unwrap_or(&Value::Null))).collect();
        let f = ListForm { levels, level: 0, styles: Vec::new(), name: String::new(), save: false };
        (0..3).map(|k| f.sample(k)).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
    }

    /// `list.define` parameters that number the selected paragraphs with it.
    pub fn params(&self) -> Value {
        json!({"levels": self.levels, "name": self.name})
    }
}

/// The Define New Multilevel List form: nine levels and the one being edited.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListForm {
    pub levels: Vec<LevelForm>,
    pub level: usize,
    /// Paragraph styles a level can link to: (id, name).
    #[serde(skip)]
    pub styles: Vec<(String, String)>,
    /// The list's name (kept in the document, and in My Lists when saved).
    pub name: String,
    /// Save in My Lists, for every document.
    pub save: bool,
}

impl ListForm {
    /// Starts from the caret's list, or from a new 1. a. i. list.
    pub fn read(app: &WordApp) -> ListForm {
        let s = &app.session;
        let levels =
            caret_list(s).and_then(|n| levels_of(s, n)).unwrap_or_else(|| wordcraft_doc::numbering::levels_for(wordcraft_doc::ListKind::Numbered));
        let levels = (0..LEVELS).map(|i| LevelForm::read(i, &levels.get(i).map(level_json).unwrap_or(Value::Null))).collect();
        let mut styles: Vec<(String, String)> = s
            .doc
            .styles
            .styles
            .iter()
            .filter(|st| st.kind == wordcraft_doc::StyleKind::Paragraph && !st.hidden)
            .map(|st| (st.id.clone(), st.name.clone()))
            .collect();
        styles.sort_by(|a, b| a.1.cmp(&b.1));
        ListForm { levels, level: 0, styles, name: String::new(), save: false }
    }

    /// `list.define` parameters for the whole list.
    pub fn params(&self) -> Value {
        let levels: Vec<Value> = self.levels.iter().enumerate().map(|(i, l)| l.params(i)).collect();
        let name = self.name.trim();
        if name.is_empty() { json!({"levels": levels}) } else { json!({"levels": levels, "name": name}) }
    }

    /// Save in My Lists (when asked, with a name): a list of the same name is replaced.
    pub fn save_to(&self, saved: &mut Vec<SavedList>) {
        let name: String = self.name.trim().chars().take(MAX_LIST_NAME).collect();
        if !self.save || name.is_empty() {
            return;
        }
        let levels = self.params().get("levels").cloned().unwrap_or(Value::Null);
        saved.retain(|l| l.name != name);
        saved.push(SavedList { name, levels });
        if saved.len() > MAX_SAVED_LISTS {
            saved.remove(0);
        }
    }

    /// The number level `k` shows with every level at its start value, as in the preview.
    pub fn sample(&self, k: usize) -> String {
        let Some(l) = self.levels.get(k) else { return String::new() };
        match l.format.as_str() {
            "none" => return String::new(),
            "bullet" => return l.text.clone(),
            _ => {}
        }
        let mut out = String::new();
        let mut chars = l.text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%'
                && let Some(d) = chars.peek().and_then(|d| d.to_digit(10))
            {
                chars.next();
                let j = (d as usize).saturating_sub(1).min(LEVELS - 1);
                let lv = self.levels.get(j);
                let fmt = if l.legal && j != k { NumFormat::Decimal } else { lv.map(|x| NumFormat::from_ooxml(&x.format)).unwrap_or_default() };
                out.push_str(&fmt.format(lv.map(|x| x.start).unwrap_or(1).max(1)));
            } else {
                out.push(c);
            }
        }
        out
    }
}

/// Define New Multilevel List. Returns true to close.
pub fn define_list(app: &mut WordApp, ui: &mut Ui, f: &mut ListForm) -> bool {
    let unit = crate::options::unit();
    fn len(v: &mut f32, unit: wordcraft_geom::Unit) -> crate::numfield::NumField<'_> {
        crate::numfield::NumField::new(v).speed(0.01).range(-10.0..=22.0).measure(unit).max_decimals(2)
    }
    let heading = |ui: &mut Ui, s: &str| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
    };
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label(tl!("Click level to modify:"));
            for i in 0..LEVELS {
                if ui.steady_label(f.level == i, format!("{}", i + 1)).clicked() {
                    f.level = i;
                }
            }
        });
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(tl!("Preview")).small().weak());
            preview(ui, f);
        });
    });
    let i = f.level.min(LEVELS - 1);
    let styles = f.styles.clone();
    let fams = app.previews.families();
    let font_preview = app.previews.font_preview_fn();
    let Some(l) = f.levels.get_mut(i) else { return true };
    heading(ui, "Number format");
    egui::Grid::new("mll_number").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Enter formatting for number:"));
        ui.text_edit_singleline(&mut l.text);
        ui.end_row();
        ui.label(tl!("Number style for this level:"));
        let current = STYLES.iter().find(|(k, _)| *k == l.format).map(|(_, s)| *s).unwrap_or("");
        egui::ComboBox::from_id_salt("mll_style").selected_text(style_label(current)).show_ui(ui, |ui| {
            for (k, s) in STYLES {
                if ui.steady_label(l.format == k, style_label(s)).clicked() {
                    // A new style puts this level's number in the text if it had none (a bullet
                    // its character).
                    if k == "bullet" {
                        l.text = "•".into();
                    } else if l.format == "bullet" || !l.text.contains('%') {
                        l.text = format!("%{}.", i + 1);
                    }
                    l.format = k.to_string();
                }
            }
        });
        ui.end_row();
        if i > 0 {
            ui.label(tl!("Include level number from:"));
            egui::ComboBox::from_id_salt("mll_include").selected_text("").show_ui(ui, |ui| {
                for k in 0..i {
                    if ui.steady_label(false, crate::i18n::fmt(tl!("Level {n}"), &[("n", &(k + 1).to_string())])).clicked() {
                        l.text.push_str(&format!("%{}", k + 1));
                    }
                }
            });
            ui.end_row();
        }
        ui.label(tl!("Start at:"));
        ui.add(crate::numfield::NumField::new(&mut l.start).range(0..=32_767));
        ui.end_row();
        if i > 0 {
            ui.checkbox(&mut l.restart, tl!("Restart list after:"));
            ui.add_enabled_ui(l.restart, |ui| {
                let shown = crate::i18n::fmt(tl!("Level {n}"), &[("n", &l.restart_after.clamp(1, i as u8).to_string())]);
                egui::ComboBox::from_id_salt("mll_restart").selected_text(shown).show_ui(ui, |ui| {
                    for k in 1..=i as u8 {
                        ui.steady_value(&mut l.restart_after, k, crate::i18n::fmt(tl!("Level {n}"), &[("n", &k.to_string())]));
                    }
                });
            });
            ui.end_row();
        }
        ui.label(tl!("Link level to style:"));
        let shown = styles.iter().find(|(id, _)| *id == l.style).map(|(_, n)| n.clone()).unwrap_or_else(|| tl!("(no style)").to_string());
        egui::ComboBox::from_id_salt("mll_link").selected_text(shown).show_ui(ui, |ui| {
            ui.steady_value(&mut l.style, String::new(), tl!("(no style)"));
            for (id, name) in &styles {
                ui.steady_value(&mut l.style, id.clone(), name);
            }
        });
        ui.end_row();
        ui.label(tl!("Font:"));
        ui.horizontal(|ui| {
            // The installed fonts, as in the ribbon's font box; empty follows the paragraph.
            let shown = if l.font.is_empty() { tl!("(as the paragraph)").to_string() } else { l.font.clone() };
            if let Some(pick) = crate::widgets::font_combo(ui, "mll_font", 170.0, &shown, &fams, Some(&*font_preview))
                && pick != crate::widgets::ADD_FONT
                && pick != crate::widgets::LOAD_SYSTEM_FONTS
            {
                l.font = if pick == tl!("(as the paragraph)") { String::new() } else { pick };
            }
            if !l.font.is_empty() && ui.small_button(tl!("Clear")).on_hover_text(tl!("Use the paragraph's font")).clicked() {
                l.font.clear();
            }
            ui.checkbox(&mut l.bold, tl!("Bold"));
            ui.checkbox(&mut l.italic, tl!("Italic"));
        });
        ui.end_row();
        ui.label("");
        ui.checkbox(&mut l.legal, tl!("Legal style numbering"));
        ui.end_row();
    });
    heading(ui, "Position");
    egui::Grid::new("mll_position").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Number alignment:"));
        ui.horizontal(|ui| {
            for (v, s) in [("left", "Left"), ("center", "Center"), ("right", "Right")] {
                ui.radio_value(&mut l.align, v.to_string(), tl!(s));
            }
        });
        ui.end_row();
        ui.label(tl!("Aligned at:"));
        ui.add(len(&mut l.aligned_at, unit));
        ui.end_row();
        ui.label(tl!("Text indent at:"));
        ui.add(len(&mut l.indent_at, unit));
        ui.end_row();
        ui.label(tl!("Follow number with:"));
        let follow = |k: &str| match k {
            "space" => tl!("Space"),
            "nothing" => tl!("Nothing"),
            _ => tl!("Tab character"),
        };
        egui::ComboBox::from_id_salt("mll_follow").selected_text(follow(&l.follow)).show_ui(ui, |ui| {
            for k in ["tab", "space", "nothing"] {
                ui.steady_value(&mut l.follow, k.to_string(), follow(k));
            }
        });
        ui.end_row();
        ui.add_enabled(l.follow == "tab", egui::Checkbox::new(&mut l.tab_on, tl!("Add tab stop at:")));
        ui.add_enabled(l.follow == "tab" && l.tab_on, len(&mut l.tab_at, unit));
        ui.end_row();
    });
    heading(ui, "Name");
    ui.horizontal(|ui| {
        ui.label(tl!("List name:"));
        ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(200.0).char_limit(MAX_LIST_NAME).hint_text(tl!("Optional")));
    });
    ui.add_enabled(!f.name.trim().is_empty(), egui::Checkbox::new(&mut f.save, tl!("Save in My Lists, for every document")));
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if ok {
        match app.run("list.define", f.params()) {
            Ok(_) => f.save_to(&mut app.ui.saved_lists),
            Err(e) => app.status(e),
        }
    }
    ok || cancel
}

fn style_label(s: &str) -> String {
    match s {
        "" => tl!("None").to_string(),
        "•" => format!("• {}", tl!("Bullet")),
        s => s.to_string(),
    }
}

/// All nine levels, each number at its aligned-at position followed by a grey line of text at
/// its text indent; the level being edited in the text colour.
fn preview(ui: &mut Ui, f: &ListForm) {
    let t = Tokens::get(ui.ctx());
    let row = 19.0;
    let (rect, _) = ui.allocate_exact_size(vec2(300.0, row * LEVELS as f32 + 8.0), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 2.0, egui::Color32::WHITE);
    p.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
    let k = crate::options::unit().pt_per_unit();
    // Positions in points, scaled so 4.5 inches fit.
    let scale = (rect.width() - 16.0) / 324.0;
    let x_of = |v: f32| rect.left() + 8.0 + (v * k * scale).clamp(0.0, rect.width() - 40.0);
    for (i, l) in f.levels.iter().enumerate() {
        let y = rect.top() + 4.0 + row * i as f32 + row / 2.0;
        let on = i == f.level;
        let ink = if on { egui::Color32::BLACK } else { egui::Color32::from_gray(150) };
        let label = f.sample(i);
        let font = egui::FontId::proportional(10.5);
        let galley = p.layout_no_wrap(label, font, ink);
        let w = galley.size().x;
        let x = x_of(l.aligned_at);
        let nx = match l.align.as_str() {
            "center" => x - w / 2.0,
            "right" => x - w,
            _ => x,
        };
        p.galley(egui::pos2(nx, y - galley.size().y / 2.0), galley, ink);
        let tx = match l.follow.as_str() {
            "space" => nx + w + 3.0,
            "nothing" => nx + w,
            _ => x_of(l.indent_at).max(nx + w + 2.0),
        };
        let bar = egui::Rect::from_min_max(egui::pos2(tx, y - 2.0), egui::pos2(rect.right() - 8.0, y + 2.0));
        if bar.width() > 0.0 {
            p.rect_filled(bar, 1.0, if on { egui::Color32::from_gray(120) } else { egui::Color32::from_gray(205) });
        }
    }
}

/// Track Changes Options (Review › Tracking): what markup shows and how revisions look.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackForm {
    pub comments: bool,
    pub ink: bool,
    pub insertions_deletions: bool,
    pub formatting: bool,
    pub balloons: String,
    pub comment_display: String,
    pub insert_mark: String,
    /// Hex, empty = by author.
    pub insert_color: String,
    pub delete_mark: String,
    pub delete_color: String,
    pub changed_lines: String,
    /// Hex, empty = automatic.
    pub changed_lines_color: String,
    pub track_formatting: bool,
}

impl TrackForm {
    pub fn read(app: &WordApp) -> TrackForm {
        let v = wordcraft_engine::cmd::review::tracking_json(&app.session.prefs.markup);
        let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let c = |k: &str| wordcraft_doc::Rgb::parse(&s(k)).map(|c| c.hex()).unwrap_or_default();
        let b = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(true);
        TrackForm {
            comments: b("comments"),
            ink: b("ink"),
            insertions_deletions: b("insertionsDeletions"),
            formatting: b("formatting"),
            balloons: s("balloons"),
            comment_display: s("commentDisplay"),
            insert_mark: s("insertMark"),
            insert_color: c("insertColor"),
            delete_mark: s("deleteMark"),
            delete_color: c("deleteColor"),
            changed_lines: s("changedLines"),
            changed_lines_color: c("changedLinesColor"),
            track_formatting: b("trackFormatting"),
        }
    }

    /// `review.trackingOptions` parameters.
    pub fn params(&self) -> Value {
        let or = |c: &str, auto: &str| if c.is_empty() { auto.to_string() } else { c.to_string() };
        json!({
            "comments": self.comments,
            "ink": self.ink,
            "insertionsDeletions": self.insertions_deletions,
            "formatting": self.formatting,
            "balloons": self.balloons,
            "commentDisplay": self.comment_display,
            "insertMark": self.insert_mark,
            "insertColor": or(&self.insert_color, "byAuthor"),
            "deleteMark": self.delete_mark,
            "deleteColor": or(&self.delete_color, "byAuthor"),
            "changedLines": self.changed_lines,
            "changedLinesColor": or(&self.changed_lines_color, "auto"),
            "trackFormatting": self.track_formatting,
        })
    }
}

/// A choice from `options` ((value, label)) in a drop-down.
fn choice(ui: &mut Ui, id: &str, value: &mut String, options: &[(&str, &str)]) {
    let label = |k: &str| options.iter().find(|(v, _)| *v == k).map(|(_, l)| tl!(l).to_string()).unwrap_or_default();
    egui::ComboBox::from_id_salt(id).selected_text(label(value)).width(150.0).show_ui(ui, |ui| {
        for (v, l) in options {
            ui.steady_value(value, v.to_string(), tl!(l));
        }
    });
}

/// A colour: `none_label` (by author, or `auto` when given) or a colour from the grid. `value` is
/// hex.
fn color(ui: &mut Ui, theme: &[wordcraft_doc::Rgb], value: &mut String, none_label: &str, auto: Option<wordcraft_doc::Rgb>) {
    ui.horizontal(|ui| {
        let id = ui.id().with("colorField");
        crate::colors::take_field(ui.ctx(), id, value);
        let c = wordcraft_doc::Rgb::parse(value);
        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        let t = Tokens::get(ui.ctx());
        match c.or(auto) {
            Some(c) => {
                ui.painter().rect_filled(r, 2.0, crate::theme::c32(c));
            }
            // By author: the first author colours, side by side.
            None => {
                for (k, a) in [0u32, 1, 2].iter().enumerate() {
                    let w = r.width() / 3.0;
                    let part = egui::Rect::from_min_size(r.min + vec2(w * k as f32, 0.0), vec2(w, r.height()));
                    ui.painter().rect_filled(part, 0.0, crate::theme::c32(wordcraft_layout::display::revision_color(*a)));
                }
            }
        }
        ui.painter().rect_stroke(r, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
        let label = if c.is_some() { value.clone() } else { tl!(none_label).to_string() };
        ui.menu_button(label, |ui| {
            if ui.button(tl!(none_label)).clicked() {
                value.clear();
                ui.close();
            }
            crate::colors::field_menu(ui, theme, id, value);
        });
    });
}

/// Track Changes Options. Returns true to close.
pub fn track_options(app: &mut WordApp, ui: &mut Ui, f: &mut TrackForm) -> bool {
    let theme = app.session.doc.settings.theme_colors.clone();
    let heading = |ui: &mut Ui, s: &str| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
    };
    heading(ui, "Show");
    egui::Grid::new("tco_show").num_columns(2).spacing(vec2(16.0, 4.0)).show(ui, |ui| {
        ui.checkbox(&mut f.comments, tl!("Comments"));
        ui.checkbox(&mut f.ink, tl!("Ink"));
        ui.end_row();
        ui.checkbox(&mut f.insertions_deletions, tl!("Insertions and Deletions"));
        ui.checkbox(&mut f.formatting, tl!("Formatting"));
        ui.end_row();
    });
    heading(ui, "Balloons");
    for (v, l) in [
        ("revisions", "Show revisions in balloons"),
        ("inline", "Show all revisions inline"),
        ("commentsAndFormatting", "Show only comments and formatting in balloons"),
    ] {
        ui.radio_value(&mut f.balloons, v.to_string(), tl!(l));
    }
    heading(ui, "Comments");
    for (v, l) in [("margin", "In the margin"), ("selected", "Only for the selected text")] {
        ui.radio_value(&mut f.comment_display, v.to_string(), tl!(l));
    }
    heading(ui, "Advanced");
    egui::Grid::new("tco_advanced").num_columns(3).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Insertions:"));
        choice(
            ui,
            "tco_ins",
            &mut f.insert_mark,
            &[
                ("underline", "Underline"),
                ("doubleUnderline", "Double Underline"),
                ("bold", "Bold"),
                ("italic", "Italic"),
                ("strikethrough", "Strikethrough"),
                ("colorOnly", "Color only"),
                ("none", "None"),
            ],
        );
        color(ui, &theme, &mut f.insert_color, "By author", None);
        ui.end_row();
        ui.label(tl!("Deletions:"));
        choice(
            ui,
            "tco_del",
            &mut f.delete_mark,
            &[
                ("strikethrough", "Strikethrough"),
                ("doubleStrikethrough", "Double Strikethrough"),
                ("hidden", "Hidden"),
                ("caret", "^"),
                ("hash", "#"),
                ("underline", "Underline"),
                ("colorOnly", "Color only"),
            ],
        );
        color(ui, &theme, &mut f.delete_color, "By author", None);
        ui.end_row();
        ui.label(tl!("Changed lines:"));
        choice(
            ui,
            "tco_bar",
            &mut f.changed_lines,
            &[("outside", "Outside border"), ("left", "Left border"), ("right", "Right border"), ("none", "None")],
        );
        color(ui, &theme, &mut f.changed_lines_color, "Automatic", Some(wordcraft_layout::display::CHANGE_BAR));
        ui.end_row();
    });
    ui.add_space(4.0);
    ui.checkbox(&mut f.track_formatting, tl!("Track formatting"));
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if ok && let Err(e) = app.run("review.trackingOptions", f.params()) {
        app.status(e);
    }
    ok || cancel
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wordcraft_engine::Session;

    use crate::dialogs::Dialog;
    use crate::{Services, WordApp};

    fn app() -> WordApp {
        WordApp::new(Session::new(wordcraft_doc::Document::new()), Services::default())
    }

    /// A list named and saved in My Lists is kept with the settings (and read back from them),
    /// shows how it numbers, and numbers a paragraph in another document; one saved again under
    /// the same name replaces it, and a malformed entry in the settings is dropped.
    #[test]
    fn a_saved_list_is_kept_and_numbers_another_document() {
        let mut a = app();
        let mut f = super::ListForm::read(&a);
        f.levels[0].format = "thaiNumbers".into();
        f.levels[0].text = "%1.".into();
        f.levels[1].format = "thaiNumbers".into();
        f.levels[1].text = "%1.%2.".into();
        f.name = "Thesis".into();
        f.save = true;
        a.run("list.define", f.params()).unwrap();
        f.save_to(&mut a.ui.saved_lists);
        f.save_to(&mut a.ui.saved_lists);
        assert_eq!(a.ui.saved_lists.len(), 1, "saved again under its name: replaced");
        let saved = a.ui.saved_lists[0].clone();
        assert_eq!(saved.sample(), "๑. ๑.๑. i.");
        // Through the settings file and back, with a bad entry beside it.
        let mut json = serde_json::to_value(a.prefs()).unwrap();
        json["savedLists"].as_array_mut().unwrap().push(json!({"name": 3}));
        let back = crate::UiState::from_json(&serde_json::to_vec(&json).unwrap()).unwrap();
        assert_eq!(back.saved_lists, vec![saved.clone()]);
        // Another document: picking it numbers the paragraph in Thai.
        let mut b = WordApp::new(Session::new(wordcraft_doc::Document::from_text("บทนำ")), Services::default());
        b.run("list.define", saved.params()).unwrap();
        let p = b.session.doc.para(wordcraft_doc::StoryRef::Body, &wordcraft_doc::Path::top(0)).unwrap();
        let num = p.props.numbering.expect("numbered").num;
        let aid = b.session.doc.numbering.num(num).unwrap().abstract_id;
        let abs = b.session.doc.numbering.abstracts.iter().find(|x| x.id == aid).unwrap();
        assert_eq!(abs.name.as_deref(), Some("Thesis"));
        // Unsaved or nameless: nothing kept.
        let mut g = super::ListForm::read(&b);
        g.name = "Draft".into();
        let mut none = Vec::new();
        g.save_to(&mut none);
        g.save = true;
        g.name = "  ".into();
        g.save_to(&mut none);
        assert!(none.is_empty());
    }

    /// #328: the ribbon opens Define New Multilevel List; OK defines the list and numbers the
    /// paragraph. Scripts without levels get an error, never the dialog.
    #[test]
    fn define_new_multilevel_list_opens_from_the_ribbon_and_applies() {
        let mut a = app();
        a.run("text.insert", json!({"text": "Intro"})).unwrap();
        assert_eq!(a.run("list.define", json!({})).unwrap(), json!({"pending": "defineList"}));
        let Some(Dialog::DefineList { form }) = &mut a.dialog else { panic!("dialog") };
        assert_eq!((form.levels.len(), form.sample(0), form.sample(1)), (9, "1.".to_string(), "a.".to_string()));
        form.levels[0].text = "Part %1:".into();
        form.levels[0].format = "upperRoman".into();
        form.levels[1].text = "%1.%2".into();
        form.levels[1].legal = true;
        assert_eq!((form.sample(0), form.sample(1)), ("Part I:".to_string(), "1.a".to_string()), "legal shows levels above in digits");
        let params = form.params();
        a.dialog = None;
        a.run("list.define", params).unwrap();
        let got = a.session.run("list.get", &json!({})).unwrap();
        assert_eq!((got["levels"][0]["text"].as_str(), got["levels"][0]["format"].as_str()), (Some("Part %1:"), Some("upperRoman")));
        // Opening it again starts from the caret's list.
        a.run("ui.dialog", json!({"name": "defineList"})).unwrap();
        assert!(matches!(&a.dialog, Some(Dialog::DefineList { form }) if form.sample(0) == "Part I:"));
        let mut b = app();
        assert!(b.execute("list.define", json!({})).is_err());
        assert!(b.dialog.is_none());
    }

    /// #328: the Tracking group's launcher opens Track Changes Options; OK changes the per-user
    /// preference that is saved with the interface settings.
    #[test]
    fn track_changes_options_open_from_the_launcher_and_persist() {
        let mut a = app();
        assert_eq!(a.run("review.trackingOptions", json!({})).unwrap(), json!({"pending": "trackChangesOptions"}));
        let Some(Dialog::TrackOptions { form }) = &mut a.dialog else { panic!("dialog") };
        assert_eq!((form.insert_mark.as_str(), form.balloons.as_str(), form.insert_color.as_str()), ("underline", "commentsAndFormatting", ""));
        assert_eq!(form.comment_display, "margin", "comments stay in the margin, as in Word, unless chosen otherwise (#1055)");
        form.comment_display = "selected".into();
        form.insert_mark = "doubleUnderline".into();
        form.changed_lines = "right".into();
        form.changed_lines_color = "FF0000".into();
        let params = form.params();
        a.dialog = None;
        a.run("review.trackingOptions", params).unwrap();
        let m = &a.session.prefs.markup;
        assert_eq!(m.insert_mark, wordcraft_layout::display::InsertMark::DoubleUnderline);
        assert_eq!(m.comment_display, wordcraft_layout::display::CommentDisplay::Selected);
        assert_eq!(m.changed_lines_color, Some(wordcraft_doc::Rgb(0xFF, 0, 0)));
        // Saved and restored with the preferences.
        let saved = a.prefs();
        let mut b = app();
        b.apply_prefs(saved);
        assert_eq!(b.session.prefs.markup.changed_lines, wordcraft_layout::display::ChangeBar::Right);
    }
}
