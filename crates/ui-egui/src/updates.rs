//! Update check: is a newer KhianKrub released? The desktop host asks GitHub for the latest
//! release ([`crate::Services::latest_release`]); this module decides when to ask (at most once a
//! day at start-up, or Help › Check for Updates), compares versions, and shows the bar that offers
//! the download. Nothing is downloaded or installed here: Download opens the release page.

use egui::{Color32, Ui};
use serde_json::{Value, json};

use crate::WordApp;
use crate::theme::Tokens;

/// A newer release: its version (`0.8.1`) and its page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub url: String,
}

/// An answer the host gives: GitHub's latest release (its JSON), or why it couldn't ask.
pub type Answer = Result<Value, String>;

/// The check in progress and what it found.
#[derive(Default)]
pub struct UpdateState {
    /// The host's answer to come.
    pub pending: Option<std::sync::mpsc::Receiver<Answer>>,
    /// Asked from Help › Check for Updates: report "up to date" and errors too.
    pub manual: bool,
    /// A newer release, while its bar shows.
    pub found: Option<Release>,
}

/// The release in GitHub's answer, if it is a published, final one: `tag_name` `v0.8.1` and
/// `html_url`. Drafts, pre-releases and test tags (`v0.8.1-test1`) are left out.
pub fn release(v: &Value) -> Option<Release> {
    if v.get("draft").and_then(Value::as_bool) == Some(true) || v.get("prerelease").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let tag = v.get("tag_name")?.as_str()?;
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if version.contains('-') || parts(version).is_none() {
        return None;
    }
    let url = v.get("html_url")?.as_str()?;
    url.starts_with("https://github.com/").then(|| Release { version: version.to_string(), url: url.to_string() })
}

/// `1.2.3` as numbers; `None` for anything else.
fn parts(v: &str) -> Option<Vec<u64>> {
    let p: Option<Vec<u64>> = v.split('.').map(|n| n.parse().ok()).collect();
    p.filter(|p| !p.is_empty() && p.len() <= 4)
}

/// Is `latest` a later version than `current`? (`0.8.10` > `0.8.9`; `0.9` > `0.8.3`.)
pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parts(latest), parts(current.split('-').next().unwrap_or(current))) {
        (Some(a), Some(b)) => {
            let n = a.len().max(b.len());
            let pad = |v: Vec<u64>| v.into_iter().chain(std::iter::repeat(0)).take(n).collect::<Vec<_>>();
            pad(a) > pad(b)
        }
        _ => false,
    }
}

/// This build's version.
pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Today's local date, `2026-10-25`.
fn today() -> String {
    wordcraft_engine::cmd::now_local_iso().get(0..10).unwrap_or("").to_string()
}

/// Ask the host now (`manual`: from Help › Check for Updates). False when this build can't ask
/// (no host hook: the web version and tests).
pub fn start(app: &mut WordApp, manual: bool) -> bool {
    let Some(ask) = &app.services.latest_release else { return false };
    app.updates.pending = Some(ask());
    app.updates.manual = manual;
    app.ui.update_checked_on = today();
    true
}

/// Every frame: ask at start-up when it is on and not yet asked today, and take the answer when
/// it comes.
pub fn tick(app: &mut WordApp) {
    if !app.update_started {
        app.update_started = true;
        if app.ui.check_updates && app.ui.update_checked_on != today() {
            start(app, false);
        }
    }
    let Some(rx) = &app.updates.pending else { return };
    let Ok(answer) = rx.try_recv() else { return };
    app.updates.pending = None;
    let manual = std::mem::take(&mut app.updates.manual);
    take_answer(app, answer, manual);
}

/// What an answer means: a newer release shows the bar (unless it was skipped and this check
/// wasn't asked for); otherwise a manual check says the version is the latest, or why it failed.
pub fn take_answer(app: &mut WordApp, answer: Answer, manual: bool) {
    match answer.map(|v| release(&v)) {
        Ok(Some(r)) if is_newer(&r.version, current()) && (manual || app.ui.update_skipped != r.version) => app.updates.found = Some(r),
        Ok(_) if manual => {
            let msg = crate::i18n::fmt(tl!("KhianKrub {version} is the latest version."), &[("version", current())]);
            app.status(msg);
        }
        Err(e) if manual => {
            let msg = crate::i18n::fmt(tl!("Couldn't check for updates: {error}"), &[("error", &e)]);
            app.status(msg);
        }
        _ => {}
    }
}

/// `ui.checkForUpdates` and the bar's buttons (`ui.update`): `{"action": "check"}` asks now,
/// `"download"` opens the release page, `"skip"` hides this version for good, `"close"` hides the
/// bar until the next start. No action reads the state.
pub fn command(app: &mut WordApp, p: &Value) -> Result<Value, String> {
    match p.get("action").and_then(Value::as_str) {
        None => {}
        Some("check") => {
            if !start(app, true) {
                return Err("this version of KhianKrub can't check for updates".into());
            }
            app.status(tl!("Checking for updates…").to_string());
        }
        Some("download") => {
            if let Some(r) = app.updates.found.take() {
                app.canvas.open_url = Some(r.url);
            }
        }
        Some("skip") => {
            if let Some(r) = app.updates.found.take() {
                app.ui.update_skipped = r.version;
            }
        }
        Some("close") => app.updates.found = None,
        Some(other) => return Err(format!("unknown action `{other}`; use check, download, skip or close")),
    }
    Ok(json!({
        "current": current(),
        "available": app.updates.found.as_ref().map(|r| json!({"version": r.version, "url": r.url})),
        "checking": app.updates.pending.is_some(),
        "skipped": app.ui.update_skipped,
    }))
}

/// The bar under the ribbon while a newer release is found: the news, Download, Skip this
/// version, and close.
pub fn bar(app: &mut WordApp, ui: &mut Ui) {
    let Some(r) = app.updates.found.clone() else { return };
    let t = Tokens::of(ui);
    let fill = if t.dark { Color32::from_rgb(0x4D, 0x42, 0x1C) } else { Color32::from_rgb(0xFD, 0xF3, 0xD2) };
    egui::Panel::top("update_bar")
        .frame(egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::symmetric(12, 6)).stroke(egui::Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::icons::paint(
                    ui.painter(),
                    egui::Rect::from_center_size(ui.cursor().min + egui::vec2(10.0, 10.0), egui::vec2(18.0, 18.0)),
                    "info",
                    t.icon,
                    t.accent,
                );
                ui.add_space(24.0);
                let news = crate::i18n::fmt(tl!("KhianKrub {version} is available."), &[("version", &r.version)]);
                ui.label(egui::RichText::new(news).strong());
                if ui.button(tl!("Download")).clicked() {
                    let _ = app.run("ui.update", json!({"action": "download"}));
                }
                if ui.button(tl!("Skip this version")).clicked() {
                    let _ = app.run("ui.update", json!({"action": "skip"}));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::widgets::icon_button(ui, "close", tl!("Close")).clicked() {
                        let _ = app.run("ui.update", json!({"action": "close"}));
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert!(is_newer("0.8.1", "0.8.0"));
        assert!(is_newer("0.8.10", "0.8.9"));
        assert!(is_newer("0.9", "0.8.3"));
        assert!(is_newer("1.0.0", "0.8.0"));
        assert!(!is_newer("0.8.0", "0.8.0"));
        assert!(!is_newer("0.7.9", "0.8.0"));
        assert!(!is_newer("junk", "0.8.0"));
        assert!(is_newer("0.8.1", "0.8.0-dev"), "a development build compares by its number");
    }

    #[test]
    fn only_published_final_releases_count() {
        let rel = |tag: &str, draft: bool, pre: bool, url: &str| json!({"tag_name": tag, "draft": draft, "prerelease": pre, "html_url": url});
        let page = "https://github.com/chanonss/khiankrub/releases/tag/v0.8.1";
        assert_eq!(release(&rel("v0.8.1", false, false, page)), Some(Release { version: "0.8.1".into(), url: page.into() }));
        assert_eq!(release(&rel("v0.8.1", true, false, page)), None, "draft");
        assert_eq!(release(&rel("v0.8.1", false, true, page)), None, "pre-release");
        assert_eq!(release(&rel("v0.8.1-test1", false, false, page)), None, "test tag");
        assert_eq!(release(&rel("v0.8.1", false, false, "https://example.com/x")), None, "only GitHub pages open");
        assert_eq!(release(&json!({})), None);
    }
}
