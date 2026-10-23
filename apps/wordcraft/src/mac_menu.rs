//! The native macOS menu bar: the app menu, File, Edit, Insert, Format, View, Window and Help,
//! over the app's own commands. The ribbon and the File panel stay as they are.
//!
//! A menu item with a keyboard shortcut (⌘B, ⌘C…) takes that key from AppKit before the window
//! sees it. So such an item hands the app the same key press instead of running its command
//! itself: the app then does exactly what the key does now, whatever has the focus (a dialog's text
//! box gets ⌘C, not the document). Items without a shortcut run their command, as the ribbon does.

use std::collections::HashMap;
use std::sync::mpsc;

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use serde_json::{Value, json};
use wordcraft_ui_egui::WordApp;

/// What a menu item does when chosen.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Press this key in the app (the item's shortcut).
    Key(egui::Key, egui::Modifiers),
    /// The clipboard's Cut, Copy and Paste, as the keys give them.
    Cut,
    Copy,
    Paste,
    /// Run a command, as a ribbon button does.
    Run(&'static str, Value),
    /// Open a web page in the browser.
    Open(&'static str),
    /// Quit: close the window, which asks first about unsaved changes.
    Quit,
}

/// The menu bar, kept alive for the app's lifetime, and what each item does.
pub struct MacMenu {
    _menu: Menu,
    actions: HashMap<String, Action>,
    chosen: mpsc::Receiver<String>,
}

/// An item: label, shortcut (as the command registry writes it: `Mod+Shift+S`) and what it does.
/// With a shortcut and no action, the action is pressing the shortcut.
struct Entry {
    label: &'static str,
    key: Option<&'static str>,
    action: Option<Action>,
}

fn key(label: &'static str, key: &'static str) -> Option<Entry> {
    Some(Entry { label, key: Some(key), action: None })
}
fn run(label: &'static str, id: &'static str, params: Value) -> Option<Entry> {
    Some(Entry { label, key: None, action: Some(Action::Run(id, params)) })
}
fn dialog(label: &'static str, name: &'static str) -> Option<Entry> {
    run(label, "ui.dialog", json!({ "name": name }))
}
fn with(label: &'static str, key: &'static str, action: Action) -> Option<Entry> {
    Some(Entry { label, key: Some(key), action: Some(action) })
}
/// A separator.
const SEP: Option<Entry> = None;

/// The menus, in menu bar order (the app menu and Window are added around them).
fn menus() -> Vec<(&'static str, Vec<Option<Entry>>)> {
    vec![
        (
            "File",
            vec![
                key("New", "Mod+N"),
                key("Open…", "Mod+O"),
                SEP,
                key("Close", "Mod+W"),
                key("Save", "Mod+S"),
                key("Save As…", "Mod+Shift+S"),
                SEP,
                run("Export as PDF…", "ui.backstage", json!({"value": true, "page": "export"})),
                SEP,
                dialog("Page Setup…", "pageSetup"),
                key("Print…", "Mod+P"),
            ],
        ),
        (
            "Edit",
            vec![
                key("Undo", "Mod+Z"),
                key("Redo", "Mod+Shift+Z"),
                SEP,
                with("Cut", "Mod+X", Action::Cut),
                with("Copy", "Mod+C", Action::Copy),
                with("Paste", "Mod+V", Action::Paste),
                key("Paste Special…", "Mod+Alt+V"),
                key("Paste and Match Formatting", "Mod+Shift+Alt+V"),
                key("Select All", "Mod+A"),
                SEP,
                key("Find…", "Mod+F"),
                dialog("Replace…", "replace"),
                key("Go To…", "Mod+Alt+G"),
            ],
        ),
        (
            "Insert",
            vec![
                key("Page Break", "Mod+Enter"),
                run("Section Break (Next Page)", "layout.break", json!({"kind": "nextPage"})),
                SEP,
                dialog("Table…", "insertTable"),
                run("Picture…", "insert.picture", json!({})),
                run("Text Box", "insert.textBox", json!({})),
                SEP,
                key("Link…", "Mod+K"),
                dialog("Bookmark…", "bookmark"),
                run("Cross-reference…", "insert.crossReference", json!({})),
                SEP,
                key("Comment", "Mod+Alt+A"),
                key("Footnote", "Mod+Alt+F"),
                SEP,
                run("Header", "insert.header", json!({"preset": "blank"})),
                run("Footer", "insert.footer", json!({"preset": "blank"})),
                dialog("Page Number Position…", "pageNumberPosition"),
                SEP,
                run("Date & Time…", "insert.dateTime", json!({})),
                dialog("Symbol…", "symbol"),
                key("Equation", "Alt+="),
            ],
        ),
        (
            "Format",
            vec![
                key("Font…", "Mod+D"),
                run("Paragraph…", "para.dialog", json!({})),
                run("Bullets and Numbering…", "list.define", json!({})),
                run("Borders and Shading…", "para.borders", json!({})),
                dialog("Columns…", "columns"),
                key("Styles", "Mod+Alt+Shift+S"),
                SEP,
                key("Bold", "Mod+B"),
                key("Italic", "Mod+I"),
                key("Underline", "Mod+U"),
                key("Increase Font Size", "Mod+Shift+."),
                key("Decrease Font Size", "Mod+Shift+,"),
                run("Clear Formatting", "format.clear", json!({})),
                SEP,
                key("Align Left", "Mod+L"),
                key("Center", "Mod+E"),
                key("Align Right", "Mod+R"),
                key("Justify", "Mod+J"),
                run("Thai Distribution", "para.thaiDistribute", json!({})),
            ],
        ),
        (
            "View",
            vec![
                key("Print Layout", "Mod+Alt+P"),
                run("Web Layout", "view.webLayout", json!({})),
                key("Outline", "Mod+Alt+O"),
                key("Draft", "Mod+Alt+N"),
                run("Read Mode", "view.readMode", json!({})),
                run("Focus", "view.focus", json!({})),
                SEP,
                run("Ruler", "view.ruler", json!({})),
                run("Navigation Pane", "view.navigationPane", json!({})),
                key("Formatting Marks", "Mod+8"),
                SEP,
                dialog("Zoom…", "zoom"),
                run("Zoom In", "view.zoomIn", json!({})),
                run("Zoom Out", "view.zoomOut", json!({})),
                run("Actual Size", "view.zoom100", json!({})),
                SEP,
                key("Enter Full Screen", "Mod+Ctrl+F"),
            ],
        ),
    ]
}

/// The app menu (named after the app by macOS) and Help.
fn app_menu() -> Vec<Option<Entry>> {
    vec![dialog("About KhianKrub", "about"), SEP, with("Settings…", "Mod+,", Action::Run("file.options", json!({})))]
}
fn help_menu() -> Vec<Option<Entry>> {
    vec![
        dialog("Commands", "commands"),
        SEP,
        run("KhianKrub on GitHub", "ui.dialog", Value::Null)
            .map(|e| Entry { action: Some(Action::Open("https://github.com/chanonss/khiankrub")), ..e }),
        run("Report a Problem", "ui.dialog", Value::Null)
            .map(|e| Entry { action: Some(Action::Open("https://github.com/chanonss/khiankrub/issues")), ..e }),
    ]
}

/// A shortcut as the command registry writes it (`Mod+Shift+S`, `Alt+=`, `Mod+Enter`), as a menu
/// accelerator and the key press it stands for. `None` for a key the menu can't show.
pub fn parse_key(s: &str) -> Option<(Accelerator, egui::Key, egui::Modifiers)> {
    let mut mods = Modifiers::empty();
    let mut emods = egui::Modifiers::NONE;
    let mut parts: Vec<&str> = s.split('+').collect();
    // "Mod+Shift++" would split into empty parts; none of ours do.
    let last = parts.pop()?;
    for m in parts {
        match m {
            "Mod" | "Cmd" => {
                mods |= Modifiers::SUPER;
                emods.mac_cmd = true;
                emods.command = true;
            }
            "Shift" => {
                mods |= Modifiers::SHIFT;
                emods.shift = true;
            }
            "Alt" => {
                mods |= Modifiers::ALT;
                emods.alt = true;
            }
            "Ctrl" => {
                mods |= Modifiers::CONTROL;
                emods.ctrl = true;
            }
            _ => return None,
        }
    }
    let (code, key) = match last {
        "Enter" => (Code::Enter, egui::Key::Enter),
        "=" => (Code::Equal, egui::Key::Equals),
        "." => (Code::Period, egui::Key::Period),
        "," => (Code::Comma, egui::Key::Comma),
        "[" => (Code::BracketLeft, egui::Key::OpenBracket),
        "]" => (Code::BracketRight, egui::Key::CloseBracket),
        k if k.len() == 1 => {
            let c = k.chars().next()?.to_ascii_uppercase();
            let key = egui::Key::from_name(&c.to_string())?;
            let code = match c {
                'A'..='Z' => letter_code(c)?,
                '0'..='9' => digit_code(c)?,
                _ => return None,
            };
            (code, key)
        }
        _ => return None,
    };
    Some((Accelerator::new((!mods.is_empty()).then_some(mods), code), key, emods))
}

fn letter_code(c: char) -> Option<Code> {
    use Code::*;
    const L: [Code; 26] = [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW,
        KeyX, KeyY, KeyZ,
    ];
    L.get((c as u32).checked_sub('A' as u32)? as usize).copied()
}

fn digit_code(c: char) -> Option<Code> {
    use Code::*;
    const D: [Code; 10] = [Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9];
    D.get((c as u32).checked_sub('0' as u32)? as usize).copied()
}

impl MacMenu {
    /// Builds the menu bar and makes it the app's. `wake` repaints the window so a choice made
    /// while the app is idle is seen at once.
    pub fn install(wake: impl Fn() + Send + Sync + 'static) -> Option<MacMenu> {
        let (tx, chosen) = mpsc::channel();
        let tx = std::sync::Mutex::new(tx);
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Ok(tx) = tx.lock() {
                let _ = tx.send(e.id().0.clone());
            }
            wake();
        }));
        let mut actions = HashMap::new();
        let mut n = 0usize;
        let mut fill = |menu: &Submenu, entries: Vec<Option<Entry>>| {
            for e in entries {
                let Some(e) = e else {
                    let _ = menu.append(&PredefinedMenuItem::separator());
                    continue;
                };
                let parsed = e.key.and_then(parse_key);
                let action = match (e.action, &parsed) {
                    (Some(a), _) => a,
                    (None, Some((_, k, m))) => Action::Key(*k, *m),
                    (None, None) => continue,
                };
                n += 1;
                let id = format!("m{n}");
                let item = MenuItem::with_id(id.clone(), e.label, true, parsed.map(|(a, _, _)| a));
                if menu.append(&item).is_ok() {
                    actions.insert(id, action);
                }
            }
        };
        let menu = Menu::new();
        let app = Submenu::new("KhianKrub", true);
        fill(&app, app_menu());
        let _ = app.append_items(&[
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
        ]);
        fill(&app, vec![with("Quit KhianKrub", "Mod+Q", Action::Quit)]);
        let _ = menu.append(&app);
        for (title, entries) in menus() {
            let sub = Submenu::new(title, true);
            fill(&sub, entries);
            let _ = menu.append(&sub);
        }
        let window = Submenu::new("Window", true);
        let _ = window.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(Some("Zoom")),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ]);
        let _ = menu.append(&window);
        let help = Submenu::new("Help", true);
        fill(&help, help_menu());
        let _ = menu.append(&help);
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();
        Some(MacMenu { _menu: menu, actions, chosen })
    }

    /// Before a frame: what was chosen since the last one, as key presses for the frame to see.
    pub fn take_keys(&mut self, raw: &mut egui::RawInput, app: &mut WordApp, ctx: &egui::Context) {
        while let Ok(id) = self.chosen.try_recv() {
            let Some(action) = self.actions.get(&id).cloned() else { continue };
            match action {
                Action::Key(key, modifiers) => {
                    for pressed in [true, false] {
                        raw.events.push(egui::Event::Key { key, physical_key: Some(key), pressed, repeat: false, modifiers });
                    }
                }
                Action::Cut => raw.events.push(egui::Event::Cut),
                Action::Copy => raw.events.push(egui::Event::Copy),
                Action::Paste => {
                    // As ⌘V arrives: the clipboard's text, if any, then the key's release, on which
                    // the app pastes a picture when there was no text.
                    let text = app.services.clipboard_text.as_ref().and_then(|f| f()).unwrap_or_default();
                    if !text.is_empty() {
                        raw.events.push(egui::Event::Paste(text));
                    }
                    let cmd = egui::Modifiers { mac_cmd: true, command: true, ..Default::default() };
                    raw.events.push(egui::Event::Key {
                        key: egui::Key::V,
                        physical_key: Some(egui::Key::V),
                        pressed: false,
                        repeat: false,
                        modifiers: cmd,
                    });
                }
                Action::Run(command, params) => {
                    if let Err(e) = app.run(command, params) {
                        app.status(e);
                    }
                }
                Action::Open(url) => {
                    let _ = std::process::Command::new("/usr/bin/open").arg(url).spawn();
                }
                Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shortcut a menu shows is one the menu can stand for (a typo would leave an item
    /// without its shortcut, or with the wrong key), and the app's own keymap has it, so pressing
    /// it through the menu does what the key does.
    #[test]
    fn menu_shortcuts_parse_and_match_the_command_keys() {
        let registry = wordcraft_engine::Session::new(wordcraft_doc::Document::new()).registry.clone();
        let all = menus().into_iter().flat_map(|(_, e)| e).chain(app_menu()).chain(help_menu()).flatten();
        for e in all {
            let Some(k) = e.key else { continue };
            assert!(parse_key(k).is_some(), "{}: {k} parses", e.label);
            if e.action.is_none() {
                assert!(registry.by_shortcut(k).is_some(), "{}: {k} is a command's key", e.label);
            }
        }
        let (_, key, m) = parse_key("Mod+Shift+S").unwrap();
        assert_eq!(key, egui::Key::S);
        assert!(m.command && m.shift && !m.alt);
        assert!(parse_key("Mod+F12").is_none(), "keys the menu can't show are left out");
    }
}
