//! The system clipboard beyond egui's text (#45, #584, #687, #833): text for the ribbon's Paste,
//! another program's formatted copy (HTML, and RTF where the platform offers it), pictures and
//! copied files, and the names of the formats there for a paste that finds nothing it can use.
//!
//! On Linux, arboard talks to Wayland compositors through the data-control protocol (KDE Plasma,
//! wlroots compositors); without it, a native Wayland window only sees what the compositor copies
//! over to Xwayland, which KWin does only while an X11 window has focus — so pictures copied in a
//! browser or Spectacle never arrived (#833). Compositors without the protocol fall back to X11.
//!
//! Everything read here comes from other programs: sizes are capped as it is read.
//!
//! Written here: a copy puts HTML beside its text, so other programs keep its formatting (#1523),
//! with equations as MathML (#816; pasting into Word and LibreOffice not yet verified). arboard sets HTML with a
//! text alternative on every desktop platform; formats of other names (Word's "Office Open XML"
//! with OMML, #1297) it can't, so they aren't offered.

use wordcraft_engine::cmd::paste::MAX_PASTE_BYTES;
use wordcraft_ui_egui::paste_picture::ClipboardPicture;

fn open() -> Option<arboard::Clipboard> {
    arboard::Clipboard::new().map_err(|e| log::warn!("clipboard: {e}")).ok()
}

thread_local! {
    /// The clipboard copies are set through, kept open: on X11 the contents are served from it.
    static WRITER: std::cell::RefCell<Option<arboard::Clipboard>> = const { std::cell::RefCell::new(None) };
}

/// Put `html` on the clipboard with `text` as its plain-text alternative. `false` when it couldn't
/// (the caller copies the text alone), including a Wayland session without the data-control
/// protocol, where arboard would fall back to X11 and native Wayland programs might not see it.
pub fn set_html(html: &str, text: &str) -> bool {
    if !platform::can_set() {
        return false;
    }
    WRITER.with(|w| {
        let Ok(mut slot) = w.try_borrow_mut() else { return false };
        if slot.is_none() {
            *slot = open();
        }
        match slot.as_mut().map(|c| c.set().html(html, Some(text))) {
            Some(Ok(())) => true,
            Some(Err(e)) => {
                log::warn!("clipboard: {e}");
                *slot = None;
                false
            }
            None => false,
        }
    })
}

/// The clipboard's text.
pub fn text() -> Option<String> {
    open()?.get_text().ok()
}

/// The clipboard's HTML (Windows: the fragment of its "HTML Format"), if not too large.
pub fn html() -> Option<String> {
    open()?.get().html().ok().filter(|h| h.len() <= MAX_PASTE_BYTES)
}

/// The clipboard's RTF: Windows ("Rich Text Format") and Wayland (`text/rtf`); not offered by
/// arboard, so read here.
pub fn rtf() -> Option<String> {
    let bytes = platform::rtf()?;
    if bytes.len() > MAX_PASTE_BYTES {
        return None;
    }
    // RTF is 7-bit by the spec; stray 8-bit bytes become replacement characters.
    let s = String::from_utf8_lossy(&bytes).trim_end_matches('\0').to_string();
    s.trim_start().starts_with("{\\rtf").then_some(s)
}

/// The names of the formats on the clipboard, where the platform lists them (Wayland, Windows).
pub fn formats() -> Vec<String> {
    platform::formats()
}

/// The picture on the clipboard: copied files, else image pixels, else (Wayland) an encoded
/// picture in a format arboard doesn't decode. A picture that is there but can't be read says
/// why (#833). The UI checks the sizes before inserting anything.
pub fn picture() -> Option<ClipboardPicture> {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("clipboard: {e}");
            return Some(ClipboardPicture::Unreadable(format!("the system clipboard couldn't be opened ({e})")));
        }
    };
    if let Ok(files) = clipboard.get().file_list()
        && !files.is_empty()
    {
        return Some(ClipboardPicture::Files(files));
    }
    match clipboard.get_image() {
        Ok(image) => Some(ClipboardPicture::Pixels { width: image.width, height: image.height, rgba: image.bytes.into_owned() }),
        Err(arboard::Error::ContentNotAvailable) => platform::encoded_picture().map(ClipboardPicture::Encoded),
        Err(arboard::Error::ConversionFailure) => Some(
            platform::encoded_picture()
                .map(ClipboardPicture::Encoded)
                .unwrap_or_else(|| ClipboardPicture::Unreadable("KhianKrub can't decode its format".into())),
        ),
        Err(e) => {
            log::warn!("clipboard picture: {e}");
            platform::encoded_picture().map(ClipboardPicture::Encoded).or(Some(ClipboardPicture::Unreadable(e.to_string())))
        }
    }
}

/// Picture types read straight from Wayland when arboard (PNG only) finds none, best first.
#[cfg(all(unix, not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))))]
const PICTURE_MIMES: [&str; 6] = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/bmp", "image/x-bmp"];

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))))]
mod platform {
    use std::io::Read;

    use wl_clipboard_rs::paste::{ClipboardType, MimeType, Seat, get_contents, get_mime_types_ordered};

    fn wayland() -> bool {
        std::env::var_os("WAYLAND_DISPLAY").is_some()
    }

    /// One MIME type's bytes, at most `cap`; `None` when absent or larger.
    fn read(mime: &str, cap: u64) -> Option<Vec<u8>> {
        let (pipe, _) = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Specific(mime)).ok()?;
        let mut out = Vec::new();
        pipe.take(cap.saturating_add(1)).read_to_end(&mut out).ok()?;
        (out.len() as u64 <= cap).then_some(out)
    }

    pub fn formats() -> Vec<String> {
        if !wayland() {
            return Vec::new();
        }
        get_mime_types_ordered(ClipboardType::Regular, Seat::Unspecified).unwrap_or_default()
    }

    /// Whether arboard sets the clipboard natively: X11, or Wayland with data-control.
    pub fn can_set() -> bool {
        !wayland()
            || !matches!(
                get_mime_types_ordered(ClipboardType::Regular, Seat::Unspecified),
                Err(wl_clipboard_rs::paste::Error::MissingProtocol { .. } | wl_clipboard_rs::paste::Error::WaylandConnection(_))
            )
    }

    pub fn rtf() -> Option<Vec<u8>> {
        if !wayland() {
            return None;
        }
        let offered = formats();
        let mime = ["text/rtf", "application/rtf", "text/richtext"].into_iter().find(|m| offered.iter().any(|o| o == m))?;
        read(mime, super::MAX_PASTE_BYTES as u64)
    }

    pub fn encoded_picture() -> Option<Vec<u8>> {
        if !wayland() {
            return None;
        }
        let offered = formats();
        let mime = super::PICTURE_MIMES.into_iter().find(|m| offered.iter().any(|o| o == m))?;
        read(mime, wordcraft_ui_egui::paste_picture::MAX_FILE_BYTES)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use clipboard_win::{Clipboard, EnumFormats, raw};

    pub fn formats() -> Vec<String> {
        let Ok(_open) = Clipboard::new_attempts(10) else { return Vec::new() };
        // Standard formats are named too (CF_BITMAP, CF_UNICODETEXT…).
        EnumFormats::new().take(64).filter_map(raw::format_name_big).collect()
    }

    pub fn rtf() -> Option<Vec<u8>> {
        let format = raw::register_format("Rich Text Format")?;
        let _open = Clipboard::new_attempts(10).ok()?;
        if raw::size(format.get()).is_none_or(|n| n.get() > super::MAX_PASTE_BYTES) {
            return None;
        }
        let mut out = Vec::new();
        raw::get_vec(format.get(), &mut out).ok()?;
        Some(out)
    }

    pub fn encoded_picture() -> Option<Vec<u8>> {
        None
    }

    pub fn can_set() -> bool {
        true
    }
}

#[cfg(not(any(target_os = "windows", all(unix, not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))))))]
mod platform {
    pub fn formats() -> Vec<String> {
        Vec::new()
    }

    pub fn rtf() -> Option<Vec<u8>> {
        None
    }

    pub fn encoded_picture() -> Option<Vec<u8>> {
        None
    }

    pub fn can_set() -> bool {
        true
    }
}
