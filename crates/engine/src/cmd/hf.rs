//! The Header & Footer contextual tab: which section's header or footer the caret is in, Link to
//! Previous, moving between sections, Show Document Text, and the labels drawn on the page
//! ("Header -Section 2-", "Same as Previous").
//!
//! A section's header (or footer) for a slot — default, first page or even pages — is linked to
//! the previous section when the section has none of its own (`HeaderSet` holds `None`; in
//! `.docx` no `w:headerReference` of that type, ECMA-376 §17.10.5). Layout then shows the nearest
//! earlier section's. Unlinking gives the section its own copy, which can then hold its own
//! text and fields; each section's page numbers restart and take their format from its own
//! `w:pgNumType` (`layout.pageNumberFormat`).

use serde_json::{Value, json};
use wordcraft_doc::section::{HeaderSet, SectionProps};
use wordcraft_doc::{PartKind, StoryRef};
use wordcraft_layout::HfSlot;

use crate::{CmdError, CmdResult, CommandSpec, Selection, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("layout.linkToPrevious", "Link to Previous", "Header & Footer › Navigation", link_to_previous)
            .key(super::non_mac("Alt+Shift+R"))
            .params(
                r#"{"value"?: bool (true: use the previous section's; false: give this section its own copy; default: toggle), "footer"?: bool (default: what the caret is in, else the header), "section"?: n (0-based; default: the section being edited, else the caret's)} → {linked, section, story}"#,
            ),
        CommandSpec::new("hf.next", "Next Section", "Header & Footer › Navigation", |s, _| nav(s, 1)).pure(),
        CommandSpec::new("hf.previous", "Previous Section", "Header & Footer › Navigation", |s, _| nav(s, -1)).pure(),
        CommandSpec::new("hf.state", "Header & Footer State", "Header & Footer", |s, _| Ok(state(s).to_json())).pure().params(
            r#"{} → {editing, header, section (0-based), sections, slot: "default|first|even", linked, canLink, differentFirst, differentOddEven, showDocumentText, headerFromTop, footerFromBottom (pt), pageNumberFormat, pageNumberStart}"#,
        ),
        CommandSpec::new("hf.showDocumentText", "Show Document Text", "Header & Footer › Options", |s, v| {
            s.view.hf_document_text = p::bool(v, "value").unwrap_or(!s.view.hf_document_text);
            Ok(json!({"value": s.view.hf_document_text}))
        })
        .pure()
        .params(r#"{"value"?: bool} (false: hide the body text while a header or footer is edited; default: toggle)"#),
    ]
}

/// The header or footer being edited: the page the caret is on, its section and slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HfContext {
    pub page: usize,
    pub section: usize,
    pub header: bool,
    pub slot: HfSlot,
    pub story: u32,
}

/// Where the caret is, when it is in a header or footer that a page shows (the page last
/// navigated to when it is shown on several).
pub fn context(s: &mut Session) -> Option<HfContext> {
    let StoryRef::Part(id) = s.sel.focus.story else { return None };
    let header = match s.doc.parts.get(&id)?.kind {
        PartKind::Header => true,
        PartKind::Footer => false,
        _ => return None,
    };
    let layout = s.layout();
    let shows = |pg: &wordcraft_layout::Page| if header { pg.header_story == Some(id) } else { pg.footer_story == Some(id) };
    let page = match layout.pages.get(s.page_hint) {
        Some(pg) if shows(pg) => s.page_hint,
        _ => layout.pages.iter().position(shows)?,
    };
    let pg = layout.pages.get(page)?;
    Some(HfContext { page, section: pg.section, header, slot: pg.hf_slot, story: id })
}

/// After a change of which header or footer a page uses (Different First Page, Different Odd &
/// Even), the caret goes to the one the page it was on now shows, as in Word.
pub(crate) fn follow_page(s: &mut Session, before: Option<HfContext>) {
    let Some(c) = before else { return };
    // The change is laid out before the command returns.
    s.touch();
    if context(s).is_some_and(|now| now.page == c.page) {
        return;
    }
    let cmd = if c.header { "insert.editHeader" } else { "insert.editFooter" };
    let _ = s.run(cmd, &json!({ "page": c.page }));
}

/// The last body block of section `i` (what `Document::section_mut` takes).
pub fn section_end(s: &Session, i: usize) -> Option<usize> {
    s.doc.sections().get(i).map(|(end, _)| *end)
}

/// The section being edited when the caret is in a header or footer (its last block).
pub(crate) fn editing_section_end(s: &mut Session) -> Option<usize> {
    let c = context(s)?;
    section_end(s, c.section)
}

fn slot_of(set: &HeaderSet, slot: HfSlot) -> Option<u32> {
    match slot {
        HfSlot::Default => set.default,
        HfSlot::First => set.first,
        HfSlot::Even => set.even,
    }
}

fn slot_mut(set: &mut HeaderSet, slot: HfSlot) -> &mut Option<u32> {
    match slot {
        HfSlot::Default => &mut set.default,
        HfSlot::First => &mut set.first,
        HfSlot::Even => &mut set.even,
    }
}

fn set_of(sp: &SectionProps, header: bool) -> &HeaderSet {
    if header { &sp.headers } else { &sp.footers }
}

/// Whether section `section`'s header (or footer) for `slot` is the previous section's: it has
/// none of its own. The first section links to nothing.
pub fn is_linked(s: &Session, section: usize, header: bool, slot: HfSlot) -> bool {
    section > 0 && s.doc.sections().get(section).is_some_and(|(_, sp)| slot_of(set_of(sp, header), slot).is_none())
}

/// The part section `section` shows for `slot`: its own or the nearest earlier section's.
fn inherited(s: &Session, section: usize, header: bool, slot: HfSlot) -> Option<u32> {
    let sections = s.doc.sections();
    sections
        .iter()
        .take(section.saturating_add(1))
        .rev()
        .find_map(|(_, sp)| slot_of(set_of(sp, header), slot))
        .filter(|id| s.doc.parts.contains_key(id))
}

/// What the Header & Footer tab shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HfState {
    pub editing: bool,
    pub header: bool,
    pub section: usize,
    pub sections: usize,
    pub slot: HfSlot,
    pub linked: bool,
    pub can_link: bool,
    pub different_first: bool,
    pub different_odd_even: bool,
    pub show_document_text: bool,
    pub header_from_top: f32,
    pub footer_from_bottom: f32,
    pub page_number_format: String,
    pub page_number_start: Option<u32>,
}

impl HfState {
    pub fn to_json(&self) -> Value {
        json!({
            "editing": self.editing,
            "header": self.header,
            "section": self.section,
            "sections": self.sections,
            "slot": match self.slot { HfSlot::Default => "default", HfSlot::First => "first", HfSlot::Even => "even" },
            "linked": self.linked,
            "canLink": self.can_link,
            "differentFirst": self.different_first,
            "differentOddEven": self.different_odd_even,
            "showDocumentText": self.show_document_text,
            "headerFromTop": self.header_from_top,
            "footerFromBottom": self.footer_from_bottom,
            "pageNumberFormat": self.page_number_format,
            "pageNumberStart": self.page_number_start,
        })
    }
}

/// The state of the header or footer being edited (else of the caret's section's header).
pub fn state(s: &mut Session) -> HfState {
    let ctx = context(s);
    let sections = s.doc.sections().len();
    let (section, header, slot) = match ctx {
        Some(c) => (c.section, c.header, c.slot),
        None => {
            let block = s.sel.focus.path.0.first().copied().unwrap_or(0) as usize;
            let i = if s.sel.focus.story == StoryRef::Body { s.doc.section_index_of(block) } else { 0 };
            (i, true, HfSlot::Default)
        }
    };
    let sp = s.doc.sections().get(section).map(|(_, x)| (*x).clone()).unwrap_or_default();
    HfState {
        editing: ctx.is_some(),
        header,
        section,
        sections,
        slot,
        linked: is_linked(s, section, header, slot),
        can_link: section > 0,
        different_first: sp.title_page,
        different_odd_even: s.doc.settings.even_odd_headers,
        show_document_text: s.view.hf_document_text,
        header_from_top: sp.header,
        footer_from_bottom: sp.footer,
        page_number_format: sp.page_num_format.ooxml().to_string(),
        page_number_start: sp.page_num_start,
    }
}

/// Link to Previous: link the section's header (or footer) for the slot being edited to the
/// previous section's, or give it its own copy to change on its own.
fn link_to_previous(s: &mut Session, v: &Value) -> CmdResult {
    let ctx = context(s);
    let header = match p::bool(v, "footer") {
        Some(f) => !f,
        None => ctx.is_none_or(|c| c.header),
    };
    let (section, slot, page) = match p::u64(v, "section") {
        Some(i) => {
            let i = usize::try_from(i).unwrap_or(usize::MAX);
            if i >= s.doc.sections().len() {
                return Err(CmdError::Params(format!("no section {i}")));
            }
            (i, ctx.map_or(HfSlot::Default, |c| c.slot), None)
        }
        None => match ctx {
            Some(c) => (c.section, c.slot, Some(c.page)),
            None => {
                let block = s.sel.focus.path.0.first().copied().unwrap_or(0) as usize;
                let i = if s.sel.focus.story == StoryRef::Body { s.doc.section_index_of(block) } else { 0 };
                (i, HfSlot::Default, None)
            }
        },
    };
    if section == 0 {
        return Err(CmdError::Disabled("the first section has no previous section to link to".into()));
    }
    let end = section_end(s, section).ok_or_else(|| CmdError::Failed("no such section".into()))?;
    let linked = is_linked(s, section, header, slot);
    let want = p::bool(v, "value").unwrap_or(!linked);
    let editing_it = ctx.is_some_and(|c| c.header == header && c.section == section);
    let story = if want {
        // Link: the section's own goes; the previous section's shows instead.
        if !linked {
            let sect = s.doc.section_mut(end);
            *slot_mut(if header { &mut sect.headers } else { &mut sect.footers }, slot) = None;
            s.touch();
        }
        inherited(s, section, header, slot)
    } else {
        // Unlink: a copy of what the section showed, its own from now on.
        match (linked, inherited(s, section, header, slot)) {
            (false, own) => own,
            (true, shown) => {
                let style = if header { "Header" } else { "Footer" };
                let blocks = shown
                    .and_then(|id| s.doc.parts.get(&id))
                    .map(|part| part.blocks.clone())
                    .unwrap_or_else(|| vec![wordcraft_doc::para_block(wordcraft_doc::Paragraph::new().styled(style))]);
                let id = s.doc.try_add_part(if header { PartKind::Header } else { PartKind::Footer }, blocks)?;
                let sect = s.doc.section_mut(end);
                *slot_mut(if header { &mut sect.headers } else { &mut sect.footers }, slot) = Some(id);
                s.touch();
                Some(id)
            }
        }
    };
    // The caret stays in the header being edited: now the one this section shows.
    if editing_it {
        if let Some(page) = page {
            s.page_hint = page;
        }
        match story {
            Some(id) => s.sel = Selection::caret(s.doc.end_of(StoryRef::Part(id))),
            None => {
                // Linked to a section that has none: an empty one to type in, as Word shows.
                if let Some(page) = page {
                    let cmd = if header { "insert.createHeader" } else { "insert.createFooter" };
                    let _ = s.run(cmd, &json!({ "page": page }));
                }
            }
        }
    }
    let story = if editing_it { context(s).map(|c| c.story).or(story) } else { story };
    Ok(json!({"linked": want, "section": section, "story": story}))
}

/// Previous / Next: the same kind (header or footer) on the first page of the previous or next
/// section, even when that section shows the previous one's.
fn nav(s: &mut Session, dir: i64) -> CmdResult {
    let Some(c) = context(s) else { return Err(CmdError::Disabled("edit a header or footer first".into())) };
    let target = (c.section as i64).saturating_add(dir);
    let n = s.doc.sections().len() as i64;
    if target < 0 || target >= n {
        return Err(CmdError::Disabled(if dir > 0 { "this is the last section" } else { "this is the first section" }.into()));
    }
    let layout = s.layout();
    let page = layout.pages.iter().position(|pg| pg.section as i64 == target).ok_or_else(|| CmdError::Failed("that section has no pages".into()))?;
    let cmd = if c.header { "insert.editHeader" } else { "insert.editFooter" };
    s.run(cmd, &json!({ "page": page }))
}

/// The tags drawn on page `page` while a header or footer is edited: for the header and the
/// footer, the label ("First Page Header -Section 2-") and whether it is "Same as Previous".
pub fn page_labels(s: &Session, layout: &wordcraft_layout::DocLayout, page: usize) -> Option<[(String, bool); 2]> {
    let pg = layout.pages.get(page)?;
    let many = s.doc.sections().len() > 1;
    let kind = match pg.hf_slot {
        HfSlot::Default => "",
        HfSlot::First => "First Page ",
        HfSlot::Even => {
            if s.doc.settings.even_odd_headers {
                "Even Page "
            } else {
                ""
            }
        }
    };
    let odd = if pg.hf_slot == HfSlot::Default && s.doc.settings.even_odd_headers { "Odd Page " } else { "" };
    let label = |what: &str| {
        let base = format!("{kind}{odd}{what}");
        if many { format!("{base} -Section {}-", pg.section + 1) } else { base }
    };
    Some([(label("Header"), is_linked(s, pg.section, true, pg.hf_slot)), (label("Footer"), is_linked(s, pg.section, false, pg.hf_slot))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Two sections, each two pages: "Front" then "Body".
    fn two_sections() -> Session {
        let mut s = Session::new(wordcraft_doc::Document::default());
        s.run("text.insert", &json!({"text": "Front"})).unwrap();
        s.run("insert.pageBreak", &json!({})).unwrap();
        s.run("text.insert", &json!({"text": "Front 2"})).unwrap();
        s.run("layout.break", &json!({"kind": "nextPage"})).unwrap();
        s.run("text.insert", &json!({"text": "Body"})).unwrap();
        s.run("insert.pageBreak", &json!({})).unwrap();
        s.run("text.insert", &json!({"text": "Body 2"})).unwrap();
        assert_eq!(s.doc.sections().len(), 2);
        s
    }

    fn page_texts(s: &mut Session, footer: bool) -> Vec<String> {
        let layout = s.layout();
        layout
            .pages
            .iter()
            .map(|pg| {
                let items = if footer { &pg.footer } else { &pg.header };
                items
                    .iter()
                    .flat_map(|it| match it {
                        wordcraft_layout::Placed::Lines { para, .. } => para.shown.iter().map(|x| x.1.clone()).collect(),
                        _ => Vec::new(),
                    })
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn different_first_page_keeps_the_caret_in_the_page_s_header() {
        let mut s = two_sections();
        s.run("insert.editHeader", &json!({"page": 2})).unwrap();
        s.run("text.insert", &json!({"text": "All"})).unwrap();
        s.run("layout.differentFirstPage", &json!({"value": true})).unwrap();
        let c = context(&mut s).unwrap();
        assert_eq!((c.page, c.section, c.slot), (2, 1, HfSlot::First));
        s.run("layout.differentFirstPage", &json!({"value": false})).unwrap();
        let c = context(&mut s).unwrap();
        assert_eq!((c.page, c.section, c.slot), (2, 1, HfSlot::Default));
    }

    #[test]
    fn positioned_page_number_uses_the_header_section_being_edited() {
        let mut s = two_sections();
        s.run("insert.editHeader", &json!({"page": 0})).unwrap();
        s.run("layout.pageNumberPosition", &json!({"fromEdge": 72})).unwrap();
        s.run("hf.next", &json!({})).unwrap();
        s.run("layout.linkToPrevious", &json!({"value": false})).unwrap();
        let own = context(&mut s).unwrap().story;
        s.run("layout.pageNumberPosition", &json!({"fromEdge": 90})).unwrap();
        assert_eq!(context(&mut s).unwrap().section, 1);
        assert_eq!(s.doc.last_section.headers.default, Some(own));
        let value = crate::cmd::insert::page_number_placement(&mut s).unwrap();
        assert!((value["fromEdge"].as_f64().unwrap() - 90.0).abs() < 0.5);
    }

    #[test]
    fn unlinked_section_restarts_its_page_numbers_in_its_own_format() {
        let mut s = two_sections();
        // A page-number footer in section 1 shows on every page (section 2 is linked).
        s.run("insert.editFooter", &json!({"page": 0})).unwrap();
        s.run("insert.footer", &json!({"preset": "pageNumber"})).unwrap();
        let st = state(&mut s);
        assert!(st.editing && !st.header && st.section == 0 && !st.can_link);
        s.run("layout.pageNumberFormat", &json!({"format": "lowerRoman", "start": 1})).unwrap();
        // Section 2 goes on counting (3, 4) in its own format until it restarts.
        assert_eq!(page_texts(&mut s, true), ["i", "ii", "3", "4"].map(String::from));
        // Section 2: Next goes to its footer, which is Same as Previous.
        s.run("hf.next", &json!({})).unwrap();
        let st = state(&mut s);
        assert!(st.section == 1 && st.linked && st.can_link);
        let shared = context(&mut s).unwrap().story;
        // Unlink: its own copy, then numbers from 1 in decimal.
        let r = s.run("layout.linkToPrevious", &json!({"value": false})).unwrap();
        assert_eq!(r["linked"], json!(false));
        let own = context(&mut s).unwrap().story;
        assert_ne!(own, shared);
        assert!(!state(&mut s).linked);
        s.run("layout.pageNumberFormat", &json!({"format": "decimal", "start": 1})).unwrap();
        // Setting the format alone keeps the restart.
        s.run("layout.pageNumberFormat", &json!({"format": "thaiNumbers"})).unwrap();
        assert_eq!(s.doc.sections().get(1).map(|(_, x)| x.page_num_start), Some(Some(1)));
        assert_eq!(page_texts(&mut s, true), ["i", "ii", "๑", "๒"].map(String::from));
        // Section 1 kept roman numerals and its restart.
        assert_eq!(s.doc.sections().first().map(|(_, x)| x.page_num_format.ooxml()), Some("lowerRoman"));
        // Own text in section 2 doesn't touch section 1.
        // Own text in section 2 doesn't touch section 1's footer.
        s.run("text.insert", &json!({"text": " body"})).unwrap();
        let shown = |s: &mut Session| s.layout().pages.iter().map(|pg| pg.footer_story).collect::<Vec<_>>();
        assert_eq!(shown(&mut s), [Some(shared), Some(shared), Some(own), Some(own)]);
        assert!(s.doc.parts.get(&shared).unwrap().blocks.iter().all(|b| b.as_para().is_some_and(|x| !x.text.contains("body"))));
        // Link again: section 1's footer shows on section 2's pages.
        s.run("layout.linkToPrevious", &json!({"value": true})).unwrap();
        assert!(state(&mut s).linked);
        assert_eq!(context(&mut s).unwrap().story, shared);
        assert_eq!(shown(&mut s), [Some(shared); 4]);
        assert_eq!(page_texts(&mut s, true), ["i", "ii", "๑", "๒"].map(String::from));
        // Undo brings the unlinked footer back.
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(shown(&mut s), [Some(shared), Some(shared), Some(own), Some(own)]);
    }

    #[test]
    fn section_commands_in_a_header_act_on_its_section() {
        let mut s = two_sections();
        s.run("insert.editHeader", &json!({"page": 2})).unwrap();
        s.run("hf.position", &json!({"header": 50.0})).unwrap();
        assert_eq!(s.doc.sections().get(1).map(|(_, x)| x.header), Some(50.0));
        assert_ne!(s.doc.sections().first().map(|(_, x)| x.header), Some(50.0));
        s.run("layout.differentFirstPage", &json!({})).unwrap();
        let sections = s.doc.sections();
        assert!(!sections.first().unwrap().1.title_page);
        assert!(sections.get(1).unwrap().1.title_page);
    }

    #[test]
    fn labels_name_the_section_and_same_as_previous() {
        let mut s = two_sections();
        s.run("insert.editHeader", &json!({"page": 0})).unwrap();
        s.run("text.insert", &json!({"text": "H"})).unwrap();
        let layout = s.layout();
        let l = page_labels(&s, &layout, 0).unwrap();
        assert_eq!(l[0], ("Header -Section 1-".to_string(), false));
        let l = page_labels(&s, &layout, 2).unwrap();
        assert_eq!(l[0], ("Header -Section 2-".to_string(), true));
        assert_eq!(l[1], ("Footer -Section 2-".to_string(), true));
    }

    #[test]
    fn link_commands_refuse_where_they_cannot_act() {
        let mut s = two_sections();
        s.run("insert.editHeader", &json!({"page": 0})).unwrap();
        assert!(s.run("layout.linkToPrevious", &json!({})).is_err());
        assert!(s.run("hf.previous", &json!({})).is_err());
        assert!(s.run("layout.linkToPrevious", &json!({"section": 99})).is_err());
        s.run("insert.closeHeader", &json!({})).unwrap();
        assert!(s.run("hf.next", &json!({})).is_err());
        // From the body, by section number: unlink section 2's footer.
        let r = s.run("layout.linkToPrevious", &json!({"section": 1, "footer": true, "value": false})).unwrap();
        assert_eq!(r["linked"], json!(false));
        assert!(s.doc.sections().get(1).unwrap().1.footers.default.is_some());
    }

    #[test]
    fn show_document_text_toggles() {
        let mut s = two_sections();
        assert!(s.view.hf_document_text);
        s.run("hf.showDocumentText", &json!({})).unwrap();
        assert!(!s.view.hf_document_text);
        s.run("hf.showDocumentText", &json!({"value": true})).unwrap();
        assert!(s.view.hf_document_text);
    }
}
