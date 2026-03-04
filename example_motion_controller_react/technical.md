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

- `ToggleCardExample`: declarative state-driven card target.
- `TabsUnderlineExample`: measured underline retargeting between tabs.
- `PointerStateExample`: app-managed hover/press/base priority.
- `QueueLatestExample`: detached-state updates replay on remount.

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
