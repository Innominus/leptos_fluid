//! Top-level scroll trigger configuration.
//!
//! `ScrollTriggerConfig` is a plain tunable-parameters bundle. Runtime concerns
//! (trigger element, scroller element, horizontal mode) live in Phase 3
//! `trigger.rs`. String-tuned fields use plain `String` rather than
//! `Cow<'static, str>` for simplicity and consistency with how the motion
//! crate treats small config strings: these strings are short, parsed once, and
//! rarely hot; the `Cow` borrow optimization is not worth the API noise here.

use crate::callbacks::{ScrollCallback, ScrollTriggerEvent, scroll_callback};
use crate::toggle::{TogglePhase, action_for, parse_toggle_actions};
use crate::position::ScrollPosition;

/// Scrub configuration: link progress directly, smooth with a catch-up
/// duration, or disable scrubbing entirely (callbacks only).
#[derive(Clone, Debug, PartialEq)]
pub enum Scrub {
    /// `Bool(true)` links progress directly to scroll; `Bool(false)` disables
    /// scrubbing (callbacks only).
    Bool(bool),
    /// Smooth scrubbing with `t` seconds catch-up.
    Number(f64),
}

impl Default for Scrub {
    fn default() -> Self {
        Scrub::Bool(false)
    }
}

impl Scrub {
    /// Returns true when scrubbing is enabled (any variant except
    /// `Scrub::Bool(false)`).
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Scrub::Bool(false))
    }

    /// Returns the smoothing duration in seconds when `Scrub::Number(t)`.
    pub fn smoothing_secs(&self) -> Option<f64> {
        match self {
            Scrub::Number(t) => Some(*t),
            _ => None,
        }
    }
}

/// Engine-global `prefers-reduced-motion` posture.
///
/// When set to `Respect`, the engine checks
/// `window.matchMedia("(prefers-reduced-motion: reduce)")` and, if it matches,
/// snaps `Scrub::Number` triggers to raw progress (skipping the continuous
/// smoothing rAF loop). Phase callbacks still fire so users can hook
/// reduced-motion-aware animations. Defaults to `Ignore` — the engine does NOT
/// auto-respect the media query; callers opt in via
/// `crate::engine::set_reduced_motion(ReducedMotion::Respect)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReducedMotion {
    /// Ignore `prefers-reduced-motion` (default; matches GSAP's posture).
    #[default]
    Ignore,
    /// Respect `prefers-reduced-motion: reduce` — snap scrub to raw, keep
    /// callbacks.
    Respect,
}

/// Wrapper around the four-element `toggleActions` array. Defaults to the GSAP
/// default `"play none none none"`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleActions(pub [crate::toggle::Action; 4]);

impl Default for ToggleActions {
    fn default() -> Self {
        ToggleActions([
            crate::toggle::Action::Play,
            crate::toggle::Action::None,
            crate::toggle::Action::None,
            crate::toggle::Action::None,
        ])
    }
}

impl ToggleActions {
    /// Parses a `"onEnter onLeave onEnterBack onLeaveBack"` string.
    pub fn parse(s: &str) -> Option<Self> {
        parse_toggle_actions(s).map(ToggleActions)
    }

    /// Returns the action mapped to the given phase.
    pub fn action_for(&self, phase: TogglePhase) -> crate::toggle::Action {
        action_for(self.0, phase)
    }
}

/// Tunable parameters for a scroll trigger.
///
/// `PartialEq` is intentionally not derived because `ScrollCallback`
/// (`Rc<dyn Fn>`) is not `PartialEq`. `Debug` is implemented manually because the
/// callback slots are not `Debug`.
#[derive(Clone)]
pub struct ScrollTriggerConfig {
    /// Start position string (default `"top bottom"`).
    pub start: String,
    /// End position string (default `"bottom top"`).
    pub end: String,
    /// Scrub mode (default `Scrub::Bool(false)`).
    pub scrub: Scrub,
    /// `toggleActions` mapping (default `"play none none none"`).
    pub toggle_actions: ToggleActions,
    /// Fire only once then kill (default `false`).
    pub once: bool,
    /// Optional trigger id for labeling/debug.
    pub id: Option<String>,
    pub on_enter: Option<ScrollCallback>,
    pub on_leave: Option<ScrollCallback>,
    pub on_enter_back: Option<ScrollCallback>,
    pub on_leave_back: Option<ScrollCallback>,
    pub on_toggle: Option<ScrollCallback>,
    pub on_update: Option<ScrollCallback>,
    pub on_refresh: Option<ScrollCallback>,
    /// Fires the first time a `Scrub::Number` trigger converges to its target
    /// after being non-converged. Mirrors GSAP's expo-tween settle callback.
    pub on_scrub_complete: Option<ScrollCallback>,
}

impl std::fmt::Debug for ScrollTriggerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollTriggerConfig")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("scrub", &self.scrub)
            .field("toggle_actions", &self.toggle_actions)
            .field("once", &self.once)
            .field("id", &self.id)
            .field("on_enter", &self.on_enter.is_some())
            .field("on_leave", &self.on_leave.is_some())
            .field("on_enter_back", &self.on_enter_back.is_some())
            .field("on_leave_back", &self.on_leave_back.is_some())
            .field("on_toggle", &self.on_toggle.is_some())
            .field("on_update", &self.on_update.is_some())
            .field("on_refresh", &self.on_refresh.is_some())
            .field("on_scrub_complete", &self.on_scrub_complete.is_some())
            .finish()
    }
}

impl Default for ScrollTriggerConfig {
    fn default() -> Self {
        Self {
            start: "top bottom".to_string(),
            end: "bottom top".to_string(),
            scrub: Scrub::default(),
            toggle_actions: ToggleActions::default(),
            once: false,
            id: None,
            on_enter: None,
            on_leave: None,
            on_enter_back: None,
            on_leave_back: None,
            on_toggle: None,
            on_update: None,
            on_refresh: None,
            on_scrub_complete: None,
        }
    }
}

impl ScrollTriggerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(mut self, s: impl Into<String>) -> Self {
        self.start = s.into();
        self
    }

    pub fn end(mut self, s: impl Into<String>) -> Self {
        self.end = s.into();
        self
    }

    pub fn scrub(mut self, s: Scrub) -> Self {
        self.scrub = s;
        self
    }

    pub fn toggle_actions(mut self, t: ToggleActions) -> Self {
        self.toggle_actions = t;
        self
    }

    pub fn once(mut self, b: bool) -> Self {
        self.once = b;
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn on_enter(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_enter = Some(scroll_callback(f));
        self
    }

    pub fn on_leave(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_leave = Some(scroll_callback(f));
        self
    }

    pub fn on_enter_back(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_enter_back = Some(scroll_callback(f));
        self
    }

    pub fn on_leave_back(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_leave_back = Some(scroll_callback(f));
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_toggle = Some(scroll_callback(f));
        self
    }

    pub fn on_update(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_update = Some(scroll_callback(f));
        self
    }

    pub fn on_refresh(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_refresh = Some(scroll_callback(f));
        self
    }

    /// Sets the `onScrubComplete` callback (fires when `Scrub::Number` settles).
    pub fn on_scrub_complete(mut self, f: impl Fn(ScrollTriggerEvent) + 'static) -> Self {
        self.on_scrub_complete = Some(scroll_callback(f));
        self
    }

    /// Parses `start`/`end` into a `(ScrollPosition, ScrollPosition)` pair.
    /// Returns `None` if either string is unparseable.
    pub fn parse_positions(&self, horizontal: bool) -> Option<(ScrollPosition, ScrollPosition)> {
        crate::position::parse_start_end(&self.start, &self.end, horizontal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toggle::Action;

    #[test]
    fn default_config_has_gsap_defaults() {
        let cfg = ScrollTriggerConfig::default();
        assert_eq!(cfg.start, "top bottom");
        assert_eq!(cfg.end, "bottom top");
        assert_eq!(cfg.scrub, Scrub::Bool(false));
        assert_eq!(cfg.toggle_actions, ToggleActions::default());
        assert!(!cfg.once);
        assert!(cfg.id.is_none());
        assert!(cfg.on_enter.is_none());
    }

    #[test]
    fn builder_methods_chain() {
        let cfg = ScrollTriggerConfig::new()
            .start("top center")
            .end("bottom 80%")
            .scrub(Scrub::Number(0.5))
            .once(true)
            .id("hero")
            .on_enter(|_| {});
        assert_eq!(cfg.start, "top center");
        assert_eq!(cfg.end, "bottom 80%");
        assert_eq!(cfg.scrub, Scrub::Number(0.5));
        assert!(cfg.once);
        assert_eq!(cfg.id.as_deref(), Some("hero"));
        assert!(cfg.on_enter.is_some());
    }

    #[test]
    fn scrub_is_enabled() {
        assert!(!Scrub::Bool(false).is_enabled());
        assert!(Scrub::Bool(true).is_enabled());
        assert!(Scrub::Number(1.0).is_enabled());
    }

    #[test]
    fn scrub_smoothing_secs() {
        assert_eq!(Scrub::Bool(false).smoothing_secs(), None);
        assert_eq!(Scrub::Bool(true).smoothing_secs(), None);
        assert_eq!(Scrub::Number(0.7).smoothing_secs(), Some(0.7));
    }

    #[test]
    fn toggle_actions_default_parses() {
        let parsed = ToggleActions::parse("play none none none").unwrap();
        assert_eq!(parsed, ToggleActions::default());
    }

    #[test]
    fn toggle_actions_invalid_is_none() {
        assert_eq!(ToggleActions::parse("invalid"), None);
        assert_eq!(ToggleActions::parse("play none none"), None);
    }

    #[test]
    fn toggle_actions_action_for_on_enter_is_play() {
        let ta = ToggleActions::default();
        assert_eq!(ta.action_for(TogglePhase::OnEnter), Action::Play);
        assert_eq!(ta.action_for(TogglePhase::OnLeave), Action::None);
        assert_eq!(ta.action_for(TogglePhase::OnEnterBack), Action::None);
        assert_eq!(ta.action_for(TogglePhase::OnLeaveBack), Action::None);
    }

    #[test]
    fn parse_positions_uses_config_strings() {
        let cfg = ScrollTriggerConfig::default();
        let (start, _end) = cfg.parse_positions(false).unwrap();
        assert_eq!(start.trigger, crate::position::ScrollPoint::Top);
    }

    #[test]
    fn reduced_motion_default_is_ignore() {
        assert_eq!(ReducedMotion::default(), ReducedMotion::Ignore);
    }
}