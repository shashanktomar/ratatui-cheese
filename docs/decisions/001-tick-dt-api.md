---
title: Use tick(dt Duration) for animation state
status: accepted
date: 2026-03-24
---

# ADR-001: Use `tick(dt: Duration)` for animation state

## Context

The spinner widget needs an API for advancing animation frames. We considered several designs for `SpinnerState::tick()`.

## Options Considered

### 1. `tick(frame_count: usize)` — caller passes frame count

```rust
state.tick(spinner.frames().len());
```

Rejected: exposes internal frame mechanics. The caller has to know the frame count and manage tick timing externally. Leaks implementation details.

### 2. `tick()` — zero arguments, state owns an `Instant`

```rust
state.tick(); // calls Instant::now() internally
```

Rejected: embeds a syscall inside a method that looks side-effect-free. Makes testing require `thread::sleep()` or a special `tick_elapsed()` escape hatch. Can't pause, fast-forward, or replay.

### 3. `tick(dt: Duration)` — caller passes elapsed time

```rust
let now = Instant::now();
state.tick(now - last_tick);
last_tick = now;
```

Accepted.

## Decision

Use `tick(dt: Duration)`. The state accumulates elapsed time internally and advances frames when the interval is reached.

## Rationale

This is the standard pattern across game and animation frameworks:

- **Bevy** (Rust) — systems receive `Time` resource with `time.delta()`
- **Unity** — `Update()` reads `Time.deltaTime`
- **Godot** — `_process(delta)` receives dt as parameter

The caller owns the clock, the component receives elapsed time. This enables:

- **Testing** — pass synthetic durations, no real time needed
- **Pause/resume** — stop passing dt to freeze animation
- **Fast-forward/slow-mo** — multiply dt
- **Deterministic replay** — record and replay dt sequence

## Tradeoff

The user writes 2-3 lines of boilerplate to compute dt. This is expected in any animation loop and not surprising to the target audience (Rust TUI developers).
