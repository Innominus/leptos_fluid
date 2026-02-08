# leptos_fluid_motion_example technical.md

This document explains the structure and intent of the `example_motion` crate.

## Purpose

`example_motion` is a behavior-heavy playground for the motion and FLIP subsystems.

It is used to validate:

- `FluidStyle` composition and transitions
- spring retargeting via `use_spring`
- `FluidTimeline` sequencing
- `Flip` single-element transitions
- `FlipGroup` reorder transitions
- interruption behavior under repeated user input

## Entrypoints and composition

- `src/main.rs`: mounts app and enables panic hook in debug
- `src/app.rs`: page composition shell
- `src/components/mod.rs`: section exports

`App` renders sections in a fixed order so contributors can run through a repeatable manual QA flow.

## Section map and what each validates

- `hero.rs` (`HeroSection`): state-driven `FluidStyle` and hover/tap variants.
- `cards.rs` (`CardsSection`): mixed transition presets and `style!` macro usage.
- `tabs.rs` (`TabsSection`): interruptible spring underline with geometry measurement + retargeting.
- `timeline.rs` (`TimelineSection`): `FluidTimeline` setup, node attachment, loop/pause/resume/reset.
- `flip.rs` (`FlipSection`, `FlipHeroSection`, `FlipGroupSection`): FLIP move/resize, shared-element style modal transition, and keyed group reorder with scale correction.
- `island.rs` (`IslandSection`): multi-spring shape morph (width/height/radius/content/glow).
- `follow.rs` (`SpringFollowSection`): pointer-follow spring target updates.
- `staggered.rs` (`StaggeredChipsSection`): staggered delayed transitions via per-item `Transition` delay.
- `perf.rs` (`PerfSection`): lightweight frame-time sampling loop while animating many dots.
- `footer.rs`: static end marker.

## Shared state strategy

Top-level app state is intentionally small:

- `pulse: RwSignal<bool>`
- `card_focus: RwSignal<bool>`

Most sections keep local state to isolate behavior and simplify regression triage.

## Performance harness (`perf.rs`)

The perf section runs an RAF loop and records frame durations in a bounded `VecDeque` window.

Computed metrics:

- average frame time
- p95 frame time
- derived FPS

Why it exists:

- gives a quick smoke signal when animation/runtime changes increase frame cost
- is not a strict benchmark, but a practical regression indicator

## Timeline integration pattern

`TimelineSection` demonstrates the recommended timeline wiring:

1. create `FluidTimeline` from a base `FluidStyle`
2. optionally `attach_node_ref` for pause/resume WAAPI control
3. set steps with `wait_for(&transition)`
4. bind `timeline.signal()` to a `FluidDiv` `animate` prop

This section is the reference implementation for timeline contributor docs.

## FLIP integration pattern

`flip.rs` intentionally demonstrates three distinct FLIP use cases:

1. fixed-id element move/resize (`FlipSection`)
2. shared element transition in/out modal layout (`FlipHeroSection`)
3. group reorder+density changes with stable identities (`FlipGroupSection`)

Group demo uses `data-flip-id` and `scale_correction_selector` to exercise the most complex FLIP path.

## CSS dependency

Visual behavior depends on `style.css` for layout and class semantics. Rust code assumes these class contracts exist (for lane alignment, grid sizing, modal states, etc.).

When refactoring component markup, update CSS in lockstep.

## Contributor workflow

Use this crate for manual validation after changing `motion` or `flip` internals.

Suggested pass:

1. Rapid-click tab buttons while underline is moving.
2. Toggle timeline play/pause/resume/reset repeatedly.
3. Trigger FLIP move/resize repeatedly before prior animation ends.
4. Reorder FLIP group with rotate/reverse/shuffle under dense and relaxed spacing.
5. Move pointer quickly in spring-follow and dynamic-island sections.
6. Run perf section with increasing dot count and compare FPS/p95 before and after changes.
