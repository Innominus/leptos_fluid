# leptos_fluid technical.md

This document explains how the top-level `leptos_fluid` crate is structured and why it exists in front of the subcrates.

## Purpose

`leptos_fluid` is a feature-gated facade crate over these implementation crates:

- `leptos_fluid_motion`
- `leptos_fluid_flip`
- `leptos_fluid_view_transitions`
- `leptos_fluid_web` (internal helper, not re-exported by the facade)

The facade gives users one dependency and one feature switch surface while letting each subsystem evolve independently.

## Public API surface

File: `src/lib.rs`

The facade does not define runtime code. It only:

1. conditionally compiles re-export modules by feature
2. exposes backward-compatible module paths for older consumers

Current re-export modules:

- `leptos_fluid::motion` (feature `motion`)
- `leptos_fluid::flip` (feature `flip`)
- `leptos_fluid::view_transitions` (feature `view-transitions`)

Back-compat shim:

- `leptos_fluid::animators::flip`
- `leptos_fluid::animators::view_transitions`

No compatibility shim exists for motion because earlier versions used the animator routes around flip/view transitions.

## Feature model

Defined in root `Cargo.toml`:

- `default = []`
- `motion`
- `flip`
- `view-transitions`
- `full = ["motion", "flip", "view-transitions"]`

The empty default set is intentional:

- keeps transitive size minimal
- avoids pulling router/runtime deps users do not need
- allows crate consumers to make deliberate runtime tradeoffs

## Workspace packaging strategy

The workspace is configured so published crates are:

- `leptos_fluid`
- `leptos_fluid_motion`
- `leptos_fluid_flip`
- `leptos_fluid_view_transitions`
- `leptos_fluid_web`

Examples are in-workspace for easy local iteration but marked `publish = false`.

## Dependency and version strategy

Workspace-level shared versions are centralized in `[workspace.dependencies]` to prevent drift. Subcrates inherit via `workspace = true`.

Important operational implication:

- when upgrading Leptos/web-sys/js-sys, all crates should be tested together because motion/flip/view_transitions depend on shared DOM/WAAPI assumptions.

## docs.rs strategy

Facade crate sets:

- `[package.metadata.docs.rs] all-features = true`

This ensures docs.rs builds all feature-gated modules so users see full API docs in one place.

## Where to work for behavior changes

If you need to modify behavior, work in subcrates:

- element motion behavior: `crates/motion/src/*`
- FLIP behavior: `crates/flip/src/*`
- router outlet transition behavior: `crates/view_transitions/src/*`
- JS/DOM helper layer: `crates/web/src/lib.rs`

Changes in those crates are automatically reflected through facade re-exports.

## Backward compatibility notes

When moving or renaming public modules:

1. keep the old path available behind a shim where feasible
2. document migration in root `README.md`
3. avoid breaking both facade and direct-subcrate users at once

## Testing and verification workflow

Typical sanity pass after cross-crate changes:

```bash
cargo fmt --all
cargo check --workspace
cargo package --allow-dirty --no-verify -p leptos_fluid --list
cargo package --allow-dirty --no-verify -p leptos_fluid_motion --list
cargo package --allow-dirty --no-verify -p leptos_fluid_flip --list
cargo package --allow-dirty --no-verify -p leptos_fluid_view_transitions --list
cargo package --allow-dirty --no-verify -p leptos_fluid_web --list
```

## Common pitfalls for contributors

- Adding APIs only to subcrates but forgetting to re-export through facade modules.
- Adding a new top-level feature but not wiring docs.rs or README feature docs.
- Breaking compatibility paths in `animators::*` unexpectedly.
- Introducing dependency version skew between subcrates.
