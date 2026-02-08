# leptos_fluid_view_transitions technical.md

This document explains how `leptos_fluid_view_transitions` coordinates nested route outlet animations.

## Scope and exported API

Public exports from `src/lib.rs`:

- `FluidManager`: shared transition coordinator stored in context
- `FluidRoutes`: thin wrapper over `leptos_router::Routes`
- `FluidOutlet`: replacement for `Outlet` with intro/outro layer rendering

## High-level model

The crate uses a dual-layer outlet strategy:

1. current outlet DOM is cloned into an **outro** layer
2. next route content renders in the **intro** layer
3. CSS classes drive both animations
4. after both `animationend` events, outro clone is removed

This allows nested route transitions without changing route component internals.

## `FluidManager` responsibilities (`src/fluid_manager.rs`)

`FluidManager` owns cross-outlet state:

- route-to-node mappings (`outlet_nodes`)
- cached outlet route hierarchy (`outlet_route_cache`)
- current pathname signal memo (`location`)
- generated route patterns from `FluidRoutes` (`generated_routes`)
- transition direction (`navigate_backwards`)
- compatibility fallback switch (`skip_transition`)

### Transition entry

`transition()` does this:

1. matches current location to best outlet route
2. optionally skips transition (compatibility path)
3. computes backward/forward direction
4. snapshots scroll positions of `[data-scrollable]` descendants
5. clones intro DOM into outro node
6. restores captured scroll offsets into cloned subtree
7. marks outlet transition flag and updates current location
8. truncates deeper route cache entries when parent route changes

## Route pattern capture (`src/fluid_route.rs`)

`FluidRoutes` wraps `Routes` and additionally captures generated route segments.

Dynamic, optional, and wildcard segments are normalized to `":"` during storage so route-order comparisons stay meaningful for direction inference.

This is used by `FluidManager::set_reversal()` to infer backward navigation when moving to a route with a lower generated index.

## Outlet runtime (`src/fluid_outlet.rs`)

Each `FluidOutlet` instance registers two nodes with manager:

- intro node (live route content)
- outro node (cloned previous content)

`FluidOutlet` then:

1. keeps route registration updated when the matched route changes
2. computes class assignment based on direction
3. applies `no-animations` to the cloned outro subtree
4. listens for `animationend` on intro and outro wrappers
5. clears clone and resets flags after both wrappers report completion

### Why wrapper-level event filtering exists

Animation events bubble. The handler only accepts events whose `target` is the wrapper node itself to avoid premature cleanup triggered by child animations.

## Scroll restoration (`src/utils.rs`)

Utility functions preserve scroll positions for annotated descendants:

- `get_scroll_pos_of_attr_children`
- `set_scroll_pos_to_children_with_attr`

Restore is deferred via `request_animation_frame` so it runs after clone insertion and layout creation.

## Browser compatibility fallback

Manager installs a `popstate` listener for known incompatible engines (iOS/Safari heuristics). On the next back navigation, `skip_transition` is toggled once to bypass a transition that would otherwise break.

The skip is one-shot: `check_skip_transition()` resets the flag after consumption.

## CSS contract

The crate does not define transition animation keyframes. Consumers must provide real CSS animations for classes passed to `FluidOutlet`.

Important behavior coupling:

- cleanup is gated by `animationend`
- if classes do not run animations, cleanup state will not progress

`NO_ANIMATION_CSS` disables nested child transitions/animations in the cloned outro layer to prevent doubled animations.

## Invariants contributors should preserve

- `FluidManager` must be context-provided exactly once per router tree.
- outlet registration and disposal must stay symmetrical.
- cleanup must wait for both intro and outro wrapper completions.
- hierarchy cache truncation must happen when parent route changes.
- backward direction logic depends on deterministic generated route order.

## Suggested contributor validation checklist

1. Nested routes: top, middle, and deep outlets all transition.
2. Back navigation: intro/outro classes reverse correctly.
3. Scroll restoration: `[data-scrollable]` containers keep offsets through transitions.
4. Child animation noise: nested component animations do not trigger early cleanup.
5. Safari/iOS back button path: one transition skip prevents broken state.
