# leptos_fluid_motion_controller_example technical.md

This document explains the structure and intent of the `example_motion_controller` crate.

## Purpose

`example_motion_controller` is a controller-first playground for `leptos_fluid_motion`.

It exercises the plain-element controller, builder, macro, resolver, timeline, and auto-size APIs directly.

## Entrypoints

- `src/main.rs`: mounts `App` and enables the panic hook in debug builds
- `src/app.rs`: page shell and demo ordering
- `src/examples/mod.rs`: example exports

## Demo sections

`App` renders these examples:

- `BuilderCardExample`: typed `AnimationController::builder()` install flow on a plain `div`
- `MacroStateExample`: `controller!` + `when!` state-machine style animation wiring
- `ResolverDeckExample`: dynamic `resolver:` target switching between multiple live nodes
- `SpringRetargetExample`: spring-based retargeting on controller-managed elements
- `SpringTimelineExample`: timeline sequencing paired with spring-tuned motion
- `TimelineBuilderExample`: typed builder API for timelines
- `TimelineMacroExample`: declarative `timeline!` setup
- `AutoSizeExample`: `bind_auto_height` and `bind_auto_width` helpers on plain elements

## Design constraints

- animation commands route through `AnimationController` or controller-backed timeline helpers
- targets are plain HTML elements reached through either stable `NodeRef`s or dynamic resolver closures

## Why this crate exists

`example_motion` is the broad visual playground. This crate is narrower so controller-surface changes can be validated directly.

## Regression hooks

The examples expose stable `data-testid` attributes for Playwright checks in `tools/playwright_regression_controller`.

Current regression focus includes:

- typed builder installation and state-driven retargeting
- macro-driven state transitions
- resolver-based target switching across live nodes
- timeline play/pause/resume behavior
- auto-height and auto-width behavior under repeated content changes
