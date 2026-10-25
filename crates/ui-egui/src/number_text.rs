//! The "Enter formatting for number" box of Define New Multilevel List. A list level's number text
//! is kept as the file format has it (`%1.%2.`: `%` and a level stands for that level's number), but
//! the box shows each level's number as a grey chip with its sample number (`1`, `a`, `๑`), as Word
//! does, so nobody has to know about `%`. Typing adds plain text; a chip is added from Include level
//! number from, and Backspace or Delete takes a whole chip away.

use egui::{Event, Key, Response, Sense, Ui, vec2};

/// One piece of a level's number text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// A character shown as typed.
    Text(char),
    /// The number of a level, 1–9.
    Level(u8),
}

/// The pieces of `text`: `%` and a digit 1–9 is that level's number; anything else is text.
pub fn parse(text: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%'
            && let Some(d) = chars.peek().and_then(|d| d.to_digit(10)).filter(|d| (1..=9).contains(d))
        {
            chars.next();
            out.push(Piece::Level(d as u8));
        } else {
            out.push(Piece::Text(c));
        }
    }
    out
}

/// `pieces` back as the file format's text (`%1.%2.`).
pub fn to_text(pieces: &[Piece]) -> String {
    pieces
        .iter()
        .map(|p| match p {
            Piece::Text(c) => c.to_string(),
            Piece::Level(l) => format!("%{l}"),
        })
        .collect()
}

/// Type `s` at `cursor`. `%` is left out: followed by a digit it would turn the text into a level's
/// number, which only Include level number from adds. Line breaks and other controls too.
pub fn type_text(pieces: &mut Vec<Piece>, cursor: &mut usize, s: &str) -> bool {
    let mut changed = false;
    for c in s.chars().filter(|c| *c != '%' && !c.is_control()) {
        let at = (*cursor).min(pieces.len());
        pieces.insert(at, Piece::Text(c));
        *cursor = at + 1;
        changed = true;
    }
    changed
}

/// Put level `level`'s number at `cursor`.
pub fn insert_level(pieces: &mut Vec<Piece>, cursor: &mut usize, level: u8) {
    let at = (*cursor).min(pieces.len());
    pieces.insert(at, Piece::Level(level.clamp(1, 9)));
    *cursor = at + 1;
}

/// One key: Backspace and Delete take away the piece before or after the cursor (a whole chip);
/// the arrows, Home and End move it. Whether the text changed.
pub fn key(pieces: &mut Vec<Piece>, cursor: &mut usize, k: Key) -> bool {
    *cursor = (*cursor).min(pieces.len());
    match k {
        Key::Backspace if *cursor > 0 => {
            *cursor -= 1;
            pieces.remove(*cursor);
            true
        }
        Key::Delete if *cursor < pieces.len() => {
            pieces.remove(*cursor);
            true
        }
        Key::ArrowLeft => {
            *cursor = cursor.saturating_sub(1);
            false
        }
        Key::ArrowRight => {
            *cursor = (*cursor + 1).min(pieces.len());
            false
        }
        Key::Home => {
            *cursor = 0;
            false
        }
        Key::End => {
            *cursor = pieces.len();
            false
        }
        _ => false,
    }
}

fn cursor_id(id: egui::Id) -> egui::Id {
    id.with("cursor")
}

/// Put level `level`'s number where the cursor of the box `id` is (at the end if it has none yet).
pub fn insert_level_at_cursor(ui: &Ui, id: egui::Id, text: &mut String, level: u8) {
    let mut pieces = parse(text);
    let mut cursor = ui.data(|d| d.get_temp::<usize>(cursor_id(id))).unwrap_or(pieces.len());
    insert_level(&mut pieces, &mut cursor, level);
    ui.data_mut(|d| d.insert_temp(cursor_id(id), cursor));
    *text = to_text(&pieces);
}

/// The box: `text` with level numbers as grey chips labelled by `chip` (a level's sample number).
/// `id` names it, so Include level number from can add a chip where the cursor is.
pub fn show(ui: &mut Ui, id: egui::Id, label: &str, text: &mut String, width: f32, chip: impl Fn(u8) -> String) -> Response {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let height = ui.spacing().interact_size.y.max(font.size + 8.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let mut resp = ui.interact(rect, id, Sense::click());
    // Read aloud as the number it makes (`1.1.`), named by `label`.
    let shown: String = parse(text)
        .iter()
        .map(|p| match p {
            Piece::Text(c) => c.to_string(),
            Piece::Level(l) => chip(*l),
        })
        .collect();
    resp.widget_info(|| {
        let mut info = egui::WidgetInfo::text_edit(ui.is_enabled(), shown.clone(), shown.clone(), "");
        info.label = Some(label.to_string());
        info
    });
    if resp.clicked() {
        resp.request_focus();
    }
    let focused = resp.has_focus();
    if focused {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(id, egui::EventFilter { horizontal_arrows: true, ..Default::default() });
        });
    }
    let mut pieces = parse(text);
    let mut cursor = ui.data(|d| d.get_temp::<usize>(cursor_id(id))).unwrap_or(pieces.len()).min(pieces.len());

    // Lay the pieces out: the x where each one starts, then the end.
    let pad = 6.0;
    let gap = 2.0;
    let ink = ui.visuals().text_color();
    let chips: Vec<Option<std::sync::Arc<egui::Galley>>> = pieces
        .iter()
        .map(|p| match p {
            Piece::Level(l) => Some(ui.painter().layout_no_wrap(chip(*l), font.clone(), ink)),
            Piece::Text(_) => None,
        })
        .collect();
    let texts: Vec<Option<std::sync::Arc<egui::Galley>>> = pieces
        .iter()
        .map(|p| match p {
            Piece::Text(c) => Some(ui.painter().layout_no_wrap(c.to_string(), font.clone(), ink)),
            Piece::Level(_) => None,
        })
        .collect();
    let mut xs = Vec::with_capacity(pieces.len() + 1);
    let mut x = rect.left() + pad;
    for (i, p) in pieces.iter().enumerate() {
        xs.push(x);
        x += match p {
            Piece::Level(_) => chips[i].as_ref().map_or(0.0, |g| g.size().x) + 8.0 + gap,
            Piece::Text(_) => texts[i].as_ref().map_or(0.0, |g| g.size().x),
        };
    }
    xs.push(x);

    // A click puts the cursor at the nearest boundary.
    if resp.clicked()
        && let Some(at) = resp.interact_pointer_pos()
    {
        cursor = xs.iter().enumerate().min_by(|a, b| (a.1 - at.x).abs().total_cmp(&(b.1 - at.x).abs())).map_or(pieces.len(), |(i, _)| i);
    }

    // Keys and typing while focused.
    let mut changed = false;
    if focused {
        for ev in ui.input(|i| i.events.clone()) {
            match ev {
                Event::Text(s) | Event::Paste(s) => changed |= type_text(&mut pieces, &mut cursor, &s),
                Event::Ime(egui::ImeEvent::Commit(s)) => changed |= type_text(&mut pieces, &mut cursor, &s),
                Event::Key { key: k, pressed: true, modifiers, .. } if !modifiers.command => changed |= key(&mut pieces, &mut cursor, k),
                _ => {}
            }
        }
    }
    ui.data_mut(|d| d.insert_temp(cursor_id(id), cursor));
    if changed {
        *text = to_text(&pieces);
        resp.mark_changed();
        // Draw what was just typed this frame.
        ui.ctx().request_repaint();
    }

    // Paint: the box, the text, the chips, the cursor.
    let v = ui.visuals();
    let stroke = if focused { v.selection.stroke } else { v.widgets.inactive.bg_stroke };
    let p = ui.painter_at(rect.expand(1.0));
    p.rect(rect, v.widgets.inactive.corner_radius, v.text_edit_bg_color(), stroke, egui::StrokeKind::Inside);
    if !changed {
        let mid = rect.center().y;
        let chip_fill = if v.dark_mode { egui::Color32::from_gray(80) } else { egui::Color32::from_gray(218) };
        for (i, piece) in pieces.iter().enumerate() {
            let x0 = xs[i];
            match piece {
                Piece::Level(_) => {
                    if let Some(g) = &chips[i] {
                        let r = egui::Rect::from_min_size(egui::pos2(x0, mid - g.size().y / 2.0 - 1.0), vec2(g.size().x + 8.0, g.size().y + 2.0));
                        p.rect_filled(r, 3.0, chip_fill);
                        p.galley(egui::pos2(x0 + 4.0, mid - g.size().y / 2.0), g.clone(), ink);
                    }
                }
                Piece::Text(_) => {
                    if let Some(g) = &texts[i] {
                        p.galley(egui::pos2(x0, mid - g.size().y / 2.0), g.clone(), ink);
                    }
                }
            }
        }
        if focused && let Some(cx) = xs.get(cursor) {
            let h = font.size + 2.0;
            p.line_segment([egui::pos2(*cx, mid - h / 2.0), egui::pos2(*cx, mid + h / 2.0)], egui::Stroke::new(1.5, ink));
        }
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_with_level_numbers_round_trips() {
        for t in ["%1.", "%1.%2.", "ข้อ %1", "(%1)", "100%", "%0x", "%%1", ""] {
            assert_eq!(to_text(&parse(t)), t, "{t}");
        }
        assert_eq!(parse("%1.%2."), [Piece::Level(1), Piece::Text('.'), Piece::Level(2), Piece::Text('.')]);
        // A % not followed by a level stays text.
        assert_eq!(parse("5%"), [Piece::Text('5'), Piece::Text('%')]);
    }

    /// Typing adds text where the cursor is but never a `%`; Backspace takes a whole level number
    /// at once; a level's number goes in at the cursor.
    #[test]
    fn editing_treats_a_level_number_as_one_piece() {
        let mut p = parse("%1.%2.");
        let mut c = p.len();
        assert!(key(&mut p, &mut c, Key::Backspace));
        assert!(key(&mut p, &mut c, Key::Backspace));
        assert_eq!(to_text(&p), "%1.", "Backspace took the dot, then the whole %2");
        assert!(!type_text(&mut p, &mut c, "%"), "% isn't typed");
        type_text(&mut p, &mut c, "-");
        insert_level(&mut p, &mut c, 2);
        assert_eq!(to_text(&p), "%1.-%2");
        key(&mut p, &mut c, Key::Home);
        type_text(&mut p, &mut c, "ข้อ ");
        assert_eq!(to_text(&p), "ข้อ %1.-%2");
        key(&mut p, &mut c, Key::End);
        assert!(!key(&mut p, &mut c, Key::Delete), "nothing after the end");
        let mut c = 99;
        assert!(!key(&mut p, &mut c, Key::ArrowRight));
        assert_eq!(c, p.len(), "a stale cursor is kept inside the text");
    }
}
