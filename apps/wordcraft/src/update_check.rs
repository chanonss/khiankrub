//! The update check's request: GitHub's latest release of KhianKrub, asked on a background
//! thread so the window never waits ([`wordcraft_ui_egui::updates`] decides what it means).

use std::io::Read;
use std::sync::mpsc;
use std::time::Duration;

/// GitHub's API for the latest published release (drafts and pre-releases aren't listed).
const LATEST: &str = "https://api.github.com/repos/chanonss/khiankrub/releases/latest";

/// The most of the answer read: a release's JSON is a few kilobytes.
const MAX_BYTES: u64 = 1 << 20;

/// Ask now; the answer arrives on the receiver.
pub fn ask() -> mpsc::Receiver<wordcraft_ui_egui::updates::Answer> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch());
    });
    rx
}

fn fetch() -> wordcraft_ui_egui::updates::Answer {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build();
    let resp = agent
        .get(LATEST)
        .set("User-Agent", concat!("KhianKrub/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("GitHub answered {code}"),
            ureq::Error::Transport(t) => t.to_string(),
        })?;
    let mut body = String::new();
    resp.into_reader().take(MAX_BYTES).read_to_string(&mut body).map_err(|e| e.to_string())?;
    serde_json::from_str(&body).map_err(|e| e.to_string())
}
