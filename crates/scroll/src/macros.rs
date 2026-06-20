//! The `scroll_trigger!` declarative macro.
//!
//! Mirrors `controller!` / `timeline!` in `crates/motion/src/macros.rs`: a
//! TT-muncher (`__fluid_scroll_parse!`) walks each `field: value,` pair and
//! accumulates state, then `__fluid_scroll_finish!` assembles the typed
//! [`ScrollTriggerBuilder`] calls and invokes [`ReadyScrollTriggerBuilder::install`].
//!
//! Supported fields:
//!
//! - `trigger: $expr` or `resolver: $expr` (exactly one required)
//! - `start: $expr`, `end: $expr`, `once: $expr`, `id: $expr`
//! - `scrub: $expr` (lowers via `__fluid_scroll_build_scrub!`)
//! - `toggle_actions: $expr`
//! - `on_enter` / `on_leave` / `on_enter_back` / `on_leave_back` / `on_toggle`
//!   / `on_update` / `on_refresh`: `$expr`
//! - `bind_controller: ($controller, $style_fn)` (feature `controller`)
//! - `bind_controller_with: ($controller, $transition, $style_fn)` (feature `controller`)
//! - `bind_timeline: ($timeline, $toggle_actions_str)` (feature `timeline`)
//! - `bind_timeline_scrub: ($timeline, $step_count, $style_fn)` (feature `timeline`)
//!
//! Each field may appear at most once; unknown fields and invalid syntax
//! produce `compile_error!`.

#[macro_export]
macro_rules! scroll_trigger {
    ($($tt:tt)*) => {
        $crate::__fluid_scroll_parse! {
            [target none ()]
            [start unset ()]
            [end unset ()]
            [scrub unset ()]
            [toggle_actions unset ()]
            [once unset ()]
            [id unset ()]
            [on_enter unset ()]
            [on_leave unset ()]
            [on_enter_back unset ()]
            [on_leave_back unset ()]
            [on_toggle unset ()]
            [on_update unset ()]
            [on_refresh unset ()]
            [bind_controller unset ()]
            [bind_controller_with unset ()]
            [bind_timeline unset ()]
            [bind_timeline_scrub unset ()]
            $($tt)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_parse {
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {
        $crate::__fluid_scroll_finish! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        }
    };
    (
        [target none ()]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        trigger: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target trigger ($next)]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target trigger $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        trigger: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `trigger` can only be specified once")
    };
    (
        [target resolver $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        trigger: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! accepts either `trigger:` or `resolver:`, not both")
    };
    (
        [target none ()]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target resolver ($next)]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target resolver $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `resolver` can only be specified once")
    };
    (
        [target trigger $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! accepts either `trigger:` or `resolver:`, not both")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start unset ()]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        start: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start set ($next)]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start set $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        start: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `start` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end unset ()]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        end: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end set ($next)]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end set $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        end: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `end` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub unset ()]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        scrub: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub set ($next)]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub set $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        scrub: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `scrub` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions unset ()]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        toggle_actions: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions set ($next)]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions set $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        toggle_actions: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `toggle_actions` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once unset ()]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        once: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once set ($next)]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once set $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        once: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `once` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id unset ()]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        id: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id set ($next)]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id set $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        id: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `id` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter unset ()]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_enter: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter set ($next)]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter set $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_enter: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_enter` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave unset ()]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_leave: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave set ($next)]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave set $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_leave: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_leave` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back unset ()]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_enter_back: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back set ($next)]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back set $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_enter_back: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_enter_back` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back unset ()]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_leave_back: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back set ($next)]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back set $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_leave_back: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_leave_back` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle unset ()]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_toggle: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle set ($next)]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle set $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_toggle: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_toggle` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update unset ()]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_update: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update set ($next)]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update set $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_update: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_update` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh unset ()]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_refresh: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh set ($next)]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh set $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        on_refresh: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `on_refresh` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller unset ()]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_controller: ($controller:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller set ($controller, $style_fn)]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller set $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_controller: ($controller:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `bind_controller` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with unset ()]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_controller_with: ($controller:expr, $transition:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with set ($controller, $transition, $style_fn)]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with set $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_controller_with: ($controller:expr, $transition:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `bind_controller_with` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline unset ()]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_timeline: ($timeline:expr, $toggle_actions:expr) $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline set ($timeline, $toggle_actions)]
            [bind_timeline_scrub $bts_state $bts_value]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline set $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        bind_timeline: ($timeline:expr, $toggle_actions:expr) $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `bind_timeline` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub unset ()]
        bind_timeline_scrub: ($timeline:expr, $step_count:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_scroll_parse! {
            [target $target_state $target_value]
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub set ($timeline, $step_count, $style_fn)]
            $($($rest)*)?
        }
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub set $bts_value:tt]
        bind_timeline_scrub: ($timeline:expr, $step_count:expr, $style_fn:expr) $(, $($rest:tt)*)?
    ) => {
        compile_error!("scroll_trigger! field `bind_timeline_scrub` can only be specified once")
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        $field:ident : $($rest:tt)+
    ) => {
        compile_error!(concat!("unknown field in scroll_trigger!: `", stringify!($field), "`"))
    };
    (
        [target $target_state:ident $target_value:tt]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
        $($rest:tt)+
    ) => {
        compile_error!("invalid scroll_trigger! syntax")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_finish {
    (
        [target none ()]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {
        compile_error!("scroll_trigger! requires exactly one of `trigger:` or `resolver:`")
    };
    (
        [target trigger ($target:expr)]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {{
        let __fluid_builder = $crate::ScrollTrigger::builder();
        let __fluid_builder = $crate::__fluid_scroll_apply_config!(
            __fluid_builder,
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        );
        let __fluid_builder = $crate::__fluid_scroll_apply_bindings!(
            __fluid_builder,
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        );
        let __fluid_builder = __fluid_builder.target($target);
        __fluid_builder.install()
    }};
    (
        [target resolver ($resolver:expr)]
        [start $start_state:ident $start_value:tt]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
        [bind_controller $bc_state:ident $bc_value:tt]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {{
        let __fluid_builder = $crate::ScrollTrigger::builder();
        let __fluid_builder = $crate::__fluid_scroll_apply_config!(
            __fluid_builder,
            [start $start_state $start_value]
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        );
        let __fluid_builder = $crate::__fluid_scroll_apply_bindings!(
            __fluid_builder,
            [bind_controller $bc_state $bc_value]
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        );
        let __fluid_builder = __fluid_builder.resolver($resolver);
        __fluid_builder.install()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_config {
    (
        $builder:expr,
        [start unset ()]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_config_end!(
            $builder,
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [start set ($start:expr)]
        [end $end_state:ident $end_value:tt]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.start($start);
        $crate::__fluid_scroll_apply_config_end!(
            __fluid_builder,
            [end $end_state $end_value]
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_config_end {
    (
        $builder:expr,
        [end unset ()]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_scrub!(
            $builder,
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [end set ($end:expr)]
        [scrub $scrub_state:ident $scrub_value:tt]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.end($end);
        $crate::__fluid_scroll_apply_scrub!(
            __fluid_builder,
            [scrub $scrub_state $scrub_value]
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_scrub {
    (
        $builder:expr,
        [scrub unset ()]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_ta!(
            $builder,
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [scrub set ($scrub:expr)]
        [toggle_actions $ta_state:ident $ta_value:tt]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder =
            $builder.scrub($crate::__fluid_scroll_build_scrub!($scrub));
        $crate::__fluid_scroll_apply_ta!(
            __fluid_builder,
            [toggle_actions $ta_state $ta_value]
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_ta {
    (
        $builder:expr,
        [toggle_actions unset ()]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_once!(
            $builder,
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [toggle_actions set ($ta:expr)]
        [once $once_state:ident $once_value:tt]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.toggle_actions($crate::__fluid_scroll_build_ta!($ta));
        $crate::__fluid_scroll_apply_once!(
            __fluid_builder,
            [once $once_state $once_value]
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_once {
    (
        $builder:expr,
        [once unset ()]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_id!(
            $builder,
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [once set ($once:expr)]
        [id $id_state:ident $id_value:tt]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.once($once);
        $crate::__fluid_scroll_apply_id!(
            __fluid_builder,
            [id $id_state $id_value]
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_id {
    (
        $builder:expr,
        [id unset ()]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_callbacks!(
            $builder,
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [id set ($id:expr)]
        [on_enter $oe_state:ident $oe_value:tt]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.id($id);
        $crate::__fluid_scroll_apply_callbacks!(
            __fluid_builder,
            [on_enter $oe_state $oe_value]
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_callbacks {
    (
        $builder:expr,
        [on_enter unset ()]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_leave!(
            $builder,
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_enter set ($oe:expr)]
        [on_leave $ol_state:ident $ol_value:tt]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_enter($oe);
        $crate::__fluid_scroll_apply_on_leave!(
            __fluid_builder,
            [on_leave $ol_state $ol_value]
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_leave {
    (
        $builder:expr,
        [on_leave unset ()]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_enter_back!(
            $builder,
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_leave set ($ol:expr)]
        [on_enter_back $oeb_state:ident $oeb_value:tt]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_leave($ol);
        $crate::__fluid_scroll_apply_on_enter_back!(
            __fluid_builder,
            [on_enter_back $oeb_state $oeb_value]
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_enter_back {
    (
        $builder:expr,
        [on_enter_back unset ()]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_leave_back!(
            $builder,
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_enter_back set ($oeb:expr)]
        [on_leave_back $olb_state:ident $olb_value:tt]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_enter_back($oeb);
        $crate::__fluid_scroll_apply_on_leave_back!(
            __fluid_builder,
            [on_leave_back $olb_state $olb_value]
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_leave_back {
    (
        $builder:expr,
        [on_leave_back unset ()]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_toggle!(
            $builder,
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_leave_back set ($olb:expr)]
        [on_toggle $ot_state:ident $ot_value:tt]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_leave_back($olb);
        $crate::__fluid_scroll_apply_on_toggle!(
            __fluid_builder,
            [on_toggle $ot_state $ot_value]
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_toggle {
    (
        $builder:expr,
        [on_toggle unset ()]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_update!(
            $builder,
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_toggle set ($ot:expr)]
        [on_update $ou_state:ident $ou_value:tt]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_toggle($ot);
        $crate::__fluid_scroll_apply_on_update!(
            __fluid_builder,
            [on_update $ou_state $ou_value]
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_update {
    (
        $builder:expr,
        [on_update unset ()]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_on_refresh!(
            $builder,
            [on_refresh $or_state $or_value]
        )
    };
    (
        $builder:expr,
        [on_update set ($ou:expr)]
        [on_refresh $or_state:ident $or_value:tt]
    ) => {{
        let __fluid_builder = $builder.on_update($ou);
        $crate::__fluid_scroll_apply_on_refresh!(
            __fluid_builder,
            [on_refresh $or_state $or_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_on_refresh {
    ($builder:expr, [on_refresh unset ()]) => {
        $builder
    };
    ($builder:expr, [on_refresh set ($or:expr)]) => {
        $builder.on_refresh($or)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_bindings {
    (
        $builder:expr,
        [bind_controller unset ()]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_bind_controller_with!(
            $builder,
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        )
    };
    (
        $builder:expr,
        [bind_controller set ($controller:expr, $style_fn:expr)]
        [bind_controller_with $bcw_state:ident $bcw_value:tt]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {{
        let __fluid_builder = $builder.bind_controller($controller, $style_fn);
        $crate::__fluid_scroll_apply_bind_controller_with!(
            __fluid_builder,
            [bind_controller_with $bcw_state $bcw_value]
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_bind_controller_with {
    (
        $builder:expr,
        [bind_controller_with unset ()]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_bind_timeline!(
            $builder,
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        )
    };
    (
        $builder:expr,
        [bind_controller_with set ($controller:expr, $transition:expr, $style_fn:expr)]
        [bind_timeline $bt_state:ident $bt_value:tt]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {{
        let __fluid_builder = $builder.bind_controller_with($controller, $transition, $style_fn);
        $crate::__fluid_scroll_apply_bind_timeline!(
            __fluid_builder,
            [bind_timeline $bt_state $bt_value]
            [bind_timeline_scrub $bts_state $bts_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_bind_timeline {
    (
        $builder:expr,
        [bind_timeline unset ()]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {
        $crate::__fluid_scroll_apply_bind_timeline_scrub!(
            $builder,
            [bind_timeline_scrub $bts_state $bts_value]
        )
    };
    (
        $builder:expr,
        [bind_timeline set ($timeline:expr, $toggle_actions:expr)]
        [bind_timeline_scrub $bts_state:ident $bts_value:tt]
    ) => {{
        let __fluid_builder = $builder.bind_timeline($timeline, $toggle_actions);
        $crate::__fluid_scroll_apply_bind_timeline_scrub!(
            __fluid_builder,
            [bind_timeline_scrub $bts_state $bts_value]
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_apply_bind_timeline_scrub {
    ($builder:expr, [bind_timeline_scrub unset ()]) => {
        $builder
    };
    ($builder:expr, [bind_timeline_scrub set ($timeline:expr, $step_count:expr, $style_fn:expr)]) => {
        $builder.bind_timeline_scrub($timeline, $step_count, $style_fn)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_build_scrub {
    ($expr:expr) => {{
        let __fluid_scrub_value: $crate::__private::ScrubKind =
            $crate::__private::ScrubKind::from_auto($expr);
        __fluid_scrub_value.into_scrub()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_scroll_build_ta {
    ($s:expr) => {
        $crate::ToggleActions::parse($s).unwrap_or_else(|| $crate::ToggleActions::default())
    };
}

#[cfg(test)]
mod tests {
    use leptos::reactive::owner::Owner;

    #[test]
    fn macro_minimal_with_resolver_compiles_and_installs() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                start: "top center",
                end: "bottom 80%",
                once: false,
            };
        });
    }

    #[test]
    fn macro_with_callbacks_and_scrub_compiles() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                scrub: true,
                on_enter: |_ev| {},
                on_update: |_ev| {},
            };
        });
    }

    #[test]
    fn macro_with_scrub_number_compiles() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                scrub: 0.5,
            };
        });
    }

    #[test]
    fn macro_with_toggle_actions_compiles() {
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                toggle_actions: "play pause resume reset",
            };
        });
    }

    #[cfg(feature = "controller")]
    #[test]
    fn macro_with_bind_controller_compiles() {
        use leptos_fluid_motion::{AnimationController, FluidStyle};
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let controller = AnimationController::new();
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                bind_controller: (controller, |p| FluidStyle::new().opacity(p)),
            };
        });
    }

    #[cfg(feature = "timeline")]
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn macro_with_bind_timeline_compiles() {
        use leptos_fluid_motion::{FluidStyle, FluidTimeline};
        let _ = any_spawner::Executor::init_futures_executor();
        Owner::new().with(|| {
            let timeline = FluidTimeline::new(FluidStyle::new());
            let _trigger = scroll_trigger! {
                resolver: || None::<web_sys::Element>,
                bind_timeline: (timeline, "play none none none"),
            };
        });
    }
}