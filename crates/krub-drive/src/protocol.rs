//! Google Drive request construction and checks. No credentials or network access here.

use crate::FileKind;

/// Multipart uploads are intended for small files; larger documents need resumable upload.
const MAX_MULTIPART_BYTES: usize = 5 * 1024 * 1024;

/// Build a multipart/related body for a new file of `kind` in Drive.
/// The boundary must not occur in the document or metadata.
pub fn multipart_upload(kind: FileKind, name: &str, bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
    if name.is_empty() || name.len() > 1024 || name.chars().any(char::is_control) {
        return Err("Invalid document name".into());
    }
    check_upload_size(bytes)?;
    let metadata = serde_json::to_vec(&serde_json::json!({"name": name, "mimeType": kind.mime})).map_err(|e| e.to_string())?;
    let boundary = (0..1000)
        .map(|n| format!("krub_drive_upload_{n:03}"))
        .find(|b| !contains(bytes, b.as_bytes()) && !contains(&metadata, b.as_bytes()))
        .ok_or("Could not prepare upload")?;
    let mut body = Vec::with_capacity(bytes.len().saturating_add(metadata.len()).saturating_add(512));
    body.extend_from_slice(format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n").as_bytes());
    body.extend_from_slice(&metadata);
    body.extend_from_slice(format!("\r\n--{boundary}\r\nContent-Type: {}\r\n\r\n", kind.mime).as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Ok((boundary, body))
}

/// Single-request uploads, new copies and replacements alike, stay within the multipart limit.
pub fn check_upload_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_MULTIPART_BYTES {
        return Err("The document is too large for this upload".into());
    }
    Ok(())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|part| part == needle)
}

/// Google Drive IDs are opaque, but cannot contain path or query delimiters.
pub fn valid_file_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 1024 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

/// A Drive file name the app can open as `kind`: printable, bounded, with the kind's extension.
pub fn valid_file_name(kind: FileKind, name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && !name.chars().any(char::is_control)
        && name.rsplit_once('.').is_some_and(|(stem, ext)| !stem.is_empty() && ext.eq_ignore_ascii_case(kind.extension))
}

/// A local OAuth client must never be selected for a public website by accident.
pub fn oauth_configuration<'a>(
    host: &str,
    client: Option<&'a str>,
    project: Option<&'a str>,
    local_client: &'a str,
    local_project: &'a str,
) -> Option<(&'a str, &'a str)> {
    let (client, project) = match (client, project) {
        (Some(client), Some(project)) if !client.is_empty() && !project.is_empty() => (client, project),
        (None, None) if host == "localhost" || host == "127.0.0.1" => (local_client, local_project),
        _ => return None,
    };
    (client.ends_with(".apps.googleusercontent.com") && !project.is_empty() && project.bytes().all(|b| b.is_ascii_digit()))
        .then_some((client, project))
}

/// The Drive connection service, when the build asks for exactly the one this app may use.
/// A deployment setting must never send session credentials to an arbitrary endpoint.
pub fn backend(requested: Option<&'static str>, allowed: &'static str) -> Option<&'static str> {
    let base = requested?.trim_end_matches('/');
    (base == allowed && allowed.starts_with("https://")).then_some(allowed)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::{DOCX, PPTX, XLSX};

    #[test]
    fn oauth_configuration_requires_public_build_configuration() {
        let local = "local.apps.googleusercontent.com";
        let prod = "production.apps.googleusercontent.com";
        assert_eq!(oauth_configuration("localhost", None, None, local, "123"), Some((local, "123")));
        assert_eq!(oauth_configuration("example.com", None, None, local, "123"), None);
        assert_eq!(oauth_configuration("example.com", Some(prod), Some("456"), local, "123"), Some((prod, "456")));
        assert_eq!(oauth_configuration("localhost", Some(prod), None, local, "123"), None);
        assert_eq!(oauth_configuration("example.com", Some(prod), Some("invalid"), local, "123"), None);
    }

    #[test]
    fn upload_preserves_binary_and_escapes_the_name() {
        let bytes = b"PK\0krub_drive_upload_000\xff";
        let (boundary, body) = multipart_upload(DOCX, "Thai \"เอกสาร\".docx", bytes).unwrap();
        assert_eq!(boundary, "krub_drive_upload_001");
        assert!(contains(&body, bytes));
        assert!(contains(&body, "Thai \\\"เอกสาร\\\".docx".as_bytes()));
        assert!(body.ends_with(format!("\r\n--{boundary}--\r\n").as_bytes()));
    }

    #[test]
    fn upload_names_the_kind_of_file() {
        for kind in [DOCX, PPTX, XLSX] {
            let (_, body) = multipart_upload(kind, &format!("a.{}", kind.extension), b"PK").unwrap();
            let text = String::from_utf8_lossy(&body);
            assert_eq!(text.matches(kind.mime).count(), 2, "metadata and part header: {}", kind.mime);
        }
    }

    #[test]
    fn upload_rejects_invalid_names_and_large_files() {
        assert!(multipart_upload(DOCX, "bad\nname.docx", b"doc").is_err());
        assert!(multipart_upload(DOCX, "", b"doc").is_err());
        assert!(multipart_upload(XLSX, "file.xlsx", &vec![0; MAX_MULTIPART_BYTES + 1]).is_err());
        assert!(multipart_upload(XLSX, "file.xlsx", &vec![0; MAX_MULTIPART_BYTES]).is_ok());
    }

    #[test]
    fn drive_file_id_cannot_change_the_request_url() {
        assert!(valid_file_id("1abc_DEF-234"));
        for id in ["", "../files", "a?alt=media", "a/b", "a#b", "a%2fb"] {
            assert!(!valid_file_id(id));
        }
        assert!(!valid_file_id(&"a".repeat(1025)));
    }

    #[test]
    fn file_names_must_have_the_apps_extension() {
        assert!(valid_file_name(PPTX, "แผน.pptx"));
        assert!(valid_file_name(PPTX, "Deck.PPTX"));
        assert!(valid_file_name(XLSX, "budget.v2.xlsx"));
        for name in ["deck.docx", "deck", ".pptx", "pptx", "a\u{7}.pptx", ""] {
            assert!(!valid_file_name(PPTX, name), "{name:?}");
        }
        assert!(!valid_file_name(XLSX, &format!("{}.xlsx", "a".repeat(1020))));
    }

    #[test]
    fn only_the_apps_own_connection_service_is_used() {
        let ours = "https://drive.phutkrub.app";
        assert_eq!(backend(Some("https://drive.phutkrub.app/"), ours), Some(ours));
        assert_eq!(backend(Some("https://drive.khiankrub.app"), ours), None);
        assert_eq!(backend(Some("https://evil.example"), ours), None);
        assert_eq!(backend(None, ours), None);
        assert_eq!(backend(Some("http://drive.test"), "http://drive.test"), None);
    }

    #[test]
    fn every_status_and_message_is_distinct() {
        let mut all: Vec<&str> = crate::STATUS.iter().chain(crate::MESSAGES).copied().collect();
        let n = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), n);
    }
}
