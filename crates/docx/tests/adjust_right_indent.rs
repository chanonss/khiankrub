//! `w:adjustRightInd` on a paragraph and in its style: kept as read and written back (from WordCraft #5384).

use std::io::{Read, Write};

fn docx(body: &str, styles: &str) -> Vec<u8> {
    let root = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let document = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    let styles = format!(r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{styles}</w:styles>"#);
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, xml) in [
        ("_rels/.rels", root.to_string()),
        ("word/document.xml", document),
        ("word/_rels/document.xml.rels", rels.to_string()),
        ("word/styles.xml", styles),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

#[test]
fn paragraph_adjust_right_indent_inherits_from_style() {
    let styles =
        r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:adjustRightInd/></w:pPr></w:style>"#;
    let body = r#"<w:p><w:pPr><w:adjustRightInd w:val="0"/></w:pPr><w:r><w:t>off</w:t></w:r></w:p>
        <w:p><w:r><w:t>inherited</w:t></w:r></w:p>"#;
    let mut d = wordcraft_docx::read(&docx(body, styles)).unwrap();
    let paragraphs: Vec<_> = d.body.iter().filter_map(|b| b.as_para()).collect();
    assert_eq!(paragraphs[0].props.adjust_right_indent, Some(false));
    assert!(!d.styles.resolve_para(&paragraphs[0].props).adjust_right_indent);
    assert!(d.styles.resolve_para(&paragraphs[1].props).adjust_right_indent);

    for _ in 0..2 {
        let out = wordcraft_docx::write(&d).unwrap();
        assert!(part(&out, "word/document.xml").contains(r#"<w:adjustRightInd w:val="0""#));
        assert!(part(&out, "word/styles.xml").contains("<w:adjustRightInd/>"));
        d = wordcraft_docx::read(&out).unwrap();
        let paragraphs: Vec<_> = d.body.iter().filter_map(|b| b.as_para()).collect();
        assert!(!d.styles.resolve_para(&paragraphs[0].props).adjust_right_indent);
        assert!(d.styles.resolve_para(&paragraphs[1].props).adjust_right_indent);
        assert_eq!(paragraphs[0].props.adjust_right_indent, Some(false));
    }
}
