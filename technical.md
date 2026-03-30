# leptos_fluid technical.md

This document explains how the top-level `leptos_fluid` crate is structured and how its feature-gated facade maps onto the implementation crates.

## Purpose

`leptos_fluid` is a facade crate over these implementation crates:

- `leptos_fluid_motion`
- `leptos_fluid_flip`
- `leptos_fluid_view_transitions`
- `leptos_fluid_web` (internal helper crate, not re-exported by the facade)

The facade exists so application code can depend on one crate while still choosing a narrow feature surface.

## Public API surface

File: `src/lib.rs`

The facade itself does not implement runtime behavior. It only:

1. conditionally re-exports subcrate APIs behind top-level modules
2. keeps compatibility shims for older `animators::*` paths

Current re-export modules:

- `leptos_fluid::flip` when feature `flip` is enabled
- `leptos_fluid::view_transitions` when feature `view-transitions` is enabled
- `leptos_fluid::motion` when feature `motion-core` is enabled

Important detail: `leptos_fluid::motion` is not tied only to the umbrella feature named `motion`.
Any umbrella feature that enables `motion-core` makes the module available, including:

- `motion-core`
- `motion`
- `motion-spring`
- `motion-controller`
- `motion-auto-size`
- `motion-timeline`
- `motion-components`
- `motion-wrappers`
- `motion-builders`
- `motion-macros`
- `motion-full`
- `full`

Back-compat shim modules:

- `leptos_fluid::animators::flip`
- `leptos_fluid::animators::view_transitions`

No motion compatibility shim exists because the older compatibility surface centered on flip/view-transition animator paths.

## Feature model

Defined in root `Cargo.toml`:

- `default = []`
- `flip`
- `view-transitions`
- `motion-core`
- `motion-spring`
- `motion-controller`
- `motion-auto-size`
- `motion-timeline`
- `motion-components`
- `motion-wrappers`
- `motion-builders`
- `motion-macros`
- `motion-full`
- `motion`
- `full`

Feature intent:

- `motion-core`: base motion types like `FluidStyle`, `Transition`, `Easing`, and `FluidSignal`
- `motion`: common element-motion surface, equivalent to controller + components + wrappers
- `motion-full`: full motion feature surface without flip/view-transitions
- `full`: enables `flip`, `view-transitions`, and `motion-full`

The empty default set is intentional:

- keeps transitive dependencies minimal
- avoids pulling router/runtime code into apps that only need one subsystem
- lets consumers make explicit wasm-size tradeoffs

## Workspace packaging strategy

Published crates in this workspace are:

- `leptos_fluid`
- `leptos_fluid_motion`
- `leptos_fluid_flip`
- `leptos_fluid_view_transitions`
- `leptos_fluid_web`

Examples and internal tools stay in the workspace for local iteration but are marked `publish = false` where appropriate.

## Dependency and version strategy

Shared versions live in `[workspace.dependencies]` so Leptos, `web-sys`, and `js-sys` stay aligned across subcrates.

Operational implication:

- when upgrading Leptos or browser-facing dependencies, test motion, flip, and view-transition crates together because they share DOM and WAAPI assumptions

## docs.rs strategy

The facade crate sets:

- `[package.metadata.docs.rs] all-features = true`

That keeps docs.rs aligned with the full re-export surface instead of hiding modules behind feature defaults.

## Where to work for behavior changes

If you need to modify behavior, work in the subcrates:

- motion runtime and components: `crates/motion/src/*`
- FLIP runtime: `crates/flip/src/*`
- router outlet transitions: `crates/view_transitions/src/*`
- shared DOM/WAAPI helpers: `crates/web/src/lib.rs`

Changes there flow through the facade automatically.

## Backward compatibility notes

When moving or renaming public APIs:

1. preserve the old path behind a shim when practical
2. document the migration in the relevant README
3. keep both umbrella-crate users and direct-subcrate users in mind

## Testing and verification workflow

Typical sanity pass after cross-crate or feature-surface changes:

```bash
cargo fmt --all
cargo check --workspace --all-features
cargo package --allow-dirty --no-verify -p leptos_fluid --list
cargo package --allow-dirty --no-verify -p leptos_fluid_motion --list
cargo package --allow-dirty --no-verify -p leptos_fluid_flip --list
cargo package --allow-dirty --no-verify -p leptos_fluid_view_transitions --list
cargo package --allow-dirty --no-verify -p leptos_fluid_web --list
```

## Common pitfalls for contributors

- Adding APIs only to subcrates but forgetting the facade re-export or facade docs.
- Documenting only the coarse `motion` feature and forgetting the forwarded `motion-*` features.
- Breaking `animators::*` compatibility paths unexpectedly.
- Letting dependency versions drift between subcrates.
