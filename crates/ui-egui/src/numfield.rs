//! A number field you type into.
//!
//! egui's `DragValue` also changes its value when it is dragged sideways and shows a resize cursor
//! over it. This field is an ordinary text box for a number: the pointer over it is the usual text
//! cursor, a click selects the number, and typing replaces it. It takes the same settings as
//! `DragValue` (range, decimals, a unit suffix, a custom formatter and parser) so it replaces it
//! one for one; the up and down arrow keys still step the value by `speed` while it has the focus.

use std::ops::RangeInclusive;

use egui::emath::Numeric;
use egui::{Align, Id, Key, Modifiers, Response, TextEdit, TextStyle, Ui, Widget};

type Formatter<'a> = Box<dyn Fn(f64, RangeInclusive<usize>) -> String + 'a>;
type Parser<'a> = Box<dyn Fn(&str) -> Option<f64> + 'a>;

/// The text being edited, kept between frames while the field has the focus.
#[derive(Clone, Default)]
struct Editing {
    text: String,
    /// The value the text belongs to: when something else changes the value, the text is stale.
    value: f64,
    /// The value when editing began, which Escape goes back to.
    start: f64,
}

pub struct NumField<'a> {
    get_set: Box<dyn FnMut(Option<f64>) -> f64 + 'a>,
    speed: f64,
    range: RangeInclusive<f64>,
    clamp_existing: bool,
    min_decimals: usize,
    max_decimals: Option<usize>,
    suffix: String,
    formatter: Option<Formatter<'a>>,
    parser: Option<Parser<'a>>,
}

impl<'a> NumField<'a> {
    pub fn new<N: Numeric>(value: &'a mut N) -> Self {
        let mut field = Self {
            get_set: Box::new(move |v: Option<f64>| {
                if let Some(v) = v {
                    *value = N::from_f64(v);
                }
                value.to_f64()
            }),
            speed: 1.0,
            range: f64::NEG_INFINITY..=f64::INFINITY,
            clamp_existing: true,
            min_decimals: 0,
            max_decimals: None,
            suffix: String::new(),
            formatter: None,
            parser: None,
        };
        if N::INTEGRAL {
            field.max_decimals = Some(0);
            field.range = N::MIN.to_f64()..=N::MAX.to_f64();
            field.speed = 0.25;
        }
        field
    }

    /// How much the arrow keys change the value (and so how many decimals it shows).
    pub fn speed(mut self, speed: impl Into<f64>) -> Self {
        self.speed = speed.into();
        self
    }

    pub fn range<N: Numeric>(mut self, range: RangeInclusive<N>) -> Self {
        self.range = range.start().to_f64()..=range.end().to_f64();
        self
    }

    /// Whether a value outside the range is pulled into it when the field is drawn (default), or kept
    /// until the user edits it.
    pub fn clamp_existing_to_range(mut self, clamp: bool) -> Self {
        self.clamp_existing = clamp;
        self
    }

    pub fn max_decimals(mut self, decimals: usize) -> Self {
        self.max_decimals = Some(decimals);
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn custom_formatter(mut self, formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String) -> Self {
        self.formatter = Some(Box::new(formatter));
        self
    }

    pub fn custom_parser(mut self, parser: impl 'a + Fn(&str) -> Option<f64>) -> Self {
        self.parser = Some(Box::new(parser));
        self
    }
}

fn clamp_to(value: f64, range: &RangeInclusive<f64>) -> f64 {
    if value.is_nan() { value } else { value.clamp(*range.start(), *range.end()) }
}

/// The plain number a field reads: spaces ignored, a Unicode minus or a decimal comma accepted.
fn parse_plain(text: &str) -> Option<f64> {
    let text: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            if c == '−' {
                '-'
            } else if c == ',' {
                '.'
            } else {
                c
            }
        })
        .collect();
    text.parse::<f64>().ok().filter(|v| v.is_finite())
}

impl Widget for NumField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { mut get_set, speed, range, clamp_existing, min_decimals, max_decimals, suffix, formatter, parser } = self;
        let id: Id = ui.next_auto_id();
        let focused = ui.is_enabled() && ui.memory_mut(|m| m.has_focus(id));
        // The text kept from the last frame; none on the first frame with the focus.
        let stored = ui.data_mut(|d| d.remove_temp::<Editing>(id));
        let stored_start = stored.as_ref().map(|e| e.start);
        let kept = if focused { stored } else { None };
        let gained_early = focused && kept.is_none();

        // How many decimals to show: as many as the step needs, up to the field's maximum.
        let aim = f64::from(ui.input(|i| i.aim_radius()));
        let auto = (aim / speed.abs()).log10().ceil().clamp(0.0, 15.0) as usize;
        let most = max_decimals.unwrap_or(auto + 2).max(min_decimals);
        let auto = auto.clamp(min_decimals, most);

        let before = get_set(None);
        let mut value = before;
        if clamp_existing {
            value = clamp_to(value, &range);
        }
        if value != before {
            get_set(Some(value));
        }
        let format = |v: f64| match &formatter {
            Some(f) => f(v, auto..=most),
            None => egui::emath::format_with_decimals_in_range(v, auto..=most),
        };
        let parse = |t: &str| match &parser {
            Some(p) => p(t),
            None => parse_plain(t),
        };
        let shown = format!("{}{suffix}", format(value));

        // The arrow keys step the value while the field has the focus.
        let mut kept_stale = false;
        if focused {
            let steps = ui.input_mut(|i| {
                i.count_and_consume_key(Modifiers::NONE, Key::ArrowUp) as f64 - i.count_and_consume_key(Modifiers::NONE, Key::ArrowDown) as f64
            });
            if steps != 0.0 {
                value = clamp_to(egui::emath::round_to_decimals(value + speed * steps, auto), &range);
                get_set(Some(value));
                kept_stale = true;
            }
        }
        let shown = if value == before { shown } else { format!("{}{suffix}", format(value)) };

        let editing = kept.filter(|e| e.value == value && !kept_stale);
        // Where Escape goes back to: the value when editing began (kept even in the frame egui took the focus).
        let start = editing.as_ref().map(|e| e.start).or(stored_start).unwrap_or(value);
        let mut text = editing.map_or_else(|| shown.clone(), |e| e.text);

        // The box is as wide as the number with its unit, whether or not it has the focus.
        let font = TextStyle::Button.resolve(ui.style());
        let color = ui.visuals().text_color();
        let text_w = ui.painter().layout_no_wrap(shown, font.clone(), color).size().x;
        let pad = ui.spacing().button_padding;
        let inner_w = (text_w + 6.0).max(ui.spacing().interact_size.x - 2.0 * pad.x);
        let escape_key = ui.input(|i| i.key_pressed(Key::Escape));
        let mut response = ui.add(
            TextEdit::singleline(&mut text)
                .id(id)
                .clip_text(false)
                .horizontal_align(Align::Center)
                .margin(pad)
                .min_size(ui.spacing().interact_size)
                .desired_width(inner_w)
                .font(font),
        );

        // A click gives the field the focus during this frame; a Tab gave it before the frame began.
        let gained = gained_early || (!focused && response.has_focus());
        if gained {
            // The number is selected, so typing replaces it.
            if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                let end = egui::text::CCursor::new(text.chars().count());
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), end)));
                state.store(ui.ctx(), response.id);
            }
        }
        if response.changed()
            && let Some(parsed) = parse(&text)
        {
            value = clamp_to(parsed, &range);
            get_set(Some(value));
        }
        // egui drops the focus itself on Escape, so the key and a focus just lost mean Escape.
        let escape = escape_key && (focused || response.lost_focus());
        if escape {
            // Escape drops what was typed and leaves the field.
            get_set(Some(start));
            ui.data_mut(|d| d.remove_temp::<Editing>(id));
            ui.memory_mut(|m| m.surrender_focus(id));
        } else if response.lost_focus() {
            ui.data_mut(|d| d.remove_temp::<Editing>(id));
            if let Some(parsed) = parse(&text) {
                get_set(Some(clamp_to(parsed, &range)));
            }
        } else if response.has_focus() {
            ui.data_mut(|d| d.insert_temp(id, Editing { text, value: get_set(None), start }));
        }

        if get_set(None) != before {
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Role;
    use egui_kittest::kittest::Queryable;

    use super::NumField;

    fn field(value: f32) -> egui_kittest::Harness<'static, f32> {
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(300.0, 100.0)).build_ui_state(
            |ui, v: &mut f32| {
                ui.add(NumField::new(v).range(0.0..=10.0).max_decimals(2).suffix(" cm"));
            },
            value,
        );
        h.run_steps(3);
        h
    }

    fn input<'a>(h: &'a egui_kittest::Harness<'static, f32>) -> egui_kittest::Node<'a> {
        h.get_by_role(Role::TextInput)
    }

    /// The pointer over the field is the text cursor, not the sideways-resize cursor of a drag value.
    #[test]
    fn the_field_shows_the_normal_text_cursor() {
        let mut h = field(1.0);
        h.hover_at(input(&h).rect().center());
        h.run_steps(3);
        assert_eq!(h.output().platform_output.cursor_icon, egui::CursorIcon::Text);
    }

    /// Dragging across the field does not change the number.
    #[test]
    fn dragging_across_the_field_changes_nothing() {
        let mut h = field(1.0);
        let c = input(&h).rect().center();
        h.event(egui::Event::PointerMoved(c));
        h.event(egui::Event::PointerButton { pos: c, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
        h.run_steps(2);
        for dx in [10.0, 20.0, 40.0] {
            h.event(egui::Event::PointerMoved(c + egui::vec2(dx, 0.0)));
            h.run_steps(2);
        }
        h.event(egui::Event::PointerButton {
            pos: c + egui::vec2(40.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        h.run_steps(3);
        assert_eq!(*h.state(), 1.0);
    }

    /// A click selects the number, typing replaces it, Enter keeps it, and the box shows it with its unit.
    #[test]
    fn typing_replaces_the_number() {
        let mut h = field(1.0);
        assert_eq!(input(&h).value().as_deref(), Some("1 cm"));
        input(&h).click();
        h.run_steps(4);
        h.event(egui::Event::Text("2.5".into()));
        h.run_steps(3);
        assert_eq!(*h.state(), 2.5, "the number follows what is typed");
        h.event(egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
        h.run_steps(4);
        assert_eq!(input(&h).value().as_deref(), Some("2.5 cm"), "after Enter it is shown with its unit again");
    }

    /// Typing past the range keeps the number in it; a decimal comma and a Unicode minus are read.
    #[test]
    fn typed_numbers_stay_in_the_range_and_read_commas() {
        let mut h = field(1.0);
        input(&h).click();
        h.run_steps(4);
        h.event(egui::Event::Text("99".into()));
        h.run_steps(3);
        assert_eq!(*h.state(), 10.0);
        assert_eq!(super::parse_plain("3,5"), Some(3.5));
        assert_eq!(super::parse_plain("−2"), Some(-2.0));
        assert_eq!(super::parse_plain(" 1 2 "), Some(12.0));
        assert_eq!(super::parse_plain("abc"), None);
    }

    /// Escape drops what was typed; the arrow keys step the number.
    #[test]
    fn escape_goes_back_and_the_arrow_keys_step() {
        let mut h = field(1.0);
        input(&h).click();
        h.run_steps(4);
        h.event(egui::Event::Text("7".into()));
        h.run_steps(3);
        assert_eq!(*h.state(), 7.0);
        h.event(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
        h.run_steps(4);
        assert_eq!(*h.state(), 1.0, "Escape goes back to the number before the edit");
        input(&h).click();
        h.run_steps(4);
        h.event(egui::Event::Key { key: egui::Key::ArrowUp, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
        h.run_steps(3);
        assert_eq!(*h.state(), 2.0, "the Up key adds the step");
    }
}
