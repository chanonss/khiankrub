//! Hovering a control must not move any other control. egui gives a hovered selectable label (a tab,
//! a list entry) a frame whose border counts toward its size, so it grew by 2 px under the pointer and
//! its neighbours jumped. These tests hover every control of the dialogs, the Options pages and the
//! File pages one at a time and compare where every control sits.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

use crate::{Services, WordApp};

const ROLES: [Role; 9] =
    [Role::Button, Role::CheckBox, Role::RadioButton, Role::Tab, Role::MenuItem, Role::ComboBox, Role::TextInput, Role::Label, Role::Unknown];
const CLICKABLE: [Role; 6] = [Role::Button, Role::CheckBox, Role::RadioButton, Role::Tab, Role::MenuItem, Role::ComboBox];

fn app() -> WordApp {
    let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
    a.run("text.insert", json!({"text": "Hello"})).unwrap();
    a.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
    a
}

fn harness(a: WordApp) -> egui_kittest::Harness<'static, WordApp> {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
        |ui, app: &mut WordApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    h.run_steps(5);
    h
}

/// Where a dialog sits (it opens centred in the 1440 x 900 window).
fn dialog_area() -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(250.0, 80.0), egui::pos2(1190.0, 840.0))
}

/// Every control as (role, label, n-th with that name) and its rectangle.
fn snapshot(h: &egui_kittest::Harness<WordApp>) -> Vec<(String, Role, egui::Rect)> {
    let mut seen = std::collections::HashMap::<String, usize>::new();
    let mut out = Vec::new();
    for role in ROLES {
        for n in h.query_all_by_role(role) {
            let label = NodeT::accesskit_node(&n).label().unwrap_or_default().to_string();
            // Unnamed items are numbered by order, which a tooltip popping up changes.
            if label.is_empty() {
                continue;
            }
            let k = seen.entry(format!("{role:?}|{label}")).or_insert(0);
            *k += 1;
            out.push((format!("{role:?} {label:?} #{k}"), role, n.rect()));
        }
    }
    out
}

/// Hover the first `limit` clickable controls one at a time; every control that moves or resizes.
fn sweep(scene: &str, h: &mut egui_kittest::Harness<WordApp>, limit: usize, region: Option<egui::Rect>) -> Vec<String> {
    let away = egui::pos2(1436.0, 896.0);
    h.hover_at(away);
    h.run_steps(2);
    let base = snapshot(h);
    let targets: Vec<(String, egui::Pos2)> = base
        .iter()
        .filter(|(_, r, rect)| CLICKABLE.contains(r) && rect.width() > 4.0 && rect.height() > 4.0 && region.is_none_or(|g| g.contains_rect(*rect)))
        .take(limit)
        .map(|(k, _, rect)| (k.clone(), rect.center()))
        .collect();
    let mut found = Vec::new();
    for (name, pos) in targets {
        h.hover_at(pos);
        h.run_steps(4);
        let now: std::collections::HashMap<String, egui::Rect> = snapshot(h).into_iter().map(|(k, _, r)| (k, r)).collect();
        for (k, _, before) in &base {
            if let Some(after) = now.get(k) {
                let d = (after.min - before.min).abs().max((after.size() - before.size()).abs());
                if d.x > 0.4 || d.y > 0.4 {
                    found.push(format!(
                        "{scene}: hovering {name} moved {k} by ({:.1}, {:.1})",
                        after.min.x - before.min.x,
                        after.min.y - before.min.y
                    ));
                }
            }
        }
        h.hover_at(away);
        h.run_steps(2);
    }
    found
}

fn scene(a: WordApp) -> egui_kittest::Harness<'static, WordApp> {
    harness(a)
}

const DIALOGS: &[&str] = &[
    "font",
    "paragraph",
    "tabs",
    "borders",
    "find",
    "replace",
    "goto",
    "insertTable",
    "pageSetup",
    "link",
    "bookmark",
    "wordCount",
    "zoom",
    "watermark",
    "phonetic",
    "newStyle",
    "modifyStyle",
    "manageStyles",
    "newTableStyle",
    "modifyTableStyle",
    "commands",
    "icons",
    "pasteSpecial",
    "about",
    "recipientList",
    "selectTable",
    "insertMergeField",
    "findRecipient",
    "fieldCode",
    "matchFields",
    "checkErrors",
    "tableProperties",
    "defineList",
    "trackChangesOptions",
    "customizeKeyboard",
    "customizeRibbon",
    "controlProperties",
    "enclose",
    "noteOptions",
    "templates",
    "organizer",
];
const OPTIONS: &[&str] = &["general", "display", "proofing", "save", "language", "accessibility", "advanced", "agents"];
const RIBBON_TABS: &[&str] =
    &["Home", "Insert", "Draw", "Design", "Layout", "References", "Mailings", "Review", "View", "Developer", "Help", "Table Layout"];
const FILE_PAGES: &[&str] = &["home", "new", "open", "info", "saveAs", "print", "export", "options"];

fn sweep_all(limit: usize) -> Vec<String> {
    let mut found = Vec::new();
    for name in DIALOGS {
        let mut a = app();
        if a.run("ui.dialog", json!({"name": name})).is_err() || a.dialog.is_none() {
            continue;
        }
        let mut h = scene(a);
        found.extend(sweep(&format!("dialog {name}"), &mut h, limit, Some(dialog_area())));
    }
    for pane in OPTIONS {
        let mut a = app();
        a.run("ui.backstage", json!({"value": true, "page": "options"})).unwrap();
        a.ui.options_pane = (*pane).into();
        let mut h = scene(a);
        found.extend(sweep(&format!("options {pane}"), &mut h, limit, None));
    }
    for tab in RIBBON_TABS {
        let mut a = app();
        a.run("ui.tab", json!({"tab": tab})).unwrap();
        let mut h = scene(a);
        found.extend(sweep(
            &format!("ribbon {tab}"),
            &mut h,
            limit,
            Some(egui::Rect::from_min_max(egui::pos2(0.0, 30.0), egui::pos2(1440.0, 200.0))),
        ));
    }
    for page in FILE_PAGES {
        let mut a = app();
        a.run("ui.backstage", json!({"value": true, "page": page})).unwrap();
        let mut h = scene(a);
        found.extend(sweep(&format!("file {page}"), &mut h, limit, None));
    }
    found
}

/// The Paragraph dialog's tab strip (the case reported): hovering a tab moves nothing.
#[test]
fn hovering_a_paragraph_dialog_tab_moves_nothing() {
    let mut a = app();
    a.run("ui.dialog", json!({"name": "paragraph"})).unwrap();
    let mut h = scene(a);
    let found = sweep("paragraph dialog", &mut h, 40, Some(dialog_area()));
    assert!(found.is_empty(), "{found:#?}");
}

/// Every dialog, Options page and File page (slow: run with `--ignored`).
#[test]
#[ignore]
fn hovering_any_control_moves_nothing() {
    let found = sweep_all(30);
    for f in &found {
        eprintln!("MOVED {f}");
    }
    assert!(found.is_empty(), "{} controls moved; first: {:?}", found.len(), found.first());
}
