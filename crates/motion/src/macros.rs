#[macro_export]
macro_rules! controller {
    ($($tt:tt)*) => {
        $crate::__fluid_controller_parse! {
            [transition unset ()]
            [attachment none ()]
            [initial unset ()]
            [animate unset ()]
            $($tt)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_controller_parse {
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
    ) => {
        $crate::__fluid_controller_finish! {
            [transition $transition_state $transition_value]
            [attachment $attachment_state $attachment_value]
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
        }
    };
    (
        [transition unset ()]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        transition: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_controller_parse! {
            [transition set ($next)]
            [attachment $attachment_state $attachment_value]
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
            $($($rest)*)?
        }
    };
    (
        [transition set $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        transition: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! field `transition` can only be specified once")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment none ()]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        target: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_controller_parse! {
            [transition $transition_state $transition_value]
            [attachment target ($next)]
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
            $($($rest)*)?
        }
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment target $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        target: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! field `target` can only be specified once")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment resolver $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        target: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! accepts either `target:` or `resolver:`, not both")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment none ()]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_controller_parse! {
            [transition $transition_state $transition_value]
            [attachment resolver ($next)]
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
            $($($rest)*)?
        }
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment resolver $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! field `resolver` can only be specified once")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment target $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        resolver: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! accepts either `target:` or `resolver:`, not both")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial unset ()]
        [animate $animate_state:ident $animate_value:tt]
        initial: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_controller_parse! {
            [transition $transition_state $transition_value]
            [attachment $attachment_state $attachment_value]
            [initial set ($next)]
            [animate $animate_state $animate_value]
            $($($rest)*)?
        }
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial set $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        initial: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! field `initial` can only be specified once")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate unset ()]
        animate: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_controller_parse! {
            [transition $transition_state $transition_value]
            [attachment $attachment_state $attachment_value]
            [initial $initial_state $initial_value]
            [animate set ($next)]
            $($($rest)*)?
        }
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate set $animate_value:tt]
        animate: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("controller! field `animate` can only be specified once")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        $field:ident : $($rest:tt)+
    ) => {
        compile_error!(concat!("unknown field in controller!: `", stringify!($field), "`"))
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment $attachment_state:ident $attachment_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
        $($rest:tt)+
    ) => {
        compile_error!("invalid controller! syntax")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_controller_finish {
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment none ()]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
    ) => {
        compile_error!("controller! requires exactly one of `target:` or `resolver:`")
    };
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment target ($target:expr)]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
    ) => {{
        let __fluid_builder =
            $crate::__fluid_controller_build!($transition_state, $transition_value);
        let __fluid_builder = __fluid_builder.target($target);
        let __fluid_builder = $crate::__fluid_controller_install_effects!(
            __fluid_builder,
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
        );
        __fluid_builder.install()
    }};
    (
        [transition $transition_state:ident $transition_value:tt]
        [attachment resolver ($resolver:expr)]
        [initial $initial_state:ident $initial_value:tt]
        [animate $animate_state:ident $animate_value:tt]
    ) => {{
        let __fluid_builder =
            $crate::__fluid_controller_build!($transition_state, $transition_value);
        let __fluid_builder = __fluid_builder.resolver($resolver);
        let __fluid_builder = $crate::__fluid_controller_install_effects!(
            __fluid_builder,
            [initial $initial_state $initial_value]
            [animate $animate_state $animate_value]
        );
        __fluid_builder.install()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_controller_build {
    (unset, ()) => {
        $crate::AnimationController::builder()
    };
    (set, ($transition:expr)) => {
        $crate::AnimationController::builder().transition($transition)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_controller_install_effects {
    ($builder:expr, [initial unset ()] [animate unset ()]) => {
        $builder
    };
    ($builder:expr, [initial set ($initial:expr)] [animate unset ()]) => {
        $builder.initial($initial)
    };
    ($builder:expr, [initial unset ()] [animate set ($animate:expr)]) => {
        $builder.animate($animate)
    };
    ($builder:expr, [initial set ($initial:expr)] [animate set ($animate:expr)]) => {
        $builder.initial($initial).animate($animate)
    };
}

#[macro_export]
macro_rules! when {
    (
        controller: $controller:expr,
        $(
            on($watch:expr) {
                $($pattern:pat => $action:ident ( $($args:tt)* )),+ $(,)?
            }
        ),+ $(,)?
    ) => {{
        let __fluid_controller = $controller;
        $(
            $crate::__private::watch_on_change(
                move || $watch,
                {
                    let __fluid_controller = __fluid_controller;
                    move |__fluid_value| match __fluid_value {
                        $(
                            $pattern => {
                                $crate::__fluid_when_apply_controller!(
                                    __fluid_controller,
                                    $action($($args)*)
                                );
                            }
                        ),+
                    }
                },
            );
        )+
    }};
    (
        timeline: $timeline:expr,
        $(
            on($watch:expr) {
                $($pattern:pat => $action:ident ( $($args:tt)* )),+ $(,)?
            }
        ),+ $(,)?
    ) => {{
        let __fluid_timeline = $timeline;
        $(
            $crate::__private::watch_on_change(
                move || $watch,
                {
                    let __fluid_timeline = __fluid_timeline;
                    move |__fluid_value| match __fluid_value {
                        $(
                            $pattern => {
                                $crate::__fluid_when_apply_timeline!(
                                    __fluid_timeline,
                                    $action($($args)*)
                                );
                            }
                        ),+
                    }
                },
            );
        )+
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_when_apply_controller {
    ($controller:expr, animate($style:expr)) => {
        $controller.animate($style)
    };
    ($controller:expr, animate_with($style:expr, $transition:expr)) => {
        $controller.animate_with($style, $transition)
    };
    ($controller:expr, set_immediate($style:expr)) => {
        $controller.set_immediate($style)
    };
    ($controller:expr, stop()) => {
        $controller.stop()
    };
    ($controller:expr, pause()) => {
        let _ = $controller.pause();
    };
    ($controller:expr, resume()) => {
        let _ = $controller.resume();
    };
    ($controller:expr, $action:ident($($args:tt)*)) => {
        compile_error!(concat!(
            "unsupported controller action in when!: ",
            stringify!($action)
        ))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_when_apply_timeline {
    ($timeline:expr, play()) => {
        $timeline.play()
    };
    ($timeline:expr, restart()) => {
        $timeline.restart()
    };
    ($timeline:expr, pause()) => {
        $timeline.pause()
    };
    ($timeline:expr, resume()) => {
        $timeline.resume()
    };
    ($timeline:expr, stop()) => {
        $timeline.stop()
    };
    ($timeline:expr, set_immediate($style:expr)) => {
        $timeline.set_immediate($style)
    };
    ($timeline:expr, $action:ident($($args:tt)*)) => {
        compile_error!(concat!(
            "unsupported timeline action in when!: ",
            stringify!($action)
        ))
    };
}

#[macro_export]
macro_rules! timeline {
    ($($tt:tt)*) => {
        $crate::__fluid_timeline_parse! {
            [controller unset ()]
            [initial unset ()]
            [autoplay unset ()]
            [auto_loop unset ()]
            [steps unset ()]
            [triggers unset ()]
            $($tt)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_parse {
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
    ) => {
        $crate::__fluid_timeline_finish! {
            [controller $controller_state $controller_value]
            [initial $initial_state $initial_value]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps $steps_state $steps_value]
            [triggers $triggers_state $triggers_value]
        }
    };
    (
        [controller unset ()]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        controller: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller set ($next)]
            [initial $initial_state $initial_value]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps $steps_state $steps_value]
            [triggers $triggers_state $triggers_value]
            $($($rest)*)?
        }
    };
    (
        [controller set $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        controller: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `controller` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial unset ()]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        initial: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller $controller_state $controller_value]
            [initial set ($next)]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps $steps_state $steps_value]
            [triggers $triggers_state $triggers_value]
            $($($rest)*)?
        }
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial set $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        initial: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `initial` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay unset ()]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        autoplay: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller $controller_state $controller_value]
            [initial $initial_state $initial_value]
            [autoplay set ($next)]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps $steps_state $steps_value]
            [triggers $triggers_state $triggers_value]
            $($($rest)*)?
        }
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay set $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        autoplay: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `autoplay` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop unset ()]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        auto_loop: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller $controller_state $controller_value]
            [initial $initial_state $initial_value]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop set ($next)]
            [steps $steps_state $steps_value]
            [triggers $triggers_state $triggers_value]
            $($($rest)*)?
        }
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop set $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        auto_loop: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `auto_loop` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps unset ()]
        [triggers $triggers_state:ident $triggers_value:tt]
        steps: [$($next:tt)*] $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller $controller_state $controller_value]
            [initial $initial_state $initial_value]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps set [$($next)*]]
            [triggers $triggers_state $triggers_value]
            $($($rest)*)?
        }
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps set $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        steps: [$($next:tt)*] $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `steps` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers unset ()]
        triggers: [$($next:tt)*] $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_parse! {
            [controller $controller_state $controller_value]
            [initial $initial_state $initial_value]
            [autoplay $autoplay_state $autoplay_value]
            [auto_loop $auto_loop_state $auto_loop_value]
            [steps $steps_state $steps_value]
            [triggers set [$($next)*]]
            $($($rest)*)?
        }
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers set $triggers_value:tt]
        triggers: [$($next:tt)*] $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! field `triggers` can only be specified once")
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        $field:ident : $($rest:tt)+
    ) => {
        compile_error!(concat!("unknown field in timeline!: `", stringify!($field), "`"))
    };
    (
        [controller $controller_state:ident $controller_value:tt]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
        $($rest:tt)+
    ) => {
        compile_error!("invalid timeline! syntax")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_finish {
    (
        [controller unset ()]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps $steps_state:ident $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
    ) => {
        compile_error!("timeline! requires a `controller:` field")
    };
    (
        [controller set ($controller:expr)]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps unset ()]
        [triggers $triggers_state:ident $triggers_value:tt]
    ) => {
        compile_error!("timeline! requires a non-empty `steps:` field")
    };
    (
        [controller set ($controller:expr)]
        [initial $initial_state:ident $initial_value:tt]
        [autoplay $autoplay_state:ident $autoplay_value:tt]
        [auto_loop $auto_loop_state:ident $auto_loop_value:tt]
        [steps set $steps_value:tt]
        [triggers $triggers_state:ident $triggers_value:tt]
    ) => {{
        let __fluid_controller = $controller;
        let __fluid_transition = __fluid_controller.transition();
        let __fluid_builder = $crate::FluidTimeline::builder(__fluid_controller);
        let __fluid_builder = $crate::__fluid_timeline_builder_initial!(
            __fluid_builder,
            $initial_state,
            $initial_value
        );
        let __fluid_builder = $crate::__fluid_timeline_builder_autoplay!(
            __fluid_builder,
            $autoplay_state,
            $autoplay_value
        );
        let __fluid_builder = $crate::__fluid_timeline_builder_auto_loop!(
            __fluid_builder,
            $auto_loop_state,
            $auto_loop_value
        );
        let __fluid_builder = $crate::__fluid_timeline_builder_steps!(
            __fluid_builder,
            &__fluid_transition,
            $steps_value
        );
        let __fluid_builder = $crate::__fluid_timeline_builder_triggers!(
            __fluid_builder,
            $triggers_state,
            $triggers_value
        );
        __fluid_builder.install()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_builder_initial {
    ($builder:expr, unset, ()) => {
        $builder
    };
    ($builder:expr, set, ($initial:expr)) => {
        $builder.initial($initial)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_builder_autoplay {
    ($builder:expr, unset, ()) => {
        $builder
    };
    ($builder:expr, set, ($value:expr)) => {
        $builder.autoplay($value)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_builder_auto_loop {
    ($builder:expr, unset, ()) => {
        $builder
    };
    ($builder:expr, set, ($value:expr)) => {
        $builder.auto_loop($value)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_builder_triggers {
    ($builder:expr, unset, ()) => {
        $builder
    };
    ($builder:expr, set, []) => {
        $builder
    };
    (
        $builder:expr,
        set,
        [
            $(
                on($watch:expr) {
                    $($pattern:pat => $action:ident ( $($args:tt)* )),+ $(,)?
                }
            ),+ $(,)?
        ]
    ) => {
        $builder$(.on_change(
            move || $watch,
            move |__fluid_value, __fluid_timeline| match __fluid_value {
                $(
                    $pattern => {
                        $crate::__fluid_when_apply_timeline!(
                            __fluid_timeline,
                            $action($($args)*)
                        );
                    }
                ),+
            }
        ))+
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_builder_steps {
    ($builder:expr, $transition:expr, []) => {
        compile_error!("timeline! requires at least one step block")
    };
    (
        $builder:expr,
        $transition:expr,
        [
            $(
                { $($step:tt)* }
            ),+ $(,)?
        ]
    ) => {
        $builder$(.step($crate::__fluid_timeline_step!($transition, $($step)*)))+
    };
    ($builder:expr, $transition:expr, $($invalid:tt)+) => {
        compile_error!("timeline! steps must use `{ to: ..., ... }` blocks")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_step {
    ($transition:expr, $($tt:tt)*) => {
        $crate::__fluid_timeline_step_parse! {
            $transition,
            [to unset ()]
            [wait unset ()]
            [on_complete unset ()]
            $($tt)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_step_parse {
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
    ) => {
        $crate::__fluid_timeline_step_finish! {
            $transition,
            [to $to_state $to_value]
            [wait $wait_state $wait_value]
            [on_complete $on_complete_state $on_complete_value]
        }
    };
    (
        $transition:expr,
        [to unset ()]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        to: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_step_parse! {
            $transition,
            [to set ($next)]
            [wait $wait_state $wait_value]
            [on_complete $on_complete_state $on_complete_value]
            $($($rest)*)?
        }
    };
    (
        $transition:expr,
        [to set $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        to: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! step field `to` can only be specified once")
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait unset ()]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        wait_ms: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_step_parse! {
            $transition,
            [to $to_state $to_value]
            [wait set ($next)]
            [on_complete $on_complete_state $on_complete_value]
            $($($rest)*)?
        }
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait set $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        wait_ms: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! step field `wait_ms` can only be specified once")
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete unset ()]
        on_complete: $next:expr $(, $($rest:tt)*)?
    ) => {
        $crate::__fluid_timeline_step_parse! {
            $transition,
            [to $to_state $to_value]
            [wait $wait_state $wait_value]
            [on_complete set ($next)]
            $($($rest)*)?
        }
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete set $on_complete_value:tt]
        on_complete: $next:expr $(, $($rest:tt)*)?
    ) => {
        compile_error!("timeline! step field `on_complete` can only be specified once")
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        $field:ident : $($rest:tt)+
    ) => {
        compile_error!(concat!("unknown field in timeline! step: `", stringify!($field), "`"))
    };
    (
        $transition:expr,
        [to $to_state:ident $to_value:tt]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
        $($rest:tt)+
    ) => {
        compile_error!("invalid timeline! step syntax")
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fluid_timeline_step_finish {
    (
        $transition:expr,
        [to unset ()]
        [wait $wait_state:ident $wait_value:tt]
        [on_complete $on_complete_state:ident $on_complete_value:tt]
    ) => {
        compile_error!("timeline! step requires a `to:` field")
    };
    (
        $transition:expr,
        [to set ($to:expr)]
        [wait unset ()]
        [on_complete unset ()]
    ) => {{
        let __fluid_step = $crate::FluidStep::to($to).inherit_wait_from($transition);
        __fluid_step
    }};
    (
        $transition:expr,
        [to set ($to:expr)]
        [wait set ($wait:expr)]
        [on_complete unset ()]
    ) => {{
        let __fluid_step = $crate::FluidStep::to($to)
            .wait_ms($wait)
            .inherit_wait_from($transition);
        __fluid_step
    }};
    (
        $transition:expr,
        [to set ($to:expr)]
        [wait unset ()]
        [on_complete set ($on_complete:expr)]
    ) => {{
        let __fluid_step = $crate::FluidStep::to($to)
            .on_complete($on_complete)
            .inherit_wait_from($transition);
        __fluid_step
    }};
    (
        $transition:expr,
        [to set ($to:expr)]
        [wait set ($wait:expr)]
        [on_complete set ($on_complete:expr)]
    ) => {{
        let __fluid_step = $crate::FluidStep::to($to)
            .wait_ms($wait)
            .on_complete($on_complete)
            .inherit_wait_from($transition);
        __fluid_step
    }};
}
