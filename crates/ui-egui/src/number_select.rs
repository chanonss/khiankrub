//! Selecting a list number, as in Word: a click on a paragraph's number selects the number (darker
//! grey) and shows every number at the same level of that list in light grey. While it is
//! selected, Bold, Italic, Underline, the font, its size and colour change that level's numbers
//! (`list.define` on the list, in place) rather than the text. Anything that moves the caret ends
//! it.

use egui::{Color32, Painter, Rect, pos2, vec2};
use serde_json::{Value, json};
use wordcraft_doc::{Path, Pos, StoryRef};
use wordcraft_layout::DocLayout;

use crate::WordApp;

/// The list and level of the paragraph at `story`/`path`, if it is numbered.
pub fn list_of(doc: &wordcraft_doc::Document, story: StoryRef, path: &Path) -> Option<(u32, u8)> {
    let para = doc.para_at(&Pos { story, path: path.clone(), off: 0 })?;
    let n = para.props.numbering.or_else(|| doc.styles.resolve_para(&para.props).numbering)?;
    (n.num != 0).then_some((n.num, n.level))
}

/// The selected number, while the caret is still where selecting it put it (at the start of its
/// paragraph) and the paragraph is still numbered.
pub fn active(app: &WordApp) -> Option<(StoryRef, Path)> {
    let (story, path) = app.canvas.number_sel.clone()?;
    let at = Pos { story, path: path.clone(), off: 0 };
    (app.session.sel.is_collapsed() && app.session.sel.focus == at && list_of(&app.session.doc, story, &path).is_some()).then_some((story, path))
}

/// Select the number of the paragraph at `story`/`path`: the caret goes to the paragraph's start.
pub fn select(app: &mut WordApp, story: StoryRef, path: Path) {
    app.session.sel = wordcraft_engine::Selection::caret(Pos { story, path: path.clone(), off: 0 });
    app.canvas.number_sel = Some((story, path));
}

/// The commands that format a selected number instead of the text.
const ROUTED: [&str; 8] =
    ["format.bold", "format.italic", "format.underline", "format.font", "format.size", "format.color", "format.growFont", "format.shrinkFont"];

/// Font sizes Increase/Decrease Font Size step through, as for text.
const SIZES: [f32; 16] = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 36.0, 48.0, 72.0];

/// With a number selected, a formatting command changes its level's numbers: `Some` with the
/// result, or `None` to run the command on the text as usual.
pub fn route(app: &mut WordApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if !ROUTED.contains(&id) {
        return None;
    }
    let (story, path) = active(app)?;
    let (num, level) = list_of(&app.session.doc, story, &path)?;
    let got = app.session.run("list.get", &json!({"num": num})).ok()?;
    let cur = got.get("levels").and_then(|l| l.get(usize::from(level))).cloned().unwrap_or_default();
    let flag = |k: &str| cur.get(k).and_then(Value::as_bool).unwrap_or(false);
    let given = |k: &str| params.get(k).and_then(Value::as_bool);
    // The number's size now: its own, else the text's at the paragraph start.
    let text_size = app.session.run("format.state", &json!({})).ok().and_then(|st| st.get("size").and_then(Value::as_f64)).map(|s| s as f32);
    let size_now = || cur.get("size").and_then(Value::as_f64).map(|s| s as f32).or(text_size);
    let patch = match id {
        "format.bold" => json!({"bold": given("value").unwrap_or(!flag("bold"))}),
        "format.italic" => json!({"italic": given("value").unwrap_or(!flag("italic"))}),
        "format.underline" => json!({"underline": given("value").unwrap_or(!flag("underline"))}),
        "format.font" => json!({"font": params.get("name").cloned().unwrap_or(Value::Null)}),
        "format.size" => json!({"size": params.get("size").cloned().unwrap_or(Value::Null)}),
        "format.color" => match params.get("color").and_then(Value::as_str) {
            Some("auto") | None => json!({"color": Value::Null}),
            Some(c) => json!({"color": c}),
        },
        "format.growFont" | "format.shrinkFont" => {
            let now = size_now().unwrap_or(11.0);
            let next = if id == "format.growFont" {
                SIZES.iter().copied().find(|s| *s > now + 0.01).unwrap_or((now + 10.0).min(1638.0))
            } else {
                SIZES.iter().rev().copied().find(|s| *s < now - 0.01).unwrap_or((now - 1.0).max(1.0))
            };
            json!({"size": next})
        }
        _ => return None,
    };
    let mut levels = vec![json!({}); usize::from(level) + 1];
    if let Some(last) = levels.last_mut() {
        *last = patch;
    }
    Some(app.run("list.define", json!({"num": num, "apply": false, "levels": levels})))
}

/// Paint the selection: every number at the selected level of the list in light grey, the
/// selected one darker. `rects` are the pages' screen rectangles.
pub fn paint(app: &WordApp, layout: &DocLayout, rects: &[Rect], scale_of: impl Fn(usize) -> f32, painter: &Painter) {
    let Some((story, path)) = active(app) else { return };
    let Some(list) = list_of(&app.session.doc, story, &path) else { return };
    let (dark, light) = if crate::theme::Tokens::get(painter.ctx()).dark {
        (Color32::from_white_alpha(70), Color32::from_white_alpha(34))
    } else {
        (Color32::from_black_alpha(64), Color32::from_black_alpha(30))
    };
    for (pi, pr) in rects.iter().enumerate() {
        if !painter.clip_rect().intersects(*pr) {
            continue;
        }
        let Some(page) = layout.pages.get(pi) else { continue };
        let scale = scale_of(pi);
        for it in &page.items {
            let wordcraft_layout::Placed::Lines { story: s, path: p, .. } = it else { continue };
            if *s != story || list_of(&app.session.doc, *s, p) != Some(list) {
                continue;
            }
            let Some((lpi, r)) = layout.label_rect(*s, p, pi) else { continue };
            if lpi != pi {
                continue;
            }
            let sr = Rect::from_min_size(pos2(pr.min.x + r.x * scale, pr.min.y + r.y * scale), vec2(r.w * scale, r.h * scale));
            painter.rect_filled(sr, 0.0, if *p == path { dark } else { light });
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wordcraft_doc::{Pos, StoryRef};

    use crate::WordApp;

    type H = egui_kittest::Harness<'static, WordApp>;

    /// 1. บทนำ / 1.1 ประเด็นที่ 1 / 1.2 ประเด็นที่ 2 / 2. สรุป, in a window.
    pub(crate) fn list_window() -> H {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        for (cmd, p) in [
            ("text.insert", json!({"text": "บทนำ"})),
            ("para.multilevel", json!({"kind": "legal"})),
            ("text.newParagraph", json!({})),
            ("text.tab", json!({})),
            ("text.insert", json!({"text": "ประเด็นที่ 1"})),
            ("text.newParagraph", json!({})),
            ("text.insert", json!({"text": "ประเด็นที่ 2"})),
            ("text.newParagraph", json!({})),
            ("text.backTab", json!({})),
            ("text.insert", json!({"text": "สรุป"})),
        ] {
            a.run(cmd, p).unwrap();
        }
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h
    }

    /// Click the number of body paragraph `i`.
    pub(crate) fn click_number(h: &mut H, i: u32) {
        let layout = h.state_mut().session.layout();
        let (pi, r) = layout.label_rect(StoryRef::Body, &wordcraft_doc::Path(vec![i]), 0).expect("a number");
        let at = crate::canvas::page_to_screen(h.state_mut(), pi, r.x + r.w / 2.0, r.y + r.h / 2.0).expect("on screen");
        h.event(egui::Event::PointerMoved(at));
        h.step();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
            h.step();
        }
        h.run_steps(2);
    }

    /// A click on "1.1" selects it: the caret goes to its paragraph's start and the number is
    /// selected; Bold then makes level 2's numbers bold (1.1 and 1.2), not the text, and Increase
    /// Font Size steps their size. Typing or another click ends it.
    #[test]
    fn clicking_a_number_selects_its_level_and_formats_the_numbers() {
        let mut h = list_window();
        click_number(&mut h, 1);
        let at = Pos::body(1, 0);
        assert_eq!(h.state().session.sel.focus, at, "the caret is at the paragraph's start");
        assert!(super::active(h.state()).is_some(), "the number is selected");
        let (num, level) = super::list_of(&h.state().session.doc, StoryRef::Body, &at.path).unwrap();
        assert_eq!(level, 1);
        h.state_mut().run("format.bold", json!({})).unwrap();
        let levels = h.state_mut().session.run("list.get", &json!({"num": num})).unwrap();
        assert_eq!(levels["levels"][1]["bold"], json!(true), "level 2's numbers are bold");
        assert_eq!(levels["levels"][0]["bold"], json!(false), "level 1's are not");
        let state = h.state_mut().session.run("format.state", &json!({})).unwrap();
        assert_eq!(state["bold"], json!(false), "the text isn't bold");
        h.state_mut().run("format.size", json!({"size": 20})).unwrap();
        h.state_mut().run("format.growFont", json!({})).unwrap();
        let levels = h.state_mut().session.run("list.get", &json!({"num": num})).unwrap();
        assert_eq!(levels["levels"][1]["size"], json!(22.0));
        // Typing ends it, and Bold formats text again.
        h.state_mut().run("text.insert", json!({"text": "x"})).unwrap();
        assert!(super::active(h.state()).is_none());
        // Another click on a number selects that one; a click in text ends it.
        click_number(&mut h, 3);
        assert_eq!(super::active(h.state()).map(|(_, p)| p), Some(wordcraft_doc::Path(vec![3])));
        h.state_mut().session.sel = wordcraft_engine::Selection::caret(Pos::body(3, 2));
        assert!(super::active(h.state()).is_none(), "moving the caret ends it");
    }
}
