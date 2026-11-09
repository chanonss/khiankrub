//! The main window's size, position and maximized state, saved with the UI preferences so the
//! next launch opens the window where the user left it (#23).
//!
//! Sizes and positions are in egui points: what [`egui::ViewportInfo`] reports and what
//! [`egui::ViewportBuilder`] takes.

use serde::{Deserialize, Serialize};

/// The smallest window restored: the desktop app's minimum inner size.
pub const MIN_SIZE: egui::Vec2 = egui::vec2(760.0, 480.0);
/// Larger saved sizes are corrupt. eframe also shrinks the start size to the largest monitor.
const MAX_SIZE: egui::Vec2 = egui::vec2(16384.0, 16384.0);
/// Saved positions farther from the origin than this are corrupt.
const MAX_COORD: f32 = 32768.0;
/// Room a window leaves on each side of its monitor for the taskbar, the Dock or a desktop panel,
/// which the monitor size (all egui reports) includes.
const SYSTEM_BAR: egui::Vec2 = egui::vec2(48.0, 48.0);
/// How much of a window's top edge must be on one monitor for the pointer to drag it back.
const GRIP: egui::Vec2 = egui::vec2(96.0, 16.0);

/// What the host knows about the monitors beyond egui's [`egui::ViewportInfo`] (from WordCraft
/// #4742). Both `None` where it can't list them (Wayland, the web, tests).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Monitors {
    /// The host's [`reachable`] answer for the window.
    pub on_a_monitor: Option<bool>,
    /// The primary monitor's bounds in egui points.
    pub primary: Option<egui::Rect>,
}

/// Whether a [`GRIP`] of `window`'s top edge lies on one of `monitors`, in one shared unit.
/// Monitor sizes alone can't tell: a monitor that went away can leave one of the same size.
pub fn reachable(window: egui::Rect, monitors: &[egui::Rect]) -> bool {
    let top = egui::Rect::from_min_size(window.min, egui::vec2(window.width(), window.height().min(GRIP.y)));
    let need = GRIP.min(top.size());
    monitors.iter().any(|m| {
        let on = top.intersect(*m);
        on.is_positive() && on.width() >= need.x && on.height() >= need.y
    })
}

/// The window as it was when the app last closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowGeometry {
    /// Top-left corner: of the frame, or of the content on macOS (where winit places a new
    /// window by it). `None` where the platform doesn't report it (Wayland).
    pub position: Option<[f32; 2]>,
    /// Content size of the un-maximized window.
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
    /// Size of the monitor the window was on, to notice when that monitor is gone.
    pub monitor: Option<[f32; 2]>,
}

impl WindowGeometry {
    /// The geometry to save, given the last saved one and the window as it is now. While the
    /// window is minimized, full screen or maximized, the last un-maximized size and position are
    /// kept, so un-maximizing after the next launch returns to them.
    pub fn track(prev: Option<Self>, info: &egui::ViewportInfo, content_size: egui::Vec2) -> Option<Self> {
        if info.minimized == Some(true) || info.fullscreen == Some(true) {
            return prev;
        }
        // egui reads the maximized state on macOS only when the window opens (egui#3494), so a
        // window that opened zoomed would stay "maximized" for good. There the zoomed frame is
        // saved like any other size instead.
        let maximized = info.maximized == Some(true) && !cfg!(target_os = "macos");
        if maximized && let Some(prev) = prev {
            return Some(Self { maximized, ..prev });
        }
        let rect = if cfg!(target_os = "macos") { info.inner_rect } else { info.outer_rect };
        let size = info.inner_rect.map_or(content_size, |r| r.size());
        let position = rect.map(|r| [r.min.x, r.min.y]);
        let monitor = info.monitor_size.map(|s| [s.x, s.y]);
        Self { position, width: size.x, height: size.y, maximized, monitor }.sanitized().or(prev)
    }

    /// `None` for a size that isn't positive and finite. Otherwise the size is clamped to
    /// [`MIN_SIZE`]..=16384, and a position or monitor size that can't be real is dropped.
    pub fn sanitized(self) -> Option<Self> {
        let size = egui::vec2(self.width, self.height);
        if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }
        let size = size.clamp(MIN_SIZE, MAX_SIZE);
        let position = self.position.filter(|p| p.iter().all(|c| c.is_finite() && c.abs() <= MAX_COORD));
        let monitor = self.monitor.filter(|m| m.iter().all(|c| c.is_finite() && *c > 0.0 && *c <= MAX_SIZE.x));
        Some(Self { position, width: size.x, height: size.y, maximized: self.maximized, monitor })
    }

    /// Whether the window is maximized after it has opened rather than as it opens. On Windows,
    /// winit shows a window the moment it creates it maximized, even one that is meant to stay
    /// hidden until the first frame (eframe's white-flash guard), so a maximized window flashed
    /// on and off about four times at start-up. There it opens at its normal size, hidden, and the
    /// first frame maximizes it ([`Self::maximize_after_open`]).
    pub fn maximizes_after_open(self) -> bool {
        self.maximized && cfg!(windows)
    }

    /// The command that maximizes the window after it opened, if [`Self::maximizes_after_open`].
    pub fn maximize_after_open(self) -> Option<egui::ViewportCommand> {
        self.maximizes_after_open().then_some(egui::ViewportCommand::Maximized(true))
    }

    /// Open the window at this size, position and maximized state.
    pub fn apply(self, builder: egui::ViewportBuilder) -> egui::ViewportBuilder {
        let builder = builder.with_inner_size([self.width, self.height]).with_maximized(self.maximized && !self.maximizes_after_open());
        match self.position {
            Some(p) => builder.with_position(p),
            None => builder,
        }
    }

    /// After a restore: where to move the window when it opened on no monitor (`monitors` says
    /// so: a monitor of the same size replaced its own), or when the monitor it now reports
    /// differs from the one it was saved on (an external monitor was unplugged, say), so it
    /// doesn't open off screen. Centred on the primary monitor (its top-left corner is the origin
    /// when the host can't say where it is). A maximized window fills whichever monitor the
    /// system picks, so it stays put.
    pub fn rescue_position(&self, info: &egui::ViewportInfo, monitors: Monitors) -> Option<egui::Pos2> {
        if self.maximized || self.position.is_none() {
            return None;
        }
        let now = info.monitor_size.filter(|s| s.is_finite() && s.x > 0.0 && s.y > 0.0)?;
        let other_monitor = self.monitor.is_some_and(|saved| (now - egui::Vec2::from(saved)).length() > 1.0);
        if !other_monitor && monitors.on_a_monitor != Some(false) {
            return None;
        }
        let monitor =
            monitors.primary.filter(|r| r.is_finite() && r.is_positive()).unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, now));
        let free = monitor.size() - egui::vec2(self.width, self.height);
        Some(monitor.min + (free / 2.0).max(egui::Vec2::ZERO))
    }

    /// Once the monitor size is known: a smaller size and a centred position for a window that
    /// doesn't fit on its monitor, frame included (#1215). eframe shrinks the start size to the
    /// monitor's, but not by the frame, the title bar or the taskbar, so on a laptop screen the
    /// default window reached past the screen's edge, and the end of the scroll bar with it: a
    /// drag of the scroll bar stopped where the pointer met the edge. A window the system placed
    /// (no position here) is on the primary monitor, whose top-left corner is the origin, so its
    /// position counts too; a restored one may be on another monitor, so only its size does. A
    /// maximized window fills its monitor already.
    pub fn fit_on_monitor(&self, info: &egui::ViewportInfo) -> Option<(egui::Pos2, egui::Vec2)> {
        if self.maximized || info.maximized == Some(true) || info.fullscreen == Some(true) || info.minimized == Some(true) {
            return None;
        }
        let monitor = info.monitor_size.filter(|s| s.is_finite() && s.x > 0.0 && s.y > 0.0)?;
        let inner = info.inner_rect.filter(|r| r.is_finite())?;
        let outer = info.outer_rect.filter(|r| r.is_finite()).unwrap_or(inner);
        let frame = (outer.size() - inner.size()).max(egui::Vec2::ZERO);
        let room = monitor - 2.0 * SYSTEM_BAR;
        let too_big = outer.width() > room.x || outer.height() > room.y;
        let edge = monitor - SYSTEM_BAR;
        let past_edge = self.position.is_none() && (outer.max.x > edge.x || outer.max.y > edge.y);
        if !too_big && !past_edge {
            return None;
        }
        let size = (room - frame).min(inner.size()).max(MIN_SIZE);
        let free = monitor - (size + frame);
        Some((egui::pos2((free.x / 2.0).max(0.0), (free.y / 2.0).max(0.0)), size))
    }
}

/// Checks the restored geometry in `pending` once, and only once the monitor size is known: while
/// `info` doesn't report a usable one, `pending` is left in place for a later frame. Then it is taken
/// and [`WindowGeometry::rescue_position`] decides where to move the window, if anywhere.
pub fn take_rescue(pending: &mut Option<WindowGeometry>, info: &egui::ViewportInfo, monitors: Monitors) -> Option<egui::Pos2> {
    info.monitor_size.filter(|s| s.is_finite() && s.x > 0.0 && s.y > 0.0)?;
    pending.take()?.rescue_position(info, monitors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(rect: egui::Rect, monitor: egui::Vec2) -> egui::ViewportInfo {
        egui::ViewportInfo { inner_rect: Some(rect), outer_rect: Some(rect), monitor_size: Some(monitor), ..Default::default() }
    }

    fn saved() -> WindowGeometry {
        WindowGeometry { position: Some([100.0, 50.0]), width: 1200.0, height: 800.0, maximized: false, monitor: Some([1920.0, 1080.0]) }
    }

    #[test]
    fn records_the_window_and_restores_it() {
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(1200.0, 800.0));
        let g = WindowGeometry::track(None, &info(rect, egui::vec2(1920.0, 1080.0)), egui::Vec2::ZERO);
        assert_eq!(g, Some(saved()));
        let b = saved().apply(egui::ViewportBuilder::default());
        assert_eq!(b.inner_size, Some(egui::vec2(1200.0, 800.0)));
        assert_eq!(b.position, Some(egui::pos2(100.0, 50.0)));
        assert_eq!(b.maximized, Some(false));
    }

    #[test]
    fn a_maximized_window_opens_normal_on_windows_and_is_maximized_by_the_first_frame() {
        let g = WindowGeometry { maximized: true, ..saved() };
        let b = g.apply(egui::ViewportBuilder::default());
        // Normal size and position either way, so un-maximizing returns to them.
        assert_eq!(b.inner_size, Some(egui::vec2(1200.0, 800.0)));
        assert_eq!(b.position, Some(egui::pos2(100.0, 50.0)));
        if cfg!(windows) {
            assert_eq!(b.maximized, Some(false), "winit would show the hidden window while creating it maximized");
            assert_eq!(g.maximize_after_open(), Some(egui::ViewportCommand::Maximized(true)));
        } else {
            assert_eq!(b.maximized, Some(true));
            assert_eq!(g.maximize_after_open(), None);
        }
        assert_eq!(saved().maximize_after_open(), None, "a normal window needs no command");
    }

    #[test]
    fn maximized_keeps_the_normal_size_and_position() {
        let full = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1040.0));
        let mut i = info(full, egui::vec2(1920.0, 1080.0));
        i.maximized = Some(true);
        let g = WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO);
        if cfg!(target_os = "macos") {
            // The zoomed frame is saved as the size; the window never reopens "maximized".
            assert_eq!(g.map(|g| (g.position, g.width, g.height, g.maximized)), Some((Some([0.0, 0.0]), 1920.0, 1040.0, false)));
        } else {
            assert_eq!(g, Some(WindowGeometry { maximized: true, ..saved() }));
        }
        // Minimized or full screen: nothing changes.
        i.maximized = Some(false);
        i.minimized = Some(true);
        assert_eq!(WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO), Some(saved()));
        i.minimized = None;
        i.fullscreen = Some(true);
        assert_eq!(WindowGeometry::track(Some(saved()), &i, egui::Vec2::ZERO), Some(saved()));
    }

    #[test]
    fn without_a_position_the_content_size_is_kept() {
        // Wayland reports neither rect.
        let i = egui::ViewportInfo::default();
        let g = WindowGeometry::track(None, &i, egui::vec2(1000.0, 700.0));
        assert_eq!(g, Some(WindowGeometry { position: None, width: 1000.0, height: 700.0, maximized: false, monitor: None }));
        assert_eq!(g.map(|g| g.apply(egui::ViewportBuilder::default()).position), Some(None));
    }

    #[test]
    fn insane_values_are_clamped_or_dropped() {
        let zero = WindowGeometry { width: 0.0, height: 800.0, ..saved() };
        assert_eq!(zero.sanitized(), None);
        assert_eq!(WindowGeometry::default().sanitized(), None);
        let nan = WindowGeometry { width: f32::NAN, ..saved() };
        assert_eq!(nan.sanitized(), None);
        let tiny = WindowGeometry { width: 10.0, height: 1.0e9, ..saved() }.sanitized();
        assert_eq!(tiny.map(|g| (g.width, g.height)), Some((MIN_SIZE.x, 16384.0)));
        let far = WindowGeometry { position: Some([1.0e7, 0.0]), monitor: Some([f32::INFINITY, 1080.0]), ..saved() }.sanitized();
        assert_eq!(far.map(|g| (g.position, g.monitor)), Some((None, None)));
        assert_eq!(saved().sanitized(), Some(saved()));
    }

    #[test]
    fn a_window_whose_monitor_is_gone_is_moved_on_screen() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // Same monitor: stays where it was.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1920.0, 1080.0)), Monitors::default()), None);
        // A different (smaller) monitor now: centred on it.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1600.0, 900.0)), Monitors::default()), Some(egui::pos2(200.0, 50.0)));
        // Larger than that monitor: its top-left corner.
        assert_eq!(saved().rescue_position(&info(on, egui::vec2(1024.0, 768.0)), Monitors::default()), Some(egui::Pos2::ZERO));
        // Nothing to compare, or no saved position: no move.
        assert_eq!(WindowGeometry { monitor: None, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), Monitors::default()), None);
        assert_eq!(WindowGeometry { position: None, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), Monitors::default()), None);
        assert_eq!(WindowGeometry { maximized: true, ..saved() }.rescue_position(&info(on, egui::vec2(1600.0, 900.0)), Monitors::default()), None);
        assert_eq!(saved().rescue_position(&egui::ViewportInfo::default(), Monitors::default()), None);
    }

    #[test]
    fn the_rescue_waits_for_the_monitor_size() {
        let on = egui::Rect::from_min_size(egui::pos2(2000.0, 50.0), egui::vec2(1200.0, 800.0));
        // The first frame doesn't know the monitor yet: nothing happens and the geometry is kept.
        let mut pending = Some(saved());
        assert_eq!(take_rescue(&mut pending, &egui::ViewportInfo::default(), Monitors::default()), None);
        assert_eq!(pending, Some(saved()));
        // A later frame reports a different monitor: centred on it, and the check is done.
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1600.0, 900.0)), Monitors::default()), Some(egui::pos2(200.0, 50.0)));
        assert_eq!(pending, None);
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1024.0, 768.0)), Monitors::default()), None);
        // The same monitor: no move, and the check is done as well.
        let mut pending = Some(saved());
        assert_eq!(take_rescue(&mut pending, &info(on, egui::vec2(1920.0, 1080.0)), Monitors::default()), None);
        assert_eq!(pending, None);
    }

    #[test]
    fn survives_the_prefs_round_trip() {
        let ui = crate::UiState { window: Some(saved()), ..Default::default() };
        let json = serde_json::to_string(&ui).unwrap();
        let back: crate::UiState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.window, Some(saved()));
        // Prefs written before #23 have no window.
        let old: crate::UiState = serde_json::from_str(r#"{"tab":"Insert"}"#).unwrap();
        assert_eq!(old.window, None);
        assert_eq!(old.tab, "Insert");
    }

    /// A window opened with its frame's top-left corner at `at`: 8 pt borders, a 31 pt title bar.
    fn opened(at: egui::Pos2, content: egui::Vec2, monitor: egui::Vec2) -> egui::ViewportInfo {
        egui::ViewportInfo {
            inner_rect: Some(egui::Rect::from_min_size(at + egui::vec2(8.0, 31.0), content)),
            outer_rect: Some(egui::Rect::from_min_size(at, content + egui::vec2(16.0, 39.0))),
            monitor_size: Some(monitor),
            ..Default::default()
        }
    }

    #[test]
    fn a_window_too_big_for_its_screen_shrinks_to_fit() {
        let system = WindowGeometry::default();
        let at = egui::pos2(78.0, 78.0);
        // A 1920 × 1080 laptop screen at 125 %: eframe gave the default window the screen's height,
        // so the frame and the title bar push its bottom, and the scroll bar's end, off screen.
        let laptop = egui::vec2(1536.0, 864.0);
        let fitted = Some((egui::pos2(48.0, 48.0), egui::vec2(1424.0, 729.0)));
        assert_eq!(system.fit_on_monitor(&opened(at, egui::vec2(1440.0, 864.0), laptop)), fitted);
        // The same size restored from an earlier launch shrinks too.
        assert_eq!(saved().fit_on_monitor(&opened(at, egui::vec2(1440.0, 864.0), laptop)), fitted);
        // Full HD: the default size fits, but where the system put it the bottom reaches into the
        // taskbar, so it is only centred.
        let full_hd = egui::vec2(1920.0, 1080.0);
        let start = egui::vec2(1440.0, 920.0);
        assert_eq!(system.fit_on_monitor(&opened(at, start, full_hd)), Some((egui::pos2(232.0, 60.5), start)));
        // A restored window keeps its place: it may be on another monitor (its origin isn't known).
        assert_eq!(saved().fit_on_monitor(&opened(at, start, full_hd)), None);
        assert_eq!(saved().fit_on_monitor(&opened(egui::pos2(2000.0, 78.0), start, full_hd)), None);
        // Room to spare: nothing moves.
        assert_eq!(system.fit_on_monitor(&opened(at, start, egui::vec2(2560.0, 1440.0))), None);
        // Once fitted, the window saved and restored on the next launch stays as it is.
        let again = WindowGeometry { position: Some([48.0, 48.0]), width: 1424.0, height: 729.0, ..saved() };
        assert_eq!(again.fit_on_monitor(&opened(egui::pos2(48.0, 48.0), egui::vec2(1424.0, 729.0), laptop)), None);
        // Never smaller than the minimum size, and never off the top-left corner.
        let tiny = egui::vec2(800.0, 600.0);
        assert_eq!(system.fit_on_monitor(&opened(at, start, tiny)), Some((egui::pos2(12.0, 40.5), MIN_SIZE)));
        // Maximized, full screen or minimized, or the monitor not known yet: nothing to do.
        let mut i = opened(at, egui::vec2(1440.0, 864.0), laptop);
        assert_eq!(WindowGeometry { maximized: true, ..saved() }.fit_on_monitor(&i), None);
        i.maximized = Some(true);
        assert_eq!(system.fit_on_monitor(&i), None);
        i.maximized = None;
        i.fullscreen = Some(true);
        assert_eq!(system.fit_on_monitor(&i), None);
        i.fullscreen = None;
        i.minimized = Some(true);
        assert_eq!(system.fit_on_monitor(&i), None);
        i.minimized = None;
        i.monitor_size = None;
        assert_eq!(system.fit_on_monitor(&i), None);
        // Wayland reports no rects.
        assert_eq!(system.fit_on_monitor(&egui::ViewportInfo { monitor_size: Some(laptop), ..Default::default() }), None);
    }
}

#[cfg(test)]
mod monitor_tests {
    use super::*;

    fn saved() -> WindowGeometry {
        WindowGeometry { position: Some([2000.0, 50.0]), width: 1200.0, height: 800.0, maximized: false, monitor: Some([1920.0, 1080.0]) }
    }

    fn info(outer: egui::Rect, monitor: egui::Vec2) -> egui::ViewportInfo {
        egui::ViewportInfo { outer_rect: Some(outer), inner_rect: Some(outer), monitor_size: Some(monitor), ..Default::default() }
    }

    /// From WordCraft #4742: a monitor of the same size replaced the window's own, so sizes alone
    /// can't tell it is off screen; the host's monitor list can.
    #[test]
    fn a_window_on_no_monitor_is_moved_on_screen_when_a_same_size_monitor_replaced_its_own() {
        let off = egui::Rect::from_min_size(egui::pos2(-1920.0, -1080.0), egui::vec2(1200.0, 800.0));
        let lost = Monitors { on_a_monitor: Some(false), primary: None };
        let same = info(off, egui::vec2(1920.0, 1080.0));
        assert_eq!(saved().rescue_position(&same, lost), Some(egui::pos2(360.0, 140.0)));
        assert_eq!(WindowGeometry { monitor: None, ..saved() }.rescue_position(&same, lost), Some(egui::pos2(360.0, 140.0)));
        assert_eq!(WindowGeometry { maximized: true, ..saved() }.rescue_position(&same, lost), None);
        assert_eq!(saved().rescue_position(&same, Monitors { on_a_monitor: Some(true), primary: None }), None);
    }

    #[test]
    fn only_a_grippable_top_edge_on_a_monitor_is_reachable() {
        let left = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0));
        let right = egui::Rect::from_min_size(egui::pos2(1920.0, 0.0), egui::vec2(1920.0, 1080.0));
        let monitors = [left, right];
        let at = |x: f32, y: f32| egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(1440.0, 920.0));
        assert!(reachable(at(100.0, 50.0), &monitors));
        assert!(reachable(at(1000.0, 50.0), &monitors), "across both monitors");
        assert!(reachable(at(3840.0 - 96.0, 50.0), &monitors), "just enough of the right edge");
        assert!(!reachable(at(3840.0 - 95.0, 50.0), &monitors));
        assert!(!reachable(at(-1920.0, -1080.0), &monitors), "the gone monitor above-left");
        assert!(!reachable(at(100.0, -900.0), &monitors), "the top edge above every monitor");
        assert!(!reachable(at(100.0, 50.0), &[]));
        assert!(
            reachable(egui::Rect::from_min_size(egui::pos2(10.0, 1072.0), egui::vec2(40.0, 8.0)), &monitors),
            "smaller than the grip at the monitor edge"
        );
    }

    #[test]
    fn an_unreachable_window_is_centered_on_the_primary_monitor_when_monitor_sizes_differ() {
        let primary = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 1080.0));
        let off = egui::Rect::from_min_size(egui::pos2(-3840.0, 0.0), egui::vec2(1200.0, 800.0));
        let now = info(off, egui::vec2(3840.0, 2160.0));
        assert_eq!(saved().rescue_position(&now, Monitors { on_a_monitor: Some(false), primary: Some(primary) }), Some(egui::pos2(40.0, 140.0)));
        let primary_left = egui::Rect::from_min_size(egui::pos2(-1280.0, 120.0), egui::vec2(1280.0, 1024.0));
        assert_eq!(
            saved().rescue_position(&now, Monitors { on_a_monitor: Some(false), primary: Some(primary_left) }),
            Some(egui::pos2(-1240.0, 232.0))
        );
    }
}
