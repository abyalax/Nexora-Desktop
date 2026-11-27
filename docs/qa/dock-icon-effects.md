# Dock icon effects and reflection (D-14/15)

## Purpose
Check selection plates, glow, magnification, reflections, and idle footprint.

## Prerequisites
- Windows 11; Dock enabled; several icons with contrasting artwork.

## Test Steps
1. Exercise each icon effect and change zoom, plate, glow color, and intensity.
2. Enable reflection and inspect its alignment while hovering and scrolling overflow.
3. Repeat at 100% and 150% display scale; observe idle CPU and memory.

## Expected Behavior
Effects follow settings without clipped icons or stale pixels; idle cost remains within the recorded baseline.

## Actual Behavior
Not run on Windows 11. Record measurements in `docs/bench.md` when available.

## Recovery
Disable effects and confirm normal rendering after restart.
