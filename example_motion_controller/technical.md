# leptos_fluid_motion_controller_example technical.md

This document explains the structure and intent of the `example_motion_controller` crate.

## Purpose

`example_motion_controller` is a controller-first playground for `leptos_fluid::motion::AnimationController`.

It intentionally avoids `FluidElement` wrappers so contributors can evaluate the element-agnostic API directly.

## Entrypoints

- `src/main.rs`: mounts `App` and enables panic hook in debug
- `src/app.rs`: all demo sections and helper style builders

## Demo sections

- `ToggleCardExample`: declarative `bind` usage with regular state toggles
- `TabsUnderlineExample`: measured tab underline animation retargeted through a single controller
- `PointerStateExample`: app-managed hover/press/base states forwarded to `animate_with`
- `QueueLatestExample`: detached target updates to validate queue-latest semantics

## Design constraints

- no `FluidDiv`, `FluidSpan`, or `FluidElement`
- all animation commands route through `AnimationController`
- all targets are plain HTML elements attached through `NodeRef`

## Why this crate exists

`example_motion` remains broad and behavior-heavy (`motion` + `flip`). This crate is intentionally narrow so controller API iteration can be validated in isolation without the component abstraction layer.

## Regression hooks

`src/app.rs` includes stable `data-testid` attributes for Playwright checks in `tools/playwright_regression_controller`.

Current regression focus:

- bind-driven style interpolation
- tab underline retargeting and settle alignment
- pointer enter/down/up/leave state choreography
- queue-latest replay when target is detached
