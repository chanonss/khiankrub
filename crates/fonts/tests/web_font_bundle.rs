//! Check the external browser font bundle without access to installed system fonts.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use wordcraft_fonts::FontDb;

#[test]
fn web_document_fonts_use_the_exact_original_faces() {
    let Some(dir) = std::env::var_os("KHIANKRUB_WEB_FONTS_DIR").map(PathBuf::from) else {
        eprintln!("external font bundle not supplied; set KHIANKRUB_WEB_FONTS_DIR to validate it");
        return;
    };
    let db = FontDb::with_font_dirs(vec![]);
    assert!(!db.has_family("TH Sarabun New"));
    assert!(!db.has_family("TH SarabunPSK"));
    let mut originals = Vec::new();
    for (family, prefix) in [("TH Sarabun New", "THSarabunNew"), ("TH SarabunPSK", "THSarabunPSK")] {
        for (style, suffix) in [("Regular", "Regular"), ("Bold", "Bold"), ("Italic", "Italic"), ("Bold Italic", "BoldItalic")] {
            let bytes = std::fs::read(dir.join(format!("{prefix}-{suffix}.ttf"))).unwrap();
            originals.push((family, style, bytes));
        }
    }
    assert_eq!(db.add_document_fonts(originals.iter().map(|(_, _, bytes)| bytes.clone()).collect()), 8);
    for (family, style, bytes) in originals {
        let face = db.face(family, style);
        assert!(face.named(family), "{family} {style} was substituted");
        assert_eq!(face.data(), bytes, "{family} {style} used the wrong face");
        for c in ['ก', 'น', '่', '้', 'A', 'z', '0', '9'] {
            assert!(face.covers(c), "{family} {style} lacks {c}");
        }
    }
}
