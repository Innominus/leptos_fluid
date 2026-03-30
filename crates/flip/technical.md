# leptos_fluid_flip technical.md

This document explains the internal runtime and data flow in `leptos_fluid_flip`.

## Scope and exported API

Public surface from `src/flip/mod.rs`:

- `Flip`: single-element FLIP animator (id lookup)
- `FlipGroup`: multi-element FLIP animator (selector lookup)
- `FlipOptions`: runtime options
- `Easing`: preset/custom easing selection
- `ScaleMode`: position-only vs position+scale animation

`FlipValues` is public for diagnostics/introspection, but is primarily an internal measurement carrier.

## Runtime architecture

The crate has two execution layers:

1. FLIP orchestration in `src/flip/mod.rs` plus `single.rs` / `group.rs`
2. scale/border-radius correction loops in `src/flip/corrections.rs`

Both use `leptos_fluid_web` helpers for DOM style reads/writes and WAAPI invocation.

## Single element path (`Flip`)

### Lifecycle

`Flip::animate` wraps user mutation closure execution in a FLIP pipeline:

1. measure **first** rect (`from`) before mutation
2. run user mutation closure (the layout-changing update)
3. schedule `request_animation_frame`
4. measure **last** rect (`to`) on the next frame
5. compute inverse transform (`dx`, `dy`, optional scale)
6. animate inverse transform to identity via WAAPI
7. restore original inline styles on completion

### Why the RAF hop exists

Measurements after the mutation are intentionally deferred by one frame so layout and style recalculation have committed. Measuring in the same turn would often capture stale geometry.

## Group path (`FlipGroup`)

`FlipGroup` performs the same FLIP idea over a selector set.

### Identity matching

Before/after snapshots are matched by stable key in this order:

1. `data-flip-id`
2. `id`
3. fallback synthetic key from selector index

The fallback keeps animation functional, but stable explicit keys are strongly preferred for reorders.

### Stagger

`FlipOptions::stagger` is applied per matched item as:

`effective_delay = base_delay + stagger * index`

where `index` is the post-mutation matched item order.

## Interruption handling

The crate handles mid-flight re-entry (rapid toggles, repeated reorder clicks):

1. active WAAPI animations are canceled
2. current computed transform is written inline first
3. inline transform/origin/will-change/transition values are carried forward
4. next FLIP run starts from the frozen visual frame

Without this freeze-and-carry step, interrupted animations snap back before continuing.

## Scale and border-radius correction (`corrections.rs`)

When `ScaleMode::PositionAndScale` is enabled, parent scaling can visually distort descendants and rounded corners. The correction layer compensates during the animation window.

### Descendant scale correction

- optional selector: `FlipOptions::scale_correction_selector`
- each matched descendant gets inverse scale transform every frame
- translation offset is applied so corrected elements stay visually anchored

This runs in an RAF loop until FLIP cleanup signals stop.

### Border radius correction

For scaled elements, computed corner radii are sampled once, then rewritten each frame with inverse-scale compensation.

This preserves perceived corner curvature while the parent scale interpolates.

## WAAPI contract and fallback behavior

Animation is attempted through `Element.animate(...)` with:

- keyframes: `transform` from inverse transform to identity
- options: duration, delay, easing, `fill = "backwards"`

If WAAPI is unavailable or the call fails:

- no panic is raised
- completion is scheduled on the next frame
- cleanup still runs so inline state is restored

## Cleanup guarantees

On finish (or fallback completion), runtime always:

1. stops correction loops
2. restores saved inline styles on animated element and corrected descendants
3. updates `is_animating` state

For groups, `is_animating` stays true until the last active member callback decrements the shared counter to zero.

## Important internals to preserve when changing behavior

- `has_flip_delta_with_size` epsilon gate prevents no-op animations and style churn.
- interruption freeze (`apply_computed_transform`) must run before cancel.
- identity matching logic in groups must stay deterministic.
- correction loops must stop on every exit path to avoid orphaned RAF recursion.

## Suggested contributor validation checklist

After changing FLIP behavior:

1. Single element: move, resize, and move+resize transitions.
2. Group reorder: with and without `data-flip-id`.
3. Mid-flight interruption: trigger another animation before first finish.
4. Scale correction: text/icon descendants remain crisp while parent scales.
5. Border radius: corners do not visibly squash/stretch during scale animation.
