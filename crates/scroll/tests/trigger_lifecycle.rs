//! Public-API smoke tests for `leptos_fluid_scroll`.
//!
//! Integration tests can only access `pub` items — the `host_test_trigger`
//! helper and `engine_update` / `inner` accessors are `pub(crate)` and are
//! therefore out of reach here. The real lifecycle coverage lives in the
//! inline tests under `src/`; these tests verify the public surface compiles
//! and behaves as documented: config defaults/builders, reduced-motion setter,
//! `ToggleActions` parsing, `Scrub` smoothing, and `ScrollTriggerEvent`
//! progress clamping.

use leptos_fluid_scroll::{
    ReducedMotion, Scrub, ScrollTrigger, ScrollTriggerConfig, ScrollTriggerEvent,
    ToggleActions, set_reduced_motion,
};

#[test]
fn config_defaults_match_gsap() {
    let cfg = ScrollTriggerConfig::default();
    assert_eq!(cfg.start, "top bottom");
    assert_eq!(cfg.end, "bottom top");
    assert_eq!(cfg.scrub, Scrub::Bool(false));
    assert!(!cfg.once);
}

#[test]
fn config_builder_methods_chain() {
    let cfg = ScrollTriggerConfig::new()
        .start("top center")
        .end("bottom 80%")
        .scrub(Scrub::Number(0.5))
        .once(true)
        .id("hero");
    assert_eq!(cfg.start, "top center");
    assert_eq!(cfg.end, "bottom 80%");
    assert_eq!(cfg.scrub, Scrub::Number(0.5));
    assert!(cfg.once);
    assert_eq!(cfg.id.as_deref(), Some("hero"));
}

#[test]
fn reduced_motion_is_settable() {
    set_reduced_motion(ReducedMotion::Respect);
    set_reduced_motion(ReducedMotion::Ignore);
}

#[test]
fn toggle_actions_parse_and_default() {
    let ta = ToggleActions::default();
    assert_eq!(ta.0[0], leptos_fluid_scroll::Action::Play);
    let parsed = ToggleActions::parse("play pause resume reset").unwrap();
    assert_eq!(parsed.0[1], leptos_fluid_scroll::Action::Pause);
}

#[test]
fn scrub_smoothing_secs() {
    assert_eq!(Scrub::Bool(false).smoothing_secs(), None);
    assert_eq!(Scrub::Bool(true).smoothing_secs(), None);
    assert_eq!(Scrub::Number(0.7).smoothing_secs(), Some(0.7));
}

#[test]
fn scroll_trigger_event_clamps_progress() {
    let ev = ScrollTriggerEvent::new(-1.0, 1, true, 10.0);
    assert_eq!(ev.progress, 0.0);
    let ev = ScrollTriggerEvent::new(2.0, 1, true, 10.0);
    assert_eq!(ev.progress, 1.0);
}

#[test]
fn scroll_trigger_builder_compiles_without_target() {
    // The builder defers target attachment; constructing it should not require
    // a DOM. `.install()` is not called here — that path needs a target and a
    // live DOM, which host targets cannot provide.
    let _builder = ScrollTrigger::builder();
}