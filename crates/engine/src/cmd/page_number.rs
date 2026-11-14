//! Hide the page number on one page (right-click the page number › "Hide Page Number on This
//! Page"), as Thai theses do on the first page of each chapter.
//!
//! Nothing here is a layout trick: the page is made the first page of a section of its own with
//! Different First Page, whose first-page header and footer are copies of what the page showed
//! without the page number. Word does the same by hand, so the file opens the same in Word.
//! Numbering carries on through the new section (no restart), and its other headers and footers
//! stay linked to the previous section ("Same as Previous"), so every other page looks as before.
//!
//! - A page that already starts a section only gets Different First Page.
//! - Any other page must start with the start of a paragraph that follows a paragraph (a section
//!   ends on a paragraph mark): the section break goes at the end of the paragraph before. A
//!   page break that ended that paragraph goes, or Word would show a blank page.
//! - The paragraph that held the page number is emptied in the copies, so "Page 3 of 10" goes
//!   as a whole (a page number inside a text box is not found).
//! - Showing it again joins the two sections back when the later one is still as this command
//!   left it (a page break goes back where one was needed); otherwise Different First Page is
//!   turned off on that section.
//! - The next section's first-page header and footer are pinned to what they showed, so a later
//!   section with Different First Page doesn't pick up the copy through Same as Previous.

use std::sync::Arc;

use serde_json::{Value, json};
use wordcraft_doc::section::{SectionProps, SectionStart};
use wordcraft_doc::{Block, Blocks, InlineObject, Paragraph, PartKind, Path, Pos, StoryRef, para_block};
use wordcraft_layout::{DocLayout, HfSlot, ViewMode};

use crate::{CmdError, CmdResult, CommandSpec, Session, p};

const PAGE_BREAK: char = '\u{000C}';
/// Tables in headers nest at most this deep for the search (deeper ones are not searched).
const MAX_DEPTH: usize = 8;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("hf.hidePageNumber", "Hide Page Number on This Page", "Header & Footer › Page Number", run).params(
            r#"{"page": number (0-based, Print Layout), "hide"?: bool (default: toggle)} — hides the page number on that page only (a section of its own with Different First Page; numbering continues), or shows it again"#,
        ),
        CommandSpec::new("hf.pageNumberState", "Page Number on Page", "Header & Footer › Page Number", |s, v| {
            let page = page_param(v)?;
            Ok(json!({"state": state(s, page).as_str()}))
        })
        .pure()
        .params(r#"{"page": number} → {"state": "shown|hidden|none"}"#),
    ]
}

/// What a page shows of its page number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageNumber {
    /// The page's header or footer shows a PAGE field.
    Shown,
    /// Hidden on this page: it starts a section with Different First Page whose first-page
    /// header and footer have no page number, while the section's other pages do.
    Hidden,
    /// No page number to hide or show (or not in Print Layout).
    None,
}

impl PageNumber {
    pub fn as_str(self) -> &'static str {
        match self {
            PageNumber::Shown => "shown",
            PageNumber::Hidden => "hidden",
            PageNumber::None => "none",
        }
    }
}

fn page_param(v: &Value) -> Result<usize, CmdError> {
    p::u64(v, "page").map(|n| usize::try_from(n).unwrap_or(usize::MAX)).ok_or_else(|| CmdError::Params("`page` is required".into()))
}

fn print_layout(s: &mut Session) -> Option<Arc<DocLayout>> {
    (!s.view.read_mode && s.view.mode == ViewMode::Print).then(|| s.layout())
}

fn is_page_field(o: &InlineObject) -> bool {
    matches!(o, InlineObject::Field { instr, .. } if wordcraft_layout::fields::field_name(instr) == "PAGE")
}

fn blocks_have_page(blocks: &Blocks, depth: usize) -> bool {
    depth < MAX_DEPTH
        && blocks.iter().any(|b| match &**b {
            Block::Para(p) => p.objects.iter().any(is_page_field),
            Block::Table(t) => t.rows.iter().any(|r| r.cells.iter().any(|c| blocks_have_page(&c.blocks, depth + 1))),
        })
}

fn part_has_page(s: &Session, id: Option<u32>) -> bool {
    id.and_then(|id| s.doc.parts.get(&id)).is_some_and(|part| blocks_have_page(&part.blocks, 0))
}

/// Empty every paragraph that shows a PAGE field (its paragraph formatting stays).
fn strip_page(blocks: &mut Blocks, depth: usize) {
    if depth >= MAX_DEPTH {
        return;
    }
    for b in blocks.iter_mut() {
        let has = match &**b {
            Block::Para(p) => p.objects.iter().any(is_page_field),
            Block::Table(_) => true,
        };
        if !has {
            continue;
        }
        match Arc::make_mut(b) {
            Block::Para(p) => {
                let n = p.text.len();
                let _ = p.delete(0, n);
            }
            Block::Table(t) => {
                for c in t.rows.iter_mut().flat_map(|r| r.cells.iter_mut()) {
                    strip_page(&mut c.blocks, depth + 1);
                }
            }
        }
    }
}

/// Is `page` the first page of its section?
fn starts_section(layout: &DocLayout, page: usize) -> bool {
    page == 0 || layout.pages.get(page - 1).map(|p| p.section) != layout.pages.get(page).map(|p| p.section)
}

/// What `page` shows of its page number.
pub fn state(s: &mut Session, page: usize) -> PageNumber {
    let Some(layout) = print_layout(s) else { return PageNumber::None };
    let Some(pg) = layout.pages.get(page) else { return PageNumber::None };
    if part_has_page(s, pg.header_story) || part_has_page(s, pg.footer_story) {
        return PageNumber::Shown;
    }
    if pg.hf_slot != HfSlot::First || !starts_section(&layout, page) {
        return PageNumber::None;
    }
    // The section's other pages show a number: the next page of the section, else what its
    // default slot would show.
    let next = layout.pages.get(page + 1).filter(|n| n.section == pg.section);
    let others = match next {
        Some(n) => part_has_page(s, n.header_story) || part_has_page(s, n.footer_story),
        None => {
            let (h, f) = (inherited(s, pg.section, true, HfSlot::Default), inherited(s, pg.section, false, HfSlot::Default));
            part_has_page(s, h) || part_has_page(s, f)
        }
    };
    if others { PageNumber::Hidden } else { PageNumber::None }
}

fn slot_of(sp: &SectionProps, header: bool, slot: HfSlot) -> Option<u32> {
    let set = if header { &sp.headers } else { &sp.footers };
    match slot {
        HfSlot::Default => set.default,
        HfSlot::First => set.first,
        HfSlot::Even => set.even,
    }
}

/// The part section `section` shows for `slot`: its own or the nearest earlier section's.
fn inherited(s: &Session, section: usize, header: bool, slot: HfSlot) -> Option<u32> {
    let sections = s.doc.sections();
    sections.iter().take(section.saturating_add(1)).rev().find_map(|(_, sp)| slot_of(sp, header, slot)).filter(|id| s.doc.parts.contains_key(id))
}

/// A copy of part `shown` without its page number, or an empty one.
fn stripped_copy(s: &mut Session, shown: Option<u32>, header: bool) -> Result<u32, CmdError> {
    let style = if header { "Header" } else { "Footer" };
    let mut blocks = shown.and_then(|id| s.doc.parts.get(&id)).map(|part| part.blocks.clone()).unwrap_or_default();
    strip_page(&mut blocks, 0);
    if blocks.is_empty() {
        blocks.push(para_block(Paragraph::new().styled(style)));
    }
    Ok(s.doc.try_add_part(if header { PartKind::Header } else { PartKind::Footer }, blocks)?)
}

/// What section `section`'s first-page header and footer show, to pin on it (an empty part
/// for one that shows none).
fn first_slots_shown(s: &mut Session, section: usize) -> Result<Option<(u32, u32)>, CmdError> {
    let Some((_, sp)) = s.doc.sections().get(section).map(|(e, sp)| (*e, (*sp).clone())) else { return Ok(None) };
    if sp.headers.first.is_some() && sp.footers.first.is_some() {
        return Ok(None);
    }
    let pin = |s: &mut Session, header: bool| -> Result<u32, CmdError> {
        match inherited(s, section, header, HfSlot::First) {
            Some(id) => Ok(id),
            None => stripped_copy(s, None, header),
        }
    };
    let h = match sp.headers.first {
        Some(id) => id,
        None => pin(s, true)?,
    };
    let f = match sp.footers.first {
        Some(id) => id,
        None => pin(s, false)?,
    };
    Ok(Some((h, f)))
}

/// The props of the section this command starts after a section with props `before`: a new
/// page, Different First Page, numbering carrying on, and everything else linked.
fn chapter_props(before: &SectionProps, first_header: Option<u32>, first_footer: Option<u32>) -> SectionProps {
    let mut x = before.clone();
    x.start = SectionStart::NextPage;
    x.title_page = true;
    x.page_num_start = None;
    x.fmt_change = None;
    x.headers.default = None;
    x.headers.even = None;
    x.footers.default = None;
    x.footers.even = None;
    x.headers.first = first_header;
    x.footers.first = first_footer;
    x
}

fn run(s: &mut Session, v: &Value) -> CmdResult {
    let page = page_param(v)?;
    let Some(layout) = print_layout(s) else {
        return Err(CmdError::Disabled("page numbers are hidden page by page in Print Layout".into()));
    };
    if layout.pages.get(page).is_none() {
        return Err(CmdError::Params(format!("no page {page}")));
    }
    let now = state(s, page);
    let hide = p::bool(v, "hide").unwrap_or(now == PageNumber::Shown);
    match (hide, now) {
        (_, PageNumber::None) => Err(CmdError::Disabled("this page shows no page number".into())),
        (true, PageNumber::Hidden) | (false, PageNumber::Shown) => Ok(json!({"page": page, "state": now.as_str()})),
        (true, PageNumber::Shown) => {
            hide_on(s, &layout, page)?;
            s.touch();
            Ok(json!({"page": page, "state": state(s, page).as_str()}))
        }
        (false, PageNumber::Hidden) => {
            show_on(s, &layout, page)?;
            s.touch();
            Ok(json!({"page": page, "state": state(s, page).as_str()}))
        }
    }
}

fn hide_on(s: &mut Session, layout: &DocLayout, page: usize) -> Result<(), CmdError> {
    let pg = layout.pages.get(page).ok_or_else(|| CmdError::Params(format!("no page {page}")))?;
    let section = pg.section;
    let (shown_h, shown_f) = (pg.header_story, pg.footer_story);
    if starts_section(layout, page) {
        let end = s.doc.sections().get(section).map(|(e, _)| *e).ok_or_else(|| CmdError::Failed("no such section".into()))?;
        let pin = first_slots_shown(s, section + 1)?;
        let (h, f) = (stripped_copy(s, shown_h, true)?, stripped_copy(s, shown_f, false)?);
        if let Some((ph, pf)) = pin {
            pin_first(s, section + 1, ph, pf);
        }
        let sect = s.doc.section_mut(end);
        sect.title_page = true;
        sect.headers.first = Some(h);
        sect.footers.first = Some(f);
        return Ok(());
    }
    // The page must start with the start of a paragraph that follows a paragraph.
    // A paragraph that ends in a page break counts as the page's first block (its mark lands
    // after the break); the paragraph after it is the one that starts the page.
    let mut k = pg.first_block;
    if layout.page_of(&Pos::body(k, 0)) != Some(page)
        && s.doc.body.get(k).and_then(|b| b.as_para()).is_some_and(|p| p.text.ends_with(PAGE_BREAK) && p.section.is_none())
    {
        k += 1;
    }
    let starts_here = layout.page_of(&Pos::body(k, 0)) == Some(page);
    if !starts_here || !matches!(s.doc.body.get(k).map(|b| &**b), Some(Block::Para(_))) {
        return Err(CmdError::Disabled("this page doesn't start with a new paragraph".into()));
    }
    let Some(prev) = k.checked_sub(1) else {
        return Err(CmdError::Disabled("this page doesn't start with a new paragraph".into()));
    };
    if !matches!(s.doc.body.get(prev).map(|b| &**b), Some(Block::Para(p)) if p.section.is_none()) {
        return Err(CmdError::Disabled("the page starts right after a table".into()));
    }
    let before = s.doc.sections().get(section).map(|(_, sp)| (*sp).clone()).ok_or_else(|| CmdError::Failed("no such section".into()))?;
    let pin = first_slots_shown(s, section + 1)?;
    let (h, f) = (stripped_copy(s, shown_h, true)?, stripped_copy(s, shown_f, false)?);
    if let Some((ph, pf)) = pin {
        pin_first(s, section + 1, ph, pf);
    }
    let para = s.doc.para_mut(StoryRef::Body, &Path::top(prev))?;
    if para.text.ends_with(PAGE_BREAK) {
        let n = para.text.len();
        para.delete(n - PAGE_BREAK.len_utf8(), n)?;
    }
    para.section = Some(Box::new(before.clone()));
    *s.doc.section_mut(k) = chapter_props(&before, Some(h), Some(f));
    Ok(())
}

fn pin_first(s: &mut Session, section: usize, h: u32, f: u32) {
    let Some(end) = s.doc.sections().get(section).map(|(e, _)| *e) else { return };
    let sect = s.doc.section_mut(end);
    sect.headers.first.get_or_insert(h);
    sect.footers.first.get_or_insert(f);
}

fn show_on(s: &mut Session, layout: &DocLayout, page: usize) -> Result<(), CmdError> {
    let pg = layout.pages.get(page).ok_or_else(|| CmdError::Params(format!("no page {page}")))?;
    let section = pg.section;
    let sections: Vec<(usize, SectionProps)> = s.doc.sections().into_iter().map(|(e, sp)| (e, sp.clone())).collect();
    let (end, this) = sections.get(section).cloned().ok_or_else(|| CmdError::Failed("no such section".into()))?;
    // Join back with the previous section when this one is still as hiding left it.
    if let Some((prev_end, before)) = section.checked_sub(1).and_then(|i| sections.get(i).cloned())
        && this == chapter_props(&before, this.headers.first, this.footers.first)
    {
        s.doc.para_mut(StoryRef::Body, &Path::top(prev_end))?.section = None;
        *s.doc.section_mut(prev_end) = before;
        s.touch();
        // A page break goes back where the paragraph no longer starts a page by itself.
        let after = s.layout();
        let start = after.page_of(&Pos::body(prev_end + 1, 0));
        let prev_page = after.page_of(&Pos::body(prev_end, s.doc.body.get(prev_end).and_then(|b| b.as_para()).map_or(0, |p| p.text.len())));
        if start.is_some() && start == prev_page {
            let para = s.doc.para_mut(StoryRef::Body, &Path::top(prev_end))?;
            let n = para.text.len();
            let props = para.props_of_char(n.saturating_sub(1)).clone();
            para.insert_text(n, &PAGE_BREAK.to_string(), &props)?;
        }
        return Ok(());
    }
    s.doc.section_mut(end).title_page = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    fn run(s: &mut Session, id: &str, v: Value) -> Value {
        s.run(id, &v).unwrap_or_else(|e| panic!("{id}: {e}"))
    }

    /// Three chapters on pages of their own (page breaks), a footer with the page number.
    fn thesis() -> Session {
        let mut s = Session::new(wordcraft_doc::Document::new());
        for (i, ch) in ["บทที่ 1", "บทที่ 2", "บทที่ 3"].iter().enumerate() {
            if i > 0 {
                run(&mut s, "insert.pageBreak", json!({}));
            }
            run(&mut s, "text.insert", json!({ "text": *ch }));
            for _ in 0..3 {
                run(&mut s, "text.newParagraph", json!({}));
                run(&mut s, "text.insert", json!({"text": "เนื้อหา"}));
            }
        }
        run(&mut s, "insert.editFooter", json!({"page": 0}));
        run(&mut s, "insert.footer", json!({"preset": "pageNumber"}));
        run(&mut s, "insert.closeHeader", json!({}));
        s
    }

    fn numbers(s: &mut Session) -> Vec<String> {
        let layout = s.layout();
        layout
            .pages
            .iter()
            .map(|p| {
                let has = part_has_page(s, p.footer_story) || part_has_page(s, p.header_story);
                if has { p.number.to_string() } else { "-".into() }
            })
            .collect()
    }

    #[test]
    fn hides_the_number_on_one_page_and_numbering_carries_on() {
        let mut s = thesis();
        assert_eq!(numbers(&mut s), ["1", "2", "3"]);
        let r = run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        assert_eq!(r["state"], "hidden");
        assert_eq!(numbers(&mut s), ["1", "-", "3"]);
        assert_eq!(s.doc.sections().len(), 2, "a section of its own");
        // The page break before chapter 2 went (it would leave a blank page in Word).
        assert!(!s.doc.body.iter().any(|b| b.as_para().is_some_and(|p| p.text.contains(PAGE_BREAK) && p.section.is_some())));
        assert_eq!(s.layout().pages.len(), 3, "no blank page");
        run(&mut s, "hf.hidePageNumber", json!({"page": 2}));
        assert_eq!(numbers(&mut s), ["1", "-", "-"]);
        assert_eq!(run(&mut s, "hf.pageNumberState", json!({"page": 0}))["state"], "shown");
    }

    #[test]
    fn showing_it_again_joins_the_sections_and_puts_the_page_break_back() {
        let mut s = thesis();
        let before = s.doc.body.clone();
        run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        let r = run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        assert_eq!(r["state"], "shown");
        assert_eq!(numbers(&mut s), ["1", "2", "3"]);
        assert_eq!(s.doc.sections().len(), 1);
        let texts = |b: &Blocks| b.iter().filter_map(|b| b.as_para().map(|p| p.text.clone())).collect::<Vec<_>>();
        assert_eq!(texts(&s.doc.body), texts(&before), "the page break is back");
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(numbers(&mut s), ["1", "-", "3"], "undo hides it again");
    }

    #[test]
    fn first_page_of_the_document_only_gets_different_first_page() {
        let mut s = thesis();
        run(&mut s, "hf.hidePageNumber", json!({"page": 0, "hide": true}));
        assert_eq!(numbers(&mut s), ["-", "2", "3"]);
        assert_eq!(s.doc.sections().len(), 1);
        run(&mut s, "hf.hidePageNumber", json!({"page": 0, "hide": false}));
        assert_eq!(numbers(&mut s), ["1", "2", "3"]);
    }

    /// Chapters that start pages by themselves: short paragraphs that fill the pages, and a
    /// chapter heading with Page Break Before.
    fn flowing(break_before: bool) -> Session {
        let mut s = Session::new(wordcraft_doc::Document::new());
        for i in 0..120 {
            if i > 0 {
                run(&mut s, "text.newParagraph", json!({}));
            }
            run(&mut s, "text.insert", json!({ "text": format!("ย่อหน้า {i}") }));
            if break_before && i == 70 {
                run(&mut s, "para.pageBreakBefore", json!({"value": true}));
            }
        }
        run(&mut s, "insert.editFooter", json!({"page": 0}));
        run(&mut s, "insert.footer", json!({"preset": "pageNumber"}));
        run(&mut s, "insert.closeHeader", json!({}));
        s
    }

    #[test]
    fn a_page_that_starts_by_itself_keeps_its_text_where_it_was() {
        for break_before in [false, true] {
            let mut s = flowing(break_before);
            let pages = s.layout().pages.len();
            assert!(pages >= 3, "{pages} pages");
            let firsts: Vec<usize> = s.layout().pages.iter().map(|p| p.first_block).collect();
            let before = s.doc.body.iter().filter_map(|b| b.as_para().map(|p| p.text.clone())).collect::<Vec<_>>();
            let target = if break_before { s.layout().page_of(&Pos::body(70, 0)).unwrap() } else { 1 };
            run(&mut s, "hf.hidePageNumber", json!({"page": target}));
            let numbers_now = numbers(&mut s);
            assert_eq!(numbers_now.get(target).map(String::as_str), Some("-"));
            assert_eq!(s.layout().pages.len(), pages, "no blank page (break before: {break_before})");
            assert_eq!(s.layout().pages.iter().map(|p| p.first_block).collect::<Vec<_>>(), firsts);
            run(&mut s, "hf.hidePageNumber", json!({"page": target}));
            let after = s.doc.body.iter().filter_map(|b| b.as_para().map(|p| p.text.clone())).collect::<Vec<_>>();
            assert_eq!(after, before, "no page break added");
            assert_eq!(s.doc.sections().len(), 1);
        }
    }

    #[test]
    fn a_page_starting_mid_paragraph_is_refused() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        run(&mut s, "text.insert", json!({"text": "คำ ".repeat(2500)}));
        run(&mut s, "insert.editFooter", json!({"page": 0}));
        run(&mut s, "insert.footer", json!({"preset": "pageNumber"}));
        run(&mut s, "insert.closeHeader", json!({}));
        assert!(s.layout().pages.len() > 1);
        assert!(matches!(s.run("hf.hidePageNumber", &json!({"page": 1})), Err(CmdError::Disabled(_))));
        assert_eq!(numbers(&mut s)[1], "2");
    }

    #[test]
    fn no_page_number_no_command() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        run(&mut s, "text.insert", json!({"text": "x"}));
        assert_eq!(run(&mut s, "hf.pageNumberState", json!({"page": 0}))["state"], "none");
        assert!(s.run("hf.hidePageNumber", &json!({"page": 0})).is_err());
        assert!(s.run("hf.hidePageNumber", &json!({"page": 99})).is_err());
        assert!(s.run("hf.hidePageNumber", &json!({})).is_err());
    }

    #[test]
    fn a_later_title_page_section_keeps_what_it_showed() {
        let mut s = thesis();
        // Chapter 3 is a section of the user's own with Different First Page and no first footer.
        run(&mut s, "hf.hidePageNumber", json!({"page": 2}));
        run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        assert_eq!(numbers(&mut s), ["1", "-", "-"]);
        run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        assert_eq!(numbers(&mut s), ["1", "2", "-"]);
    }

    #[test]
    fn hidden_page_survives_docx() {
        let mut s = thesis();
        run(&mut s, "hf.hidePageNumber", json!({"page": 1}));
        let bytes = wordcraft_docx::write(&s.doc).unwrap();
        let doc = wordcraft_docx::read(&bytes).unwrap();
        let mut r = Session::new(doc);
        assert_eq!(numbers(&mut r), ["1", "-", "3"]);
        assert_eq!(run(&mut r, "hf.pageNumberState", json!({"page": 1}))["state"], "hidden");
        run(&mut r, "hf.hidePageNumber", json!({"page": 1}));
        assert_eq!(numbers(&mut r), ["1", "2", "3"]);
    }
}
