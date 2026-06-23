//! `toggleActions` parsing and the four-phase lifecycle state machine.
//!
//! The state machine itself lives in the runtime layer (Phase 3+); this module
//! only provides the pure parsing and lookup primitives the runtime uses to
//! map a phase onto an [`Action`].

/// The eight `toggleActions` keywords supported by GSAP ScrollTrigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Play,
    Pause,
    Resume,
    Reset,
    Restart,
    Complete,
    Reverse,
    None,
}

/// The four lifecycle phases of a scroll trigger.
///
/// Discriminants are explicit and ordered so `as usize` indexes directly into a
/// `[Action; 4]` produced by [`parse_toggle_actions`].
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum TogglePhase {
    /// Forward crossing of the start boundary into the active range.
    OnEnter = 0,
    /// Forward crossing of the end boundary out of the active range.
    OnLeave = 1,
    /// Backward crossing of the end boundary into the active range.
    OnEnterBack = 2,
    /// Backward crossing of the start boundary out of the active range.
    OnLeaveBack = 3,
}

/// Scroll direction sign: `1` for forward, `-1` for backward.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum ScrollDirection {
    Forward = 1,
    Backward = -1,
}

impl ScrollDirection {
    /// Converts a signed direction value, returning `None` for `0` or any
    /// value outside `{-1, 1}`.
    pub fn from_i8(v: i8) -> Option<ScrollDirection> {
        match v {
            1 => Some(ScrollDirection::Forward),
            -1 => Some(ScrollDirection::Backward),
            _ => None,
        }
    }
}

/// Parses a single action keyword, case-insensitively. Returns `None` (the
/// `Option`) for unknown input.
pub fn parse_action(s: &str) -> Option<Action> {
    match s.trim().to_ascii_lowercase().as_str() {
        "play" => Some(Action::Play),
        "pause" => Some(Action::Pause),
        "resume" => Some(Action::Resume),
        "reset" => Some(Action::Reset),
        "restart" => Some(Action::Restart),
        "complete" => Some(Action::Complete),
        "reverse" => Some(Action::Reverse),
        "none" => Some(Action::None),
        _ => None,
    }
}

/// Parses a full `"onEnter onLeave onEnterBack onLeaveBack"` toggleActions
/// string into a `[Action; 4]`. Returns `None` if the token count is not
/// exactly four or any token is invalid. The GSAP default is
/// `"play none none none"`.
pub fn parse_toggle_actions(s: &str) -> Option<[Action; 4]> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out = [Action::None; 4];
    for (i, part) in parts.iter().enumerate() {
        out[i] = parse_action(part)?;
    }
    Some(out)
}

/// Returns the action mapped to the given phase from a parsed `toggleActions`
/// array.
pub fn action_for(actions: [Action; 4], phase: TogglePhase) -> Action {
    actions[phase as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_keywords() {
        assert_eq!(parse_action("play"), Some(Action::Play));
        assert_eq!(parse_action("pause"), Some(Action::Pause));
        assert_eq!(parse_action("resume"), Some(Action::Resume));
        assert_eq!(parse_action("reset"), Some(Action::Reset));
        assert_eq!(parse_action("restart"), Some(Action::Restart));
        assert_eq!(parse_action("complete"), Some(Action::Complete));
        assert_eq!(parse_action("reverse"), Some(Action::Reverse));
        assert_eq!(parse_action("none"), Some(Action::None));
    }

    #[test]
    fn parses_case_insensitively() {
        assert_eq!(parse_action("PLAY"), Some(Action::Play));
        assert_eq!(parse_action("Resume"), Some(Action::Resume));
        assert_eq!(parse_action("NoNe"), Some(Action::None));
    }

    #[test]
    fn rejects_unknown_keyword() {
        assert_eq!(parse_action("banana"), None);
        assert_eq!(parse_action(""), None);
    }

    #[test]
    fn rejects_wrong_token_count() {
        assert_eq!(parse_toggle_actions("play none none"), None);
        assert_eq!(parse_toggle_actions("play none none none none"), None);
        assert_eq!(parse_toggle_actions(""), None);
    }

    #[test]
    fn rejects_invalid_token() {
        assert_eq!(parse_toggle_actions("play banana none none"), None);
    }

    #[test]
    fn parses_default() {
        let actions = parse_toggle_actions("play none none none").unwrap();
        assert_eq!(actions, [Action::Play, Action::None, Action::None, Action::None]);
    }

    #[test]
    fn phase_discriminants_are_ordered() {
        assert_eq!(TogglePhase::OnEnter as usize, 0);
        assert_eq!(TogglePhase::OnLeave as usize, 1);
        assert_eq!(TogglePhase::OnEnterBack as usize, 2);
        assert_eq!(TogglePhase::OnLeaveBack as usize, 3);
    }

    #[test]
    fn action_for_returns_correct_action() {
        let actions = parse_toggle_actions("play pause resume reset").unwrap();
        assert_eq!(action_for(actions, TogglePhase::OnEnter), Action::Play);
        assert_eq!(action_for(actions, TogglePhase::OnLeave), Action::Pause);
        assert_eq!(action_for(actions, TogglePhase::OnEnterBack), Action::Resume);
        assert_eq!(action_for(actions, TogglePhase::OnLeaveBack), Action::Reset);
    }

    #[test]
    fn scroll_direction_from_i8() {
        assert_eq!(ScrollDirection::from_i8(1), Some(ScrollDirection::Forward));
        assert_eq!(ScrollDirection::from_i8(-1), Some(ScrollDirection::Backward));
        assert_eq!(ScrollDirection::from_i8(0), None);
        assert_eq!(ScrollDirection::from_i8(2), None);
    }

    #[test]
    fn action_for_on_leave_back_is_none_by_default() {
        let actions = [Action::Play, Action::None, Action::None, Action::None];
        assert_eq!(action_for(actions, TogglePhase::OnLeaveBack), Action::None);
    }

    #[test]
    fn action_for_on_enter_back_is_none_by_default() {
        let actions = [Action::Play, Action::None, Action::None, Action::None];
        assert_eq!(action_for(actions, TogglePhase::OnEnterBack), Action::None);
    }

    #[test]
    fn parse_toggle_with_extra_whitespace() {
        let parsed = parse_toggle_actions("play  none  none  none").unwrap();
        assert_eq!(parsed, [Action::Play, Action::None, Action::None, Action::None]);
    }

    #[test]
    fn parse_toggle_with_leading_trailing_whitespace() {
        let parsed = parse_toggle_actions("  play none none none  ").unwrap();
        assert_eq!(parsed, [Action::Play, Action::None, Action::None, Action::None]);
    }

    #[test]
    fn parse_toggle_lowercase_mixed_case() {
        let parsed = parse_toggle_actions("play None NONE none").unwrap();
        assert_eq!(parsed[0], Action::Play);
        assert_eq!(parsed[1], Action::None);
        assert_eq!(parsed[2], Action::None);
        assert_eq!(parsed[3], Action::None);
    }
}