# example_motion_controller_react

React + Motion equivalent of `example_motion_controller` for bundle-size comparison.

## Goals

- Mirror the current controller-first demos from `example_motion_controller`, including builder, macro, resolver, spring, timeline, and auto-size flows.
- Provide two Motion builds for direct size comparison:
  - `LazyMotion` + `domAnimation` from `motion/react`
  - `m.*` components from `motion/react-m`
  - `useAnimate` from `motion/react-mini`
- Compare build output against `example_motion_controller/dist`.

## Run

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

Build both variants:

```bash
npm run build:all
```

## Compare output sizes vs Leptos demo

```bash
npm run build:size
```

This prints per-file and total raw/gzip/brotli sizes for:

- `example_motion_controller_react/dist`
- `example_motion_controller_react/dist-mini`
- `example_motion_controller/dist`

## Optional regression pass

Use the same controller regression suite against this React dist:

```bash
cargo run -p playwright_regression_controller -- --dist-dir example_motion_controller_react/dist
cargo run -p playwright_regression_controller -- --dist-dir example_motion_controller_react/dist-mini
```
