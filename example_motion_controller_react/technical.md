# example_motion_controller_react technical.md

This project mirrors `example_motion_controller` using React + Motion in tree-shaken Vite builds.

## Purpose

- Provide a behavior-matched implementation for size/performance comparison.
- Keep UI and interaction semantics close to the Rust controller demo.
- Measure output bundle weight against `example_motion_controller/dist`.

## Motion bundle strategy

- `src/App.tsx`: `LazyMotion` + `domAnimation` and `m.*` components (`motion/react-m`).
- `src/AppMini.tsx`: imperative `useAnimate` from `motion/react-mini`.
- `src/main.tsx` builds the `m.*` version to `dist`.
- `src/main-mini.tsx` builds the mini version to `dist-mini`.
- Avoid importing `motion` directly so tree shaking remains effective.

## Demo sections

- builder-style card state transitions
- macro-style state machine transitions
- resolver-driven target switching between live cards
- spring retarget and timeline examples
- auto-size examples for height and width

## Verification

Build `m.*` variant:

```bash
npm run build
```

Build mini variant:

```bash
npm run build:mini
```

Build both and compare outputs:

```bash
npm run build:size
```

Run existing regression checks against this dist:

```bash
cargo run -p playwright_regression_controller -- --dist-dir example_motion_controller_react/dist
cargo run -p playwright_regression_controller -- --dist-dir example_motion_controller_react/dist-mini
```
