# leptos_fluid_motion_example technical.md

This document explains the structure and intent of the `example_motion` crate.

## Purpose

`example_motion` is the broad visual playground for the motion and FLIP crates.

It is used to validate:

- controller builders and `bind_interaction_node_ref` hover/tap behavior on plain elements
- `FluidStyle` composition and transition presets
- auto-size helpers driven by `ResizeObserver`
- timeline sequencing
- spring retargeting via `use_spring`
- single-element and group FLIP behavior
- interruption behavior under repeated user input
- rough runtime cost signals through the perf panel

## Entrypoints and composition

- `src/main.rs`: mounts the app and enables the panic hook in debug builds
- `src/app.rs`: renders the demo sections in a fixed order
- `src/components/mod.rs`: section exports

`App` renders these sections in order:

1. `HeroSection`
2. `WrapperGallerySection`
3. `StyleLabSection`
4. `AutoLayoutSection`
5. `TimelineStudioSection`
6. `SpringShowcaseSection`
7. `SpringFollowSection`
8. `FlipCardSection`
9. `FlipBoardSection`
10. `PerfSection`
11. `FooterSection`

## Section map and what each validates

- `hero.rs` (`HeroSection`): landing copy and visual framing for the playground.
- `wrappers.rs` (`WrapperGallerySection`): controller builders and `bind_interaction_node_ref` on plain div/span/button elements.
- `style_lab.rs` (`StyleLabSection`): `FluidStyle`, transform composition, and arbitrary CSS property animation on a plain article element.
- `auto_layout.rs` (`AutoLayoutSection`): auto-height and auto-width helpers bound to live content changes.
- `timeline.rs` (`TimelineStudioSection`): `FluidTimeline` sequencing, loop control, and step playback.
- `spring_showcase.rs` (`SpringShowcaseSection`): spring-driven values and transition tuning.
- `spring_follow.rs` (`SpringFollowSection`): continuously retargeted spring motion driven by pointer movement.
- `flip_card.rs` (`FlipCardSection`): single-element FLIP movement and resize behavior.
- `flip_board.rs` (`FlipBoardSection`): group FLIP reorder behavior with stable identities.
- `perf.rs` (`PerfSection`): lightweight frame-time sampling while animating many elements.
- `footer.rs` (`FooterSection`): end marker for manual walkthroughs.

## Performance harness (`perf.rs`)

The perf section runs an RAF loop and records frame durations in a bounded window.

Reported metrics include:

- average frame time
- p95 frame time
- derived FPS

It is a practical smoke test, not a formal benchmark.

## CSS dependency

Visual behavior depends on the example stylesheet. Rust components assume the matching layout and class contracts exist.

When refactoring example markup, update the CSS in lockstep.

## Contributor workflow

Use this crate for manual validation after changing `motion` or `flip` internals.

Suggested pass:

1. Exercise controller-driven cards and verify hover/tap transitions stay smooth.
2. Toggle the auto-size panels repeatedly and confirm shells track content without snapping.
3. Start, pause, resume, and restart timeline sequences.
4. Move the pointer quickly through the spring-follow panel.
5. Trigger FLIP card and board updates repeatedly before prior animations finish.
6. Run the perf panel at different loads and compare FPS/p95 before and after changes.
