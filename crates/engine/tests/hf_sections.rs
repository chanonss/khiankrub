//! Header & Footer › Link to Previous with per-section page numbering, end to end: built with
//! commands, saved as `.docx` (ECMA-376: one `w:headerReference`/`w:footerReference` per type a
//! section has of its own, none when linked to the previous; `w:pgNumType` start and format) and
//! opened again with the same pages and numbers.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use serde_json::json;
use wordcraft_engine::{Session, io};

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
    xml
}

/// The PAGE results each page's footer shows.
fn footer_numbers(s: &mut Session) -> Vec<String> {
    s.layout()
        .pages
        .iter()
        .map(|pg| {
            pg.footer
                .iter()
                .flat_map(|it| match it {
                    wordcraft_layout::Placed::Lines { para, .. } => para.shown.iter().map(|x| x.1.clone()).collect(),
                    _ => Vec::new(),
                })
                .collect::<String>()
        })
        .collect()
}

/// Front matter in roman numerals from i, then the body in decimal from 1, its footer unlinked.
fn thesis() -> Session {
    let mut s = Session::new(wordcraft_doc::Document::default());
    for (i, text) in ["คำนำ", "สารบัญ"].iter().enumerate() {
        if i > 0 {
            s.run("insert.pageBreak", &json!({})).unwrap();
        }
        s.run("text.insert", &json!({"text": text})).unwrap();
    }
    s.run("layout.break", &json!({"kind": "nextPage"})).unwrap();
    for (i, text) in ["บทที่ 1", "บทที่ 2", "บทที่ 3"].iter().enumerate() {
        if i > 0 {
            s.run("insert.pageBreak", &json!({})).unwrap();
        }
        s.run("text.insert", &json!({"text": text})).unwrap();
    }
    s.run("insert.editFooter", &json!({"page": 0})).unwrap();
    s.run("insert.footer", &json!({"preset": "pageNumber"})).unwrap();
    s.run("layout.pageNumberFormat", &json!({"format": "lowerRoman", "start": 1})).unwrap();
    s.run("hf.next", &json!({})).unwrap();
    s.run("layout.linkToPrevious", &json!({"value": false})).unwrap();
    s.run("layout.pageNumberFormat", &json!({"format": "decimal", "start": 1})).unwrap();
    s.run("insert.closeHeader", &json!({})).unwrap();
    s
}

#[test]
fn restarted_page_numbers_round_trip_through_docx() {
    let mut s = thesis();
    assert_eq!(footer_numbers(&mut s), ["i", "ii", "1", "2", "3"]);

    let bytes = io::save_bytes("thesis.docx", &s.doc).unwrap();
    let xml = document_xml(&bytes);
    let secs: Vec<&str> = xml.split("<w:sectPr").skip(1).map(|x| x.split("</w:sectPr>").next().unwrap_or("")).collect();
    assert_eq!(secs.len(), 2, "{xml}");
    // Each section has a footer of its own and its own numbering.
    for sec in &secs {
        assert!(sec.contains(r#"<w:footerReference w:type="default""#), "{sec}");
        assert!(sec.contains(r#"w:start="1""#), "{sec}");
    }
    assert!(secs[0].contains(r#"w:fmt="lowerRoman""#), "{}", secs[0]);
    assert!(!secs[1].contains("lowerRoman"), "{}", secs[1]);
    // Two different footer parts.
    let refs: Vec<&str> = secs.iter().filter_map(|x| x.split(r#"<w:footerReference w:type="default" r:id=""#).nth(1)?.split('"').next()).collect();
    assert_eq!(refs.len(), 2);
    assert_ne!(refs[0], refs[1]);

    let mut back = Session::new(io::open_bytes("thesis.docx", &bytes).unwrap());
    assert_eq!(footer_numbers(&mut back), ["i", "ii", "1", "2", "3"]);
    let sections = back.doc.sections();
    assert!(sections.iter().all(|(_, x)| x.footers.default.is_some()));
    assert_ne!(sections[0].1.footers.default, sections[1].1.footers.default);
}

#[test]
fn a_linked_footer_saves_no_reference_and_continues_numbering() {
    let mut s = thesis();
    s.run("insert.editFooter", &json!({"page": 2})).unwrap();
    s.run("layout.linkToPrevious", &json!({"value": true})).unwrap();
    s.run("layout.pageNumberFormat", &json!({"start": null})).unwrap();
    s.run("insert.closeHeader", &json!({})).unwrap();
    assert_eq!(footer_numbers(&mut s), ["i", "ii", "3", "4", "5"]);

    let bytes = io::save_bytes("linked.docx", &s.doc).unwrap();
    let xml = document_xml(&bytes);
    let secs: Vec<&str> = xml.split("<w:sectPr").skip(1).map(|x| x.split("</w:sectPr>").next().unwrap_or("")).collect();
    assert_eq!(secs.len(), 2, "{xml}");
    assert!(secs[0].contains("<w:footerReference"), "{}", secs[0]);
    assert!(!secs[1].contains("<w:footerReference"), "linked: no reference of its own: {}", secs[1]);
    let mut back = Session::new(io::open_bytes("linked.docx", &bytes).unwrap());
    assert_eq!(footer_numbers(&mut back), ["i", "ii", "3", "4", "5"]);
}
