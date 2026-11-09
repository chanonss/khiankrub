//! Families a host can load on request: the web's bundled fonts, fetched only when a document,
//! the font box or its list asks for them, so the first start downloads just the fonts it needs.
//! An announced family is listed and counts as installed; until it loads, lookups get the fallback
//! face and the family is queued for the host ([`FontDb::take_requested_families`]). Loading it
//! adds its faces, which bumps [`FontDb::generation`], and layout runs again in the real face.

use super::FontDb;

/// Most families a host may announce, and the longest name kept: the list is the host's own, but
/// names reach the font box, so both stay bounded.
const MAX_FAMILIES: usize = 512;
const MAX_NAME: usize = 200;

#[derive(Default)]
pub(super) struct OnRequest {
    /// Announced families, as the host spells them.
    families: Vec<String>,
    /// Families asked for so far (each is queued once).
    asked: Vec<String>,
    /// Asked for and not yet handed to the host.
    queue: Vec<String>,
    /// Asked for and the host failed to load them (until a retry is allowed).
    failed: Vec<String>,
}

impl FontDb {
    fn read_on_request(&self) -> std::sync::RwLockReadGuard<'_, OnRequest> {
        self.on_request.read().unwrap_or_else(|e| e.into_inner())
    }

    fn write_on_request(&self) -> std::sync::RwLockWriteGuard<'_, OnRequest> {
        self.on_request.write().unwrap_or_else(|e| e.into_inner())
    }

    /// Offer `families` for loading on request. Families already loaded are left out; the list
    /// replaces any announced before.
    pub fn announce_families(&self, families: impl IntoIterator<Item = String>) {
        let mut list: Vec<String> =
            families.into_iter().filter(|f| !f.trim().is_empty() && f.len() <= MAX_NAME && !self.is_loaded(f)).take(MAX_FAMILIES).collect();
        list.sort_by_key(|a| a.to_lowercase());
        list.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        self.write_on_request().families = list;
    }

    /// The announced families, as announced (loaded ones included).
    pub(super) fn announced_families(&self) -> Vec<String> {
        self.read_on_request().families.clone()
    }

    /// Was `family` announced? (Loaded or not.)
    pub(super) fn is_announced(&self, family: &str) -> bool {
        self.read_on_request().families.iter().any(|f| super::same_family(f, family))
    }

    /// Is `family` announced but not loaded yet? Its text draws in the fallback face for now.
    pub fn is_pending(&self, family: &str) -> bool {
        self.is_announced(family) && !self.is_loaded(family)
    }

    /// Ask for `family` if it is announced and not loaded: the host loads it soon. Asking again
    /// does nothing, so a family whose load failed isn't fetched over and over.
    pub fn request_family(&self, family: &str) {
        if !self.is_pending(family) {
            return;
        }
        {
            let r = self.read_on_request();
            if r.asked.iter().any(|f| super::same_family(f, family)) {
                return;
            }
        }
        let mut r = self.write_on_request();
        let Some(name) = r.families.iter().find(|f| super::same_family(f, family)).cloned() else { return };
        if !r.asked.iter().any(|f| f.eq_ignore_ascii_case(&name)) {
            r.asked.push(name.clone());
            r.queue.push(name);
        }
    }

    /// The families asked for since the last call, for the host to load.
    pub fn take_requested_families(&self) -> Vec<String> {
        {
            if self.read_on_request().queue.is_empty() {
                return Vec::new();
            }
        }
        std::mem::take(&mut self.write_on_request().queue)
    }

    /// The host couldn't load `family`: it no longer counts as loading (an export can go ahead in
    /// the fallback face) until [`FontDb::request_failed`] lets a later request try again.
    pub fn mark_failed(&self, family: &str) {
        let mut r = self.write_on_request();
        if !r.failed.iter().any(|f| f.eq_ignore_ascii_case(family)) {
            r.failed.push(family.to_string());
        }
    }

    /// The host may try `family` again: a later request asks for it afresh.
    pub fn request_failed(&self, family: &str) {
        let mut r = self.write_on_request();
        r.asked.retain(|f| !f.eq_ignore_ascii_case(family));
        r.failed.retain(|f| !f.eq_ignore_ascii_case(family));
    }

    /// Families asked for whose files are still on their way: announced, not loaded, not failed. A
    /// PDF or print made now would draw their text in the fallback face.
    pub fn loading_families(&self) -> Vec<String> {
        let r = self.read_on_request();
        r.asked.iter().filter(|f| !r.failed.iter().any(|x| x.eq_ignore_ascii_case(f)) && self.is_pending(f)).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn announced_families_are_listed_requested_once_and_bounded() {
        let db = FontDb::with_font_dirs(vec![]);
        let fallback = db.face("Nope", "Regular").family.clone();
        db.announce_families(["TH Niramit AS".to_string(), "th niramit as".into(), String::new(), "x".repeat(MAX_NAME + 1)]);
        assert_eq!(db.announced_families(), ["TH Niramit AS"]);
        assert!(db.families().iter().any(|f| f == "TH Niramit AS"), "listed before it loads");
        assert!(db.has_family("th niramit as"), "counts as installed, so no substitute replaces it");
        assert!(db.is_pending("TH Niramit AS"));
        // A lookup gets the fallback face for now and asks for the family, once.
        assert_eq!(db.face("TH Niramit AS", "Bold").family, fallback);
        db.face("TH Niramit AS", "Regular");
        assert!(db.preview_face("TH Niramit AS").is_none());
        assert_eq!(db.take_requested_families(), ["TH Niramit AS"]);
        assert!(db.take_requested_families().is_empty());
        db.request_family("TH Niramit AS");
        assert!(db.take_requested_families().is_empty(), "asked once");
        db.request_failed("TH Niramit AS");
        db.request_family("TH Niramit AS");
        assert_eq!(db.take_requested_families(), ["TH Niramit AS"], "a failed load may be asked again");
        // Unknown and loaded families are never queued.
        db.request_family("Unknown Family");
        db.request_family(&fallback);
        assert!(db.take_requested_families().is_empty());
        let many: Vec<String> = (0..MAX_FAMILIES + 50).map(|i| format!("Family {i}")).collect();
        db.announce_families(many);
        assert_eq!(db.announced_families().len(), MAX_FAMILIES);
    }

    #[test]
    fn a_family_is_loading_from_its_request_until_it_loads_or_fails() {
        let db = FontDb::with_font_dirs(vec![]);
        db.announce_families(["TH Chakra Petch".to_string()]);
        assert!(db.loading_families().is_empty(), "announced but never asked for: nothing is on its way");
        db.face("TH Chakra Petch", "Regular");
        assert_eq!(db.loading_families(), ["TH Chakra Petch"], "asked for: on its way");
        // The host gave up: an export may go ahead in the fallback face.
        db.mark_failed("th chakra petch");
        assert!(db.loading_families().is_empty());
        db.mark_failed("TH Chakra Petch");
        assert!(db.loading_families().is_empty(), "failing twice changes nothing");
        // A retry asks again and the family is on its way again.
        db.request_failed("TH Chakra Petch");
        db.face("TH Chakra Petch", "Regular");
        assert_eq!(db.loading_families(), ["TH Chakra Petch"]);
    }
}
