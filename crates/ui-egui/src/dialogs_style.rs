//! Modify Style (Home › Styles, Manage Styles, the Styles pane, Table of Contents › Modify…): a
//! window with a list of pages on the left — General, Font, Indents & Spacing, Alignment, Text Flow,
//! Tabs, Borders & Shading, Numbering, and for text Font Effects and Position — each with its icon —
//! and a
//! preview of the style as the boxes leave it. OK sends everything as one `styles.modify`.
//!
//! The Font page sets Latin and Thai (complex script) text side by side: Word keeps a separate
//! font, size, bold and italic for each, and a Thai heading showed one size in the size box and
//! another on the page when only the Latin ones could be seen.

use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};
use wordcraft_doc::props::{LineSpacing, TabAlign, TabLeader, TabStop, Underline, VertAlign};
use wordcraft_doc::{Align, CharProps, ParaProps, StyleKind, TextColor};

use crate::WordApp;
use crate::options::MeasureExt;
use crate::theme::semibold;
use crate::widgets::SteadyUi;

/// The pages, in order: (id, name, icon). A character style has only the first four.
const PAGES: [(&str, &str, &str); 10] = [
    ("general", "General", "styles"),
    ("font", "Font", "fonts"),
    ("effects", "Font Effects", "effects"),
    ("position", "Position", "superscript"),
    ("indents", "Indents & Spacing", "paraSpacing"),
    ("alignment", "Alignment", "justify"),
    ("flow", "Text Flow", "pageBreak"),
    ("tabs", "Tabs", "ruler"),
    ("borders", "Borders & Shading", "borders"),
    ("numbering", "Numbering", "multilevel"),
];
const CHARACTER_PAGES: usize = 4;

/// Underline styles offered: (OOXML name, label).
const UNDERLINES: [(&str, &str); 9] = [
    ("none", "(none)"),
    ("single", "Single"),
    ("words", "Words only"),
    ("double", "Double"),
    ("thick", "Thick"),
    ("dotted", "Dotted"),
    ("dash", "Dashed"),
    ("dotDash", "Dot dash"),
    ("wave", "Wave"),
];

/// Tab stop alignments and leaders offered: (model value, label).
const TAB_ALIGNS: [(TabAlign, &str); 5] =
    [(TabAlign::Left, "Left"), (TabAlign::Center, "Center"), (TabAlign::Right, "Right"), (TabAlign::Decimal, "Decimal"), (TabAlign::Bar, "Bar")];
const TAB_LEADERS: [(TabLeader, &str); 5] = [
    (TabLeader::None, "None"),
    (TabLeader::Dot, "....."),
    (TabLeader::Hyphen, "-----"),
    (TabLeader::Underscore, "_____"),
    (TabLeader::MiddleDot, "·····"),
];

/// Border line styles offered: (OOXML name, label).
const LINES: [(&str, &str); 6] =
    [("single", "Single"), ("double", "Double"), ("dotted", "Dotted"), ("dashed", "Dashed"), ("thick", "Thick"), ("dotDash", "Dot dash")];

/// New multilevel lists a style can be numbered with: (`list.kind`, sample).
const NEW_LISTS: [(&str, &str); 4] =
    [("legal", "1. 1.1. 1.1.1."), ("outline", "I. A. 1. a."), ("thaiLegal", "๑. ๑.๑. ๑.๑.๑."), ("thaiOutline", "๑. ก. (๑) (ก)")];

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
    // Font Effects.
    /// `Underline`'s OOXML name (`none`, `single`, `double`…).
    pub underline: String,
    /// `RRGGBB`, or `auto` (the text's colour).
    pub underline_color: String,
    pub strike: bool,
    pub double_strike: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub small_caps: bool,
    pub caps: bool,
    pub hidden: bool,
    pub outline: bool,
    pub shadow: bool,
    pub emboss: bool,
    pub engrave: bool,
    // Position.
    /// Character spacing, points.
    pub spacing: f32,
    /// Raised (+) or lowered (−), points.
    pub raise: f32,
    /// Width scale, percent.
    pub scale: f32,
    pub kern: bool,
    /// Kern from this font size up, points.
    pub kern_at: f32,
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
    // Text Flow.
    pub widow: bool,
    pub keep_next: bool,
    pub keep_lines: bool,
    pub page_break: bool,
    pub no_hyphens: bool,
    pub no_line_numbers: bool,
    // Tabs: position (interface unit), alignment, leader.
    pub tabs: Vec<(f32, TabAlign, TabLeader)>,
    /// The stop being edited.
    pub tab: usize,
    /// Stops the style inherits (positions in points), so deleting one writes a clear.
    #[serde(skip)]
    pub inherited_tabs: Vec<f32>,
    // Borders & Shading: the sides with a line, and that line.
    pub border_top: bool,
    pub border_bottom: bool,
    pub border_left: bool,
    pub border_right: bool,
    pub border_between: bool,
    pub border_style: String,
    /// Points.
    pub border_width: f32,
    pub border_color: String,
    /// Distance from the text, points.
    pub border_space: f32,
    /// `RRGGBB`, or empty for none.
    pub shading: String,
    // Numbering: `""` none, `num:<id>` a list of the document's, `kind:<legal…>` a new one.
    pub list: String,
    /// 1-based.
    pub list_level: u8,
    /// The document's lists: (`num:<id>`, how it numbers).
    #[serde(skip)]
    pub lists: Vec<(String, String)>,
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
        let bs = rp.borders.unwrap_or_default();
        let shown = |b: Option<wordcraft_doc::props::Border>| b.is_some_and(|b| b.is_visible());
        let first_border = [bs.top, bs.bottom, bs.left, bs.right, bs.between].into_iter().flatten().find(|b| b.is_visible());
        // The document's numbered lists, one per definition, with how they number.
        let doc = &app.session.doc;
        let mut lists = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for n in &doc.numbering.nums {
            if !seen.insert(n.abstract_id) {
                continue;
            }
            let Some(levels) = wordcraft_engine::cmd::lists::levels_of(&app.session, n.id) else { continue };
            if levels.first().is_none_or(|l| l.format == wordcraft_doc::section::NumFormat::Bullet) {
                continue;
            }
            let levels = Value::Array(levels.iter().map(wordcraft_engine::cmd::lists::level_json).collect());
            let sample = crate::dialogs_lists::SavedList { name: String::new(), levels }.sample();
            let name = doc.numbering.abstracts.iter().find(|a| a.id == n.abstract_id).and_then(|a| a.name.clone());
            lists.push((format!("num:{}", n.id), name.map_or(sample.clone(), |nm| format!("{nm}   {sample}"))));
        }
        // The tab stops the style's base gives it (the document default's for a style without one).
        let base_para = ParaProps { style: st.based_on.clone(), ..Default::default() };
        let inherited_tabs = if st.based_on.is_some() { sheet.resolve_para(&base_para).tabs.iter().map(|t| t.pos).collect() } else { Vec::new() };
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
            underline: rc.underline.ooxml().to_string(),
            underline_color: rc.underline_color.map_or("auto".to_string(), |c| c.hex()),
            strike: rc.strike,
            double_strike: rc.double_strike,
            hidden: rc.hidden,
            outline: rc.outline,
            shadow: rc.shadow,
            emboss: rc.emboss,
            engrave: rc.engrave,
            raise: rc.position,
            scale: rc.scale,
            kern: rc.kern > 0.0,
            kern_at: if rc.kern > 0.0 { rc.kern } else { 14.0 },
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
            widow: rp.widow_control,
            keep_next: rp.keep_next,
            keep_lines: rp.keep_lines,
            page_break: rp.page_break_before,
            no_hyphens: rp.suppress_hyphens,
            no_line_numbers: rp.suppress_line_numbers,
            tabs: rp.tabs.iter().filter(|t| t.align != TabAlign::Clear).map(|t| (t.pos / k, t.align, t.leader)).collect(),
            tab: 0,
            inherited_tabs,
            border_top: shown(bs.top),
            border_bottom: shown(bs.bottom),
            border_left: shown(bs.left),
            border_right: shown(bs.right),
            border_between: shown(bs.between),
            border_style: first_border.map_or("single".to_string(), |b| b.style.ooxml().to_string()),
            border_width: first_border.map_or(0.5, |b| b.width),
            border_color: first_border.and_then(|b| b.color).map_or("auto".to_string(), |c| c.hex()),
            border_space: first_border.map_or(1.0, |b| b.space),
            shading: rp.shading.map(|c| c.hex()).unwrap_or_default(),
            list: rp.numbering.filter(|n| n.num != 0).map(|n| format!("num:{}", n.num)).unwrap_or_default(),
            list_level: rp.numbering.filter(|n| n.num != 0).map_or(1, |n| n.level.min(8) + 1),
            lists,
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
            underline: self.changed(|f| f.underline.clone()).then_some(Underline::from_ooxml(&self.underline)),
            underline_color: if self.changed(|f| f.underline_color.clone()) { wordcraft_doc::Rgb::parse(&self.underline_color) } else { None },
            strike: self.changed(|f| f.strike).then_some(self.strike),
            double_strike: self.changed(|f| f.double_strike).then_some(self.double_strike),
            hidden: self.changed(|f| f.hidden).then_some(self.hidden),
            outline: self.changed(|f| f.outline).then_some(self.outline),
            shadow: self.changed(|f| f.shadow).then_some(self.shadow),
            emboss: self.changed(|f| f.emboss).then_some(self.emboss),
            engrave: self.changed(|f| f.engrave).then_some(self.engrave),
            position: self.changed(|f| f.raise.to_bits()).then_some(self.raise.clamp(-1584.0, 1584.0)),
            scale: self.changed(|f| f.scale.to_bits()).then_some(self.scale.clamp(1.0, 600.0)),
            kern: self.changed(|f| (f.kern, f.kern_at.to_bits())).then_some(if self.kern { self.kern_at.clamp(1.0, 1638.0) } else { 0.0 }),
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
            widow_control: self.changed(|f| f.widow).then_some(self.widow),
            keep_next: self.changed(|f| f.keep_next).then_some(self.keep_next),
            keep_lines: self.changed(|f| f.keep_lines).then_some(self.keep_lines),
            page_break_before: self.changed(|f| f.page_break).then_some(self.page_break),
            suppress_hyphens: self.changed(|f| f.no_hyphens).then_some(self.no_hyphens),
            suppress_line_numbers: self.changed(|f| f.no_line_numbers).then_some(self.no_line_numbers),
            tabs: self.changed(|f| f.tabs.iter().map(|t| (t.0.to_bits(), t.1, t.2)).collect::<Vec<_>>()).then(|| self.tab_stops(k)),
            borders: self.borders_changed().then(|| self.borders()),
            shading: wordcraft_doc::Rgb::parse(&self.shading).filter(|_| self.changed(|f| f.shading.clone())),
            ..Default::default()
        })
    }

    /// The style's own stops: the form's, and a clear for each inherited one deleted.
    fn tab_stops(&self, k: f32) -> Vec<TabStop> {
        let mut v: Vec<TabStop> =
            self.tabs.iter().map(|(pos, align, leader)| TabStop { pos: (pos * k).clamp(0.0, 1584.0), align: *align, leader: *leader }).collect();
        for pos in &self.inherited_tabs {
            if !v.iter().any(|t| (t.pos - pos).abs() < 0.5) {
                v.push(TabStop { pos: *pos, align: TabAlign::Clear, leader: TabLeader::None });
            }
        }
        v.sort_by(|a, b| a.pos.total_cmp(&b.pos));
        v
    }

    fn borders_changed(&self) -> bool {
        self.changed(|f| {
            (
                [f.border_top, f.border_bottom, f.border_left, f.border_right, f.border_between],
                f.border_style.clone(),
                f.border_width.to_bits(),
                f.border_color.clone(),
                f.border_space.to_bits(),
            )
        })
    }

    /// The sides with a line, each that line; the others explicitly off (so a base style's
    /// borders don't show through).
    fn borders(&self) -> wordcraft_doc::props::Borders {
        use wordcraft_doc::props::{Border, BorderStyle};
        let color = if self.border_color.trim().eq_ignore_ascii_case("auto") { None } else { wordcraft_doc::Rgb::parse(&self.border_color) };
        let space = self.border_space.clamp(0.0, 31.0);
        let line = |on: bool, _: f32| {
            Some(if on {
                Border { style: BorderStyle::from_ooxml(&self.border_style), width: self.border_width.clamp(0.25, 6.0), color, space }
            } else {
                Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 }
            })
        };
        wordcraft_doc::props::Borders {
            top: line(self.border_top, 1.0),
            bottom: line(self.border_bottom, 1.0),
            left: line(self.border_left, 4.0),
            right: line(self.border_right, 4.0),
            between: line(self.border_between, 1.0),
            inside_v: None,
        }
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
            if self.changed(|f| f.shading.clone()) && self.shading.trim().is_empty() {
                v["clear"] = json!(["shading"]);
            }
            if self.changed(|f| (f.list.clone(), f.list_level)) {
                v["listLevel"] = json!(self.list_level.clamp(1, 9) - 1);
                v["list"] = if let Some(n) = self.list.strip_prefix("num:").and_then(|n| n.parse::<u64>().ok()) {
                    json!({"num": n})
                } else if let Some(k) = self.list.strip_prefix("kind:") {
                    json!({"kind": k})
                } else {
                    Value::Null
                };
            }
        }
        v
    }
}

/// The Modify Style window. Returns true to close.
pub fn body(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) -> bool {
    // A fixed area, so the window keeps its size from page to page (a separator here grew to the
    // whole screen's height).
    const PAGE_H: f32 = 400.0;
    ui.allocate_ui_with_layout(vec2(760.0, PAGE_H), egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.set_height(PAGE_H);
        // The pages.
        ui.vertical(|ui| {
            ui.set_width(150.0);
            for (id, name, icon) in PAGES.iter().take(if f.character { CHARACTER_PAGES } else { PAGES.len() }) {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                    let t = crate::theme::Tokens::get(ui.ctx());
                    crate::icons::paint(ui.painter(), r, icon, t.icon, t.accent);
                    if ui.steady_label(f.page == *id, tl!(name)).clicked() {
                        f.page = id.to_string();
                    }
                });
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
                "effects" => effects_page(app, ui, f),
                "position" => position_page(ui, f),
                "tabs" if !f.character => tabs_page(ui, f),
                "indents" if !f.character => indents_page(ui, f),
                "alignment" if !f.character => alignment_page(ui, f),
                "flow" if !f.character => flow_page(ui, f),
                "borders" if !f.character => borders_page(app, ui, f),
                "numbering" if !f.character => numbering_page(ui, f),
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
    let (ok, apply, cancel) = crate::dialogs::buttons_with_apply(ui, tl!("OK"));
    if ok || apply {
        match app.run("styles.modify", f.params()) {
            // Apply: the document shows the change and the window stays, now starting from it
            // (so the next Apply or OK sends only what changes after this).
            Ok(_) if apply => {
                if let Some(fresh) = StyleForm::read(app, &f.id) {
                    *f = StyleForm { page: f.page.clone(), tab: f.tab, ..fresh };
                }
            }
            Ok(_) => {}
            Err(e) => app.status(e),
        }
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
    egui::Grid::new("ms_color").num_columns(2).spacing(vec2(8.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Font color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.color, "Automatic", "auto");
        ui.end_row();
    });
}

fn effects_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Underline");
    egui::Grid::new("ms_underline").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Style:"));
        let shown = UNDERLINES.iter().find(|(k, _)| *k == f.underline).map_or(f.underline.clone(), |(_, l)| tl!(l).to_string());
        egui::ComboBox::from_id_salt("ms_ul").selected_text(shown).show_ui(ui, |ui| {
            for (k, l) in UNDERLINES {
                ui.steady_value(&mut f.underline, k.to_string(), tl!(l));
            }
        });
        ui.end_row();
        ui.label(tl!("Underline color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.underline_color, "Automatic", "auto");
        ui.end_row();
    });
    heading(ui, "Effects");
    egui::Grid::new("ms_fx").num_columns(3).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
        if ui.checkbox(&mut f.strike, tl!("Strikethrough")).changed() && f.strike {
            f.double_strike = false;
        }
        if ui.checkbox(&mut f.double_strike, tl!("Double strikethrough")).changed() && f.double_strike {
            f.strike = false;
        }
        ui.checkbox(&mut f.hidden, tl!("Hidden"));
        ui.end_row();
        if ui.checkbox(&mut f.superscript, tl!("Superscript")).changed() && f.superscript {
            f.subscript = false;
        }
        if ui.checkbox(&mut f.subscript, tl!("Subscript")).changed() && f.subscript {
            f.superscript = false;
        }
        ui.end_row();
        if ui.checkbox(&mut f.small_caps, tl!("Small caps")).changed() && f.small_caps {
            f.caps = false;
        }
        if ui.checkbox(&mut f.caps, tl!("All caps")).changed() && f.caps {
            f.small_caps = false;
        }
        ui.end_row();
        ui.checkbox(&mut f.outline, tl!("Outline"));
        ui.checkbox(&mut f.shadow, tl!("Shadow"));
        ui.end_row();
        // Emboss and engrave exclude each other, as in Word.
        if ui.checkbox(&mut f.emboss, tl!("Emboss")).changed() && f.emboss {
            f.engrave = false;
        }
        if ui.checkbox(&mut f.engrave, tl!("Engrave")).changed() && f.engrave {
            f.emboss = false;
        }
        ui.end_row();
    });
}

fn position_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Position");
    egui::Grid::new("ms_pos").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Raise (+) or lower (−) by (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.raise).range(-1584.0..=1584.0).speed(0.5).max_decimals(1));
        ui.end_row();
        ui.label(tl!("Scale width (%):"));
        ui.add(crate::numfield::NumField::new(&mut f.scale).range(1.0..=600.0).speed(1.0).max_decimals(0));
        ui.end_row();
    });
    heading(ui, "Spacing");
    egui::Grid::new("ms_charsp").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Character spacing (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.spacing).speed(0.1).range(-20.0..=20.0));
        ui.end_row();
        ui.checkbox(&mut f.kern, tl!("Kerning for fonts (pt) and up:"));
        ui.add_enabled(f.kern, crate::numfield::NumField::new(&mut f.kern_at).range(1.0..=1638.0).speed(0.5));
        ui.end_row();
    });
}

fn tabs_page(ui: &mut Ui, f: &mut StyleForm) {
    let unit = crate::options::unit();
    heading(ui, "Tab stops");
    ui.horizontal_top(|ui| {
        // The stops, by position; one is edited on the right.
        ui.vertical(|ui| {
            ui.set_width(150.0);
            egui::ScrollArea::vertical().id_salt("ms_tabs").max_height(220.0).show(ui, |ui| {
                if f.tabs.is_empty() {
                    ui.label(egui::RichText::new(tl!("(no tab stops)")).weak());
                }
                for (i, (pos, _, _)) in f.tabs.iter().enumerate() {
                    if ui.steady_label(f.tab == i, crate::options::length(pos * unit.pt_per_unit())).clicked() {
                        f.tab = i;
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button(tl!("New")).clicked() {
                    // After the last stop, 0.5" (1.27 cm) on, as Word's default stops are.
                    let next = f.tabs.iter().map(|t| t.0).fold(0.0f32, f32::max) + 36.0 / unit.pt_per_unit();
                    f.tabs.push((next, TabAlign::Left, TabLeader::None));
                    f.tab = f.tabs.len() - 1;
                }
                if ui.add_enabled(f.tab < f.tabs.len(), egui::Button::new(tl!("Delete"))).clicked() {
                    f.tabs.remove(f.tab);
                    f.tab = f.tab.min(f.tabs.len().saturating_sub(1));
                }
                if ui.add_enabled(!f.tabs.is_empty(), egui::Button::new(tl!("Delete All"))).clicked() {
                    f.tabs.clear();
                    f.tab = 0;
                }
            });
        });
        ui.add_space(16.0);
        let Some((pos, align, leader)) = f.tabs.get_mut(f.tab) else { return };
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(tl!("Position:"));
                ui.add(crate::numfield::NumField::new(pos).speed(0.01).range(0.0..=22.0).measure(unit).max_decimals(2));
            });
            heading(ui, "Alignment");
            ui.horizontal_wrapped(|ui| {
                for (v, l) in TAB_ALIGNS {
                    ui.radio_value(align, v, tl!(l));
                }
            });
            heading(ui, "Leader");
            ui.horizontal_wrapped(|ui| {
                for (v, l) in TAB_LEADERS {
                    ui.radio_value(leader, v, tl!(l));
                }
            });
        });
    });
    // Keep the list in position order once a position is typed.
    let editing = f.tabs.get(f.tab).copied();
    f.tabs.sort_by(|a, b| a.0.total_cmp(&b.0));
    if let Some(e) = editing {
        f.tab = f.tabs.iter().position(|t| *t == e).unwrap_or(0);
    }
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

fn flow_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Pagination");
    ui.checkbox(&mut f.widow, tl!("Widow/Orphan control"));
    ui.checkbox(&mut f.keep_next, tl!("Keep with next"));
    ui.checkbox(&mut f.keep_lines, tl!("Keep lines together"));
    ui.checkbox(&mut f.page_break, tl!("Page break before"));
    heading(ui, "Formatting exceptions");
    ui.checkbox(&mut f.no_line_numbers, tl!("Suppress line numbers"));
    ui.checkbox(&mut f.no_hyphens, tl!("Don't hyphenate"));
}

fn borders_page(app: &mut WordApp, ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Borders");
    ui.horizontal(|ui| {
        if ui.button(tl!("None")).clicked() {
            (f.border_top, f.border_bottom, f.border_left, f.border_right, f.border_between) = (false, false, false, false, false);
        }
        if ui.button(tl!("Box")).clicked() {
            (f.border_top, f.border_bottom, f.border_left, f.border_right) = (true, true, true, true);
        }
    });
    egui::Grid::new("ms_sides").num_columns(3).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
        ui.checkbox(&mut f.border_top, tl!("Top"));
        ui.checkbox(&mut f.border_bottom, tl!("Bottom"));
        ui.checkbox(&mut f.border_between, tl!("Between paragraphs"));
        ui.end_row();
        ui.checkbox(&mut f.border_left, tl!("Left"));
        ui.checkbox(&mut f.border_right, tl!("Right"));
        ui.end_row();
    });
    egui::Grid::new("ms_line").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Style:"));
        let shown = LINES.iter().find(|(k, _)| *k == f.border_style).map_or(f.border_style.clone(), |(_, l)| tl!(l).to_string());
        egui::ComboBox::from_id_salt("ms_line_style").selected_text(shown).show_ui(ui, |ui| {
            for (k, l) in LINES {
                ui.steady_value(&mut f.border_style, k.to_string(), tl!(l));
            }
        });
        ui.end_row();
        ui.label(tl!("Width (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.border_width).range(0.25..=6.0).speed(0.25).max_decimals(2));
        ui.end_row();
        ui.label(tl!("Distance from text (pt):"));
        ui.add(crate::numfield::NumField::new(&mut f.border_space).range(0.0..=31.0).speed(0.5).max_decimals(1));
        ui.end_row();
        ui.label(tl!("Color:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.border_color, "Automatic", "auto");
        ui.end_row();
    });
    heading(ui, "Shading");
    egui::Grid::new("ms_shade").num_columns(2).spacing(vec2(10.0, 8.0)).show(ui, |ui| {
        ui.label(tl!("Fill:"));
        let theme = app.session.doc.settings.theme_colors.clone();
        crate::dialogs::color_field(ui, &theme, &mut f.shading, "No Color", "");
        ui.end_row();
    });
}

fn numbering_page(ui: &mut Ui, f: &mut StyleForm) {
    heading(ui, "Numbering");
    ui.label(tl!("Number paragraphs in this style with:"));
    let shown = if f.list.is_empty() {
        tl!("(no numbering)").to_string()
    } else if let Some(k) = f.list.strip_prefix("kind:") {
        NEW_LISTS.iter().find(|(x, _)| *x == k).map_or(k.to_string(), |(_, s)| crate::i18n::fmt(tl!("New list: {sample}"), &[("sample", s)]))
    } else {
        f.lists.iter().find(|(id, _)| *id == f.list).map_or(f.list.clone(), |(_, s)| s.clone())
    };
    egui::ComboBox::from_id_salt("ms_list").selected_text(shown).width(360.0).show_ui(ui, |ui| {
        ui.steady_value(&mut f.list, String::new(), tl!("(no numbering)"));
        for (id, sample) in &f.lists {
            ui.steady_value(&mut f.list, id.clone(), sample);
        }
        for (k, sample) in NEW_LISTS {
            ui.steady_value(&mut f.list, format!("kind:{k}"), crate::i18n::fmt(tl!("New list: {sample}"), &[("sample", sample)]));
        }
    });
    ui.add_space(6.0);
    ui.add_enabled_ui(!f.list.is_empty(), |ui| {
        ui.horizontal(|ui| {
            ui.label(tl!("Level:"));
            egui::ComboBox::from_id_salt("ms_list_level").selected_text(f.list_level.to_string()).show_ui(ui, |ui| {
                for l in 1..=9u8 {
                    ui.steady_value(&mut f.list_level, l, l.to_string());
                }
            });
        });
    });
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tl!("The level is linked to this style, as Define New Multilevel List's Link level to style does.")).small().weak());
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

    /// Apply changes the style at once and keeps the window open, starting again from the change:
    /// a second Apply with nothing new sends nothing.
    #[test]
    fn apply_changes_the_style_and_keeps_the_window() {
        use egui_kittest::kittest::Queryable;
        let mut a = app();
        a.dialog = crate::dialogs::Dialog::modify_style(&a, "Heading1");
        if let Some(crate::dialogs::Dialog::ModifyStyle { form, .. }) = &mut a.dialog {
            form.size = 30.0;
            form.size_cs = 30.0;
        }
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Apply").click();
        h.run_steps(3);
        let size = h.state().session.doc.styles.resolve_char(Some("Heading1"), &Default::default()).size;
        assert_eq!(size, 30.0, "applied");
        let Some(crate::dialogs::Dialog::ModifyStyle { form, .. }) = &h.state().dialog else { panic!("the window stays open") };
        assert!(form.params()["chr"].as_object().is_some_and(|c| c.is_empty()), "nothing new to send");
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

    /// Round 2's pages: Text Flow's boxes, a double bottom border with yellow shading (and No
    /// Color taking the shading off again), and a new ๑. ๑.๑. list for Heading 1 that numbers its
    /// paragraphs, with the level linked back to the style.
    #[test]
    fn flow_borders_and_numbering_reach_the_style() {
        let mut a = app();
        a.run("styles.apply", json!({"style": "Heading 1"})).unwrap();
        let mut f = StyleForm::read(&a, "Heading1").unwrap();
        f.keep_next = false;
        f.page_break = true;
        f.border_bottom = true;
        f.border_style = "double".into();
        f.border_width = 1.5;
        f.shading = "FFFF00".into();
        f.list = "kind:thaiLegal".into();
        f.list_level = 1;
        a.run("styles.modify", f.params()).unwrap();
        let rp = a.session.doc.styles.resolve_para(&wordcraft_doc::ParaProps { style: Some("Heading1".into()), ..Default::default() });
        assert_eq!((rp.keep_next, rp.page_break_before), (false, true));
        let b = rp.borders.unwrap().bottom.unwrap();
        assert_eq!((b.style, b.width), (wordcraft_doc::props::BorderStyle::Double, 1.5));
        assert!(rp.borders.unwrap().top.is_some_and(|t| !t.is_visible()), "the other sides are off");
        assert_eq!(rp.shading, wordcraft_doc::Rgb::parse("FFFF00"));
        let n = rp.numbering.expect("numbered by the style");
        assert_eq!(n.level, 0);
        let lv = wordcraft_engine::cmd::lists::levels_of(&a.session, n.num).unwrap();
        assert_eq!(lv[0].style.as_deref(), Some("Heading1"), "the level is linked to the style");
        let label = a.session.layout().pages[0].items.iter().find_map(|i| match i {
            wordcraft_layout::Placed::Lines { para, .. } => para.label.as_ref().map(|l| l.text.clone()),
            _ => None,
        });
        assert_eq!(label.as_deref(), Some("๑."));
        // Opened again: the list shows; No Color takes the shading off.
        let mut g = StyleForm::read(&a, "Heading1").unwrap();
        assert_eq!(g.list, format!("num:{}", n.num));
        g.shading.clear();
        a.run("styles.modify", g.params()).unwrap();
        let rp = a.session.doc.styles.resolve_para(&wordcraft_doc::ParaProps { style: Some("Heading1".into()), ..Default::default() });
        assert_eq!(rp.shading, None);
    }

    /// The pages added after LibreOffice's: a double wavy-free underline style and colour, double
    /// strikethrough and emboss; text raised 3 pt at 90% width with kerning from 12 pt; tab stops
    /// (a right one with dots, and an inherited one deleted as a clear); Don't hyphenate; a border
    /// 4 pt from the text.
    #[test]
    fn effects_position_tabs_and_more_reach_the_style() {
        use wordcraft_doc::props::{TabAlign, TabLeader, TabStop};
        let mut a = app();
        let k = crate::options::unit().pt_per_unit();
        // Normal has a stop at 1"; Heading 1 (based on it) inherits it.
        if let Some(n) = a.session.doc.styles.get_mut("Normal") {
            n.para.tabs = Some(vec![TabStop { pos: 72.0, align: TabAlign::Left, leader: TabLeader::None }]);
        }
        let mut f = StyleForm::read(&a, "Heading1").unwrap();
        assert_eq!(f.tabs.len(), 1, "the inherited stop shows");
        f.underline = "double".into();
        f.underline_color = "C00000".into();
        f.double_strike = true;
        f.emboss = true;
        f.raise = 3.0;
        f.scale = 90.0;
        f.kern = true;
        f.kern_at = 12.0;
        f.tabs = vec![(216.0 / k, TabAlign::Right, TabLeader::Dot)];
        f.no_hyphens = true;
        f.border_bottom = true;
        f.border_space = 4.0;
        a.run("styles.modify", f.params()).unwrap();
        let c = a.session.doc.styles.resolve_char(Some("Heading1"), &Default::default());
        assert_eq!((c.underline, c.underline_color), (wordcraft_doc::props::Underline::Double, wordcraft_doc::Rgb::parse("C00000")));
        assert!(c.double_strike && c.emboss && !c.engrave);
        assert_eq!((c.position, c.scale, c.kern), (3.0, 90.0, 12.0));
        let p = a.session.doc.styles.resolve_para(&wordcraft_doc::ParaProps { style: Some("Heading1".into()), ..Default::default() });
        let stops: Vec<_> = p.tabs.iter().filter(|t| t.align != TabAlign::Clear).map(|t| (t.pos, t.align, t.leader)).collect();
        assert_eq!(stops, vec![(216.0, TabAlign::Right, TabLeader::Dot)], "the inherited 1\" stop is gone");
        assert!(p.suppress_hyphens);
        assert_eq!(p.borders.unwrap().bottom.unwrap().space, 4.0);
        // A character style has the first four pages only.
        assert!(StyleForm::read(&a, "Strong").unwrap().character);
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
