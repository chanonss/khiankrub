//! Report a problem (แจ้งปัญหา): a button beside Share opens Chanon's Google Form in a new tab, with
//! the app version and the browser filled in. No document content is sent; the form is answered on
//! Google's page. The button shows only while a form is configured here.

/// The Google Form's prefill address, without its query (`…/viewform`); empty hides the button.
/// Chanon's form "แจ้งปัญหา เขียนครับ (KhianKrub)" (personal Google account csarasuk@gmail.com).
pub const FORM_URL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSddU9FebLOY5spPMQFfnGadCSkn_Rw6jJBdUhxdwWWsGHjEWA/viewform";
/// The form's question ids (`entry.<number>`) for the two answers the app fills in.
pub const VERSION_ENTRY: &str = "entry.368913138";
pub const BROWSER_ENTRY: &str = "entry.1229712171";

/// The form's address with the version and browser filled in, or `None` when no form is configured.
pub fn report_url(version: &str, browser: &str) -> Option<String> {
    report_url_for(FORM_URL, VERSION_ENTRY, BROWSER_ENTRY, version, browser)
}

fn report_url_for(form: &str, version_entry: &str, browser_entry: &str, version: &str, browser: &str) -> Option<String> {
    if form.is_empty() {
        return None;
    }
    let mut url = format!("{form}?usp=pp_url");
    for (entry, value) in [(version_entry, version), (browser_entry, browser)] {
        if !entry.is_empty() && !value.is_empty() {
            // Long user-agent strings are cut, at a character boundary, to keep the address short.
            let value: String = value.chars().take(300).collect();
            url.push_str(&format!("&{entry}={}", encode(&value)));
        }
    }
    Some(url)
}

/// Percent-encode a query value (RFC 3986 unreserved characters are kept).
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_configured_form_opens_with_the_version() {
        let url = report_url("0.9.1", "Chrome").unwrap();
        assert!(url.starts_with("https://docs.google.com/forms/d/e/") && url.contains("&entry.368913138=0.9.1&entry.1229712171=Chrome"), "{url}");
    }

    #[test]
    fn no_form_means_no_button() {
        assert_eq!(report_url_for("", "entry.1", "entry.2", "0.9.1", "Chrome"), None);
    }

    #[test]
    fn the_version_and_browser_are_filled_in_and_encoded() {
        let url =
            report_url_for("https://docs.google.com/forms/d/e/X/viewform", "entry.11", "entry.22", "0.9.1", "Mozilla/5.0 (Mac) ไทย&x=1").unwrap();
        assert_eq!(
            url,
            "https://docs.google.com/forms/d/e/X/viewform?usp=pp_url&entry.11=0.9.1&entry.22=Mozilla%2F5.0%20%28Mac%29%20%E0%B9%84%E0%B8%97%E0%B8%A2%26x%3D1"
        );
    }

    #[test]
    fn a_long_browser_string_is_cut() {
        let ua = "a".repeat(1000);
        let url = report_url_for("https://f/viewform", "entry.1", "entry.2", "1", &ua).unwrap();
        assert!(url.len() < 400, "{}", url.len());
    }
}
