//! The Thai interface: the most-seen parts (the ribbon tabs, the File page, right-click menus) are
//! in Thai; the rest falls back to English.

use std::collections::BTreeSet;

use egui::vec2;
use serde_json::json;

use super::{Lang, record};
use crate::WordApp;

/// A window with a document that has a table and a picture, so the table and picture tabs and
/// right-click items can show.
fn window() -> egui_kittest::Harness<'static, WordApp> {
    let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("Text")), crate::Services::default());
    a.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
    egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
        |ui, app: &mut WordApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    )
}

/// Every English string drawn on each ribbon tab and each File page (but Options, which stays
/// English for now).
fn seen_on_tabs_and_file() -> BTreeSet<(String, String)> {
    let mut h = window();
    h.run_steps(4);
    record::start();
    let tabs = wordcraft_engine::ribbon::TABS.iter().filter(|t| **t != "File").chain(["Table Design", "Table Layout"].iter());
    for tab in tabs {
        h.state_mut().ui.tab = (*tab).into();
        h.run_steps(3);
    }
    // The Header & Footer tab and the page's header tags, in a second section's header that is
    // the same as the first's.
    let _ = h.state_mut().run("caret.docEnd", json!({}));
    let _ = h.state_mut().run("layout.break", json!({"kind": "nextPage"}));
    let _ = h.state_mut().run("insert.editHeader", json!({"page": 1}));
    h.run_steps(4);
    let _ = h.state_mut().run("layout.differentFirstPage", json!({"value": true}));
    h.run_steps(3);
    let _ = h.state_mut().run("layout.differentOddEven", json!({"value": true}));
    h.run_steps(3);
    let _ = h.state_mut().run("insert.closeHeader", json!({}));
    h.run_steps(2);
    // The tabs that come with an object: each object inserted (and selected), its tab opened.
    let picture = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/app-icon/khiankrub-192.png");
    for (cmd, params, tabs) in [
        ("insert.picture", json!({"path": picture}), &["Picture Format"][..]),
        ("insert.shape", json!({"kind": "rectangle"}), &["Shape Format"][..]),
        ("insert.chart", json!({"type": "column"}), &["Chart Design"][..]),
        ("insert.smartArt", json!({"layout": "process"}), &["SmartArt Design"][..]),
        ("insert.equation", json!({}), &["Equation"][..]),
    ] {
        let _ = h.state_mut().run("caret.docEnd", json!({}));
        let _ = h.state_mut().run(cmd, params);
        for tab in tabs {
            h.state_mut().ui.tab = (*tab).into();
            h.run_steps(3);
        }
        let _ = h.state_mut().run("equation.exit", json!({}));
        h.run_steps(1);
    }
    for page in ["home", "new", "open", "info", "recover", "save", "saveAs", "print", "export", "close"] {
        h.state_mut().ui.backstage = true;
        h.state_mut().ui.backstage_page = page.into();
        h.run_steps(3);
    }
    record::stop()
}

/// The labels of the page's right-click menu and its submenus (the submenus draw only when
/// pointed at, so they are read from the menu's code).
fn right_click_labels() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let canvas = include_str!("../canvas.rs");
    let start = canvas.find("fn context_menu(app: &mut WordApp").unwrap_or(0);
    let body = &canvas[start..];
    let body = &body[..body.find("\n}\n").unwrap_or(body.len())];
    let ribbon = include_str!("../ribbon.rs");
    let wrap = ribbon.find("fn wrap_text_items").map(|i| &ribbon[i..]).unwrap_or("");
    let wrap = &wrap[..wrap.find("\n}\n").unwrap_or(wrap.len())];
    for src in [body, wrap] {
        for marker in ["item(ui, app, \"", "tl!(\"", "mi_check(ui, app, \""] {
            for piece in src.split(marker).skip(1) {
                if let Some(end) = piece.find('"') {
                    out.insert(piece[..end].to_string());
                }
            }
        }
    }
    for (label, _, _) in crate::ribbon::WRAP_STYLES {
        out.insert(label.to_string());
    }
    out
}

/// Text that depends on the clock or the document's state, so one run draws only some of it: the
/// File page's greeting and the title bar's saved state.
fn shown_some_of_the_time() -> BTreeSet<String> {
    ["Good morning", "Good afternoon", "Good evening", "Edited", "Saved", "Not saved", "draft", "Saved on this device", "Saving…"]
        .map(String::from)
        .into()
}

/// Shown as they are in Thai too: numbers, and product and format names.
const KEEP: &[&str] = &[
    "100%",
    "Zotero",
    "SmartArt",
    "WordArt",
    "LaTeX (*.tex)",
    "Markdown (*.md)",
    "Rich Text Format (*.rtf)",
    // Previews and paper sizes, and a label that is already Thai.
    "........",
    "AaBbYyZz",
    "{n} pt",
    "Letter",
    "ตารางที่",
];

fn thai() -> Lang {
    Lang::from_code("th").expect("Thai is a built-in interface language")
}

/// The Thai catalog has no bad lines (each would show in English).
#[test]
fn the_thai_catalog_is_well_formed() {
    let (_, errors) = super::catalog::Catalog::parse(include_str!("th.tsv"));
    assert!(errors.is_empty(), "{errors:#?}");
}

/// Everything drawn on the ribbon tabs, the File page and the right-click menus has a Thai entry,
/// so a new button or menu item fails here until it is translated.
#[test]
fn the_most_seen_parts_are_in_thai() {
    let th = thai();
    let mut missing: Vec<String> = seen_on_tabs_and_file()
        .into_iter()
        .chain(right_click_labels().into_iter().chain(shown_some_of_the_time()).map(|s| (String::new(), s)))
        .filter(|(_, s)| !KEEP.contains(&s.as_str()))
        .filter(|(ctx, s)| if ctx.is_empty() { !super::has(th, s) } else { super::trc(th, ctx, s) == s })
        .map(|(ctx, s)| format!("{ctx}\t{}", s.replace('\n', "\\n")))
        .collect();
    missing.dedup();
    assert!(missing.is_empty(), "add these to th.tsv:\n{}", missing.join("\n"));
}

/// Thai Macs and PCs get the Thai interface on Automatic; the setting picks it by hand too, and
/// the ribbon's tabs then read in Thai. The document is untouched.
#[test]
fn choosing_thai_shows_the_ribbon_in_thai() {
    use egui_kittest::kittest::Queryable;
    for tag in ["th", "th-TH", "th_TH.UTF-8"] {
        assert_eq!(super::lang_from_tag(tag).map(Lang::code), Some("th"), "{tag}");
    }
    let mut h = window();
    let before = serde_json::to_value(&h.state().session.doc).unwrap();
    h.state_mut().run("ui.language", json!({"value": "th"})).unwrap();
    h.run_steps(4);
    assert_eq!(h.state().ui.language, "th");
    for tab in ["หน้าแรก", "แทรก", "เค้าโครง", "การอ้างอิง", "มุมมอง"] {
        assert!(h.query_by_label(tab).is_some(), "the {tab} tab");
    }
    assert!(h.query_by_label("Home").is_none(), "no English tab name");
    assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), before);
    h.state_mut().run("ui.language", json!({"value": "en"})).unwrap();
    h.run_steps(2);
    super::set_current(Lang::EN);
}

/// The dialogs people open most, each `(name, tabs)`: the `ui.dialog` name and the labels of its
/// other tabs, clicked in turn.
const COMMON_DIALOGS: &[(&str, &[&str])] = &[
    ("font", &["Advanced"]),
    ("paragraph", &["Line and Page Breaks"]),
    ("tabs", &[]),
    ("borders", &[]),
    ("pageBorders", &[]),
    ("shading", &[]),
    ("find", &[]),
    ("replace", &[]),
    ("goto", &[]),
    ("insertTable", &[]),
    ("pageSetup", &["Paper", "Layout"]),
    ("link", &[]),
    ("bookmark", &[]),
    ("wordCount", &[]),
    ("zoom", &[]),
    ("watermark", &[]),
    ("symbol", &[]),
    ("specialCharacters", &[]),
    ("columns", &[]),
    ("lineNumbers", &[]),
    ("hyphenation", &[]),
    ("language", &[]),
    ("pageNumberPosition", &[]),
    ("defineList", &[]),
    ("renameDocument", &[]),
    ("pasteSpecial", &[]),
    ("tableProperties", &["Row", "Column", "Cell"]),
    ("caption", &[]),
    ("crossReference", &[]),
    ("toc", &[]),
    ("noteOptions", &[]),
    ("trackChangesOptions", &[]),
    ("encryptPassword", &[]),
    ("autoCorrect", &[]),
    ("newStyle", &[]),
    ("manageStyles", &[]),
];

/// The File › Options panes.
const OPTION_PANES: &[&str] = &["general", "display", "proofing", "save", "language", "accessibility", "advanced"];

/// Every English string drawn in the common dialogs (each tab) and File › Options.
fn seen_in_common_dialogs() -> BTreeSet<(String, String)> {
    let mut h = window();
    h.state_mut().ui.language = "en".into();
    h.run_steps(4);
    record::start();
    for (name, tabs) in COMMON_DIALOGS {
        if h.state_mut().run("ui.dialog", json!({"name": name})).is_err() {
            continue;
        }
        h.run_steps(3);
        for tab in *tabs {
            use egui_kittest::kittest::Queryable;
            if let Some(node) = h.query_all_by_label(tab).last() {
                node.click();
                h.run_steps(3);
            }
        }
        h.state_mut().dialog = None;
        h.run_steps(2);
    }
    for pane in OPTION_PANES {
        h.state_mut().ui.backstage = true;
        h.state_mut().ui.backstage_page = "options".into();
        h.state_mut().ui.options_pane = (*pane).into();
        h.run_steps(3);
    }
    record::stop()
}

/// The dialogs people open most (each tab) and File › Options are in Thai, so a new label there
/// fails here until it is translated.
#[test]
fn the_common_dialogs_are_in_thai() {
    let th = thai();
    let missing: Vec<String> = seen_in_common_dialogs()
        .into_iter()
        .filter(|(_, s)| !KEEP.contains(&s.as_str()))
        .filter(|(ctx, s)| if ctx.is_empty() { !super::has(th, s) } else { super::trc(th, ctx, s) == s })
        .map(|(ctx, s)| format!("{ctx}\t{}", s.replace('\n', "\\n")))
        .collect();
    assert!(missing.is_empty(), "add these to th.tsv:\n{}", missing.join("\n"));
}

#[test]
fn shape_format_controls_and_effects_are_translated() {
    use egui_kittest::kittest::Queryable;
    let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
    app.run("ui.language", json!({"value": "th"})).unwrap();
    app.run("insert.shape", json!({"kind": "rectangle"})).unwrap();
    app.run("ui.tab", json!({"tab": "Shape Format"})).unwrap();
    let mut h = egui_kittest::Harness::builder().with_size(vec2(2200.0, 400.0)).build_ui_state(
        |ui, app: &mut WordApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    for label in ["สีเติมรูปร่าง", "เส้นกรอบรูปร่าง", "เงา แสงเรือง และขอบนุ่ม", "สีเติมข้อความ", "เส้นกรอบข้อความ", "เอฟเฟกต์ข้อความ"]
    {
        assert!(h.query_by_label(label).is_some(), "missing translated control: {label}");
    }
    let th = Lang::from_code("th").unwrap();
    for source in [
        "Shape Format",
        "Alt Text",
        "Glow",
        "No Glow",
        "Glow Colors",
        "Soft Edges",
        "No Soft Edges",
        "Reflection",
        "Transform",
        "Create Link",
        "Break Link",
    ] {
        assert_ne!(super::tr(th, source), source, "missing translation: {source}");
    }
}

#[test]
fn chart_design_controls_are_translated() {
    use egui_kittest::kittest::Queryable;
    let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
    app.run("ui.language", json!({"value": "th"})).unwrap();
    app.run("insert.chart", json!({"type": "column"})).unwrap();
    app.run("ui.tab", json!({"tab": "Chart Design"})).unwrap();
    let mut h = egui_kittest::Harness::builder().with_size(vec2(2000.0, 400.0)).build_ui_state(
        |ui, app: &mut WordApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    for label in ["ชื่อแผนภูมิ", "คำอธิบายแผนภูมิ", "ป้ายชื่อข้อมูล"]
    {
        assert!(h.query_by_label(label).is_some(), "missing translated control: {label}");
    }
    let th = Lang::from_code("th").unwrap();
    for label in ["Chart Design", "Edit\nData", "Change\nChart Type", "Above Chart", "Insert SmartArt"] {
        assert_ne!(super::tr(th, label), label, "missing translation: {label}");
    }
}

#[test]
fn home_menu_labels_are_translated() {
    let th = Lang::from_code("th").unwrap();
    for label in [
        "Advanced Find…",
        "Go To…",
        "Select All",
        "Select Paragraph",
        "Select Sentence",
        "Dot-dash",
        "Small Caps",
        "All Caps",
        "Double Strikethrough",
        "Standard Colors",
        "More Colors…",
        "Add Space Before Paragraph",
        "Remove Space After Paragraph",
        "Line Spacing Options…",
        "Bottom Border",
        "Top Border",
        "Left Border",
        "Right Border",
        "No Border",
        "All Borders",
        "Outside Borders",
        "Inside Borders",
        "Inside Horizontal Border",
        "Inside Vertical Border",
        "Diagonal Down Border",
        "Diagonal Up Border",
        "Horizontal Line",
        "Draw Table",
        "View Gridlines",
        "Borders and Shading…",
        "Keep Text Only",
        "Merge Formatting",
        "Paste Special…",
    ] {
        assert_ne!(super::tr(th, label), label, "missing Thai translation: {label}");
    }
}

#[test]
fn header_footer_and_print_range_labels_are_translated() {
    let th = Lang::from_code("th").unwrap();
    for label in [
        "Blank",
        "Blank (Three Columns)",
        "Document Title",
        "Edit Header",
        "Remove Header",
        "Edit Footer",
        "Remove Footer",
        "Top of Page",
        "Bottom of Page",
        "Page X of Y",
        "Current Position",
        "Format Page Numbers…",
        "Page Number Position…",
        "Current page",
        "Selected pages",
        "Custom range",
        "Odd pages only",
        "Even pages only",
    ] {
        assert_ne!(super::tr(th, label), label, "missing Thai translation: {label}");
    }
}
