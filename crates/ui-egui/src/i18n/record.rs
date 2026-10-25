//! Test helper: records the English strings the interface looks up while it draws, so a test can
//! check that what is on screen has a translation.

use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    static SEEN: RefCell<Option<BTreeSet<(String, String)>>> = const { RefCell::new(None) };
}

/// Start recording on this thread (clears what was recorded before).
pub(crate) fn start() {
    SEEN.with(|s| *s.borrow_mut() = Some(BTreeSet::new()));
}

/// Stop recording and return each `(context, English)` looked up since [`start`].
pub(crate) fn stop() -> BTreeSet<(String, String)> {
    SEEN.with(|s| s.borrow_mut().take()).unwrap_or_default()
}

pub(super) fn note(context: &str, s: &str) {
    SEEN.with(|seen| {
        if let Some(seen) = seen.borrow_mut().as_mut() {
            seen.insert((context.to_string(), s.to_string()));
        }
    });
}
