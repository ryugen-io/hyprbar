# Animation API in ks-core

We refactored the `Dish` trait in `ks-core` to support internal animations.

## Changes
- `Dish::render` signature changed to: `fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: Duration);`
- `BarRenderer` passes delta time (`dt`) to dishes.
- `ks-bin` (wash command) automatically adds `tachyonfx` dependency to washed plugins.
- Plugins can now hold `tachyonfx::Effect` in their struct and call `effect.process(dt, buf, area)` inside `render`.

## Dependencies
- `tachyonfx` version must match between host (ks-bin/ks-core) and plugins.
