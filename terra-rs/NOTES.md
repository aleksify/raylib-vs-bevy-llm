# terra-rs friction diary

## M1
- HURT (Bevy churn): `WindowResolution` no longer `From<(f32, f32)>` — u32 physical
  pixels since 0.17. Exactly the API-churn class CLAUDE.md warns about; compile
  error was clear, fix trivial.
- DODGED: camera zoom done via `Transform::from_scale(0.5)` instead of the
  `Projection` component — projection API has churned repeatedly; transform scale
  is stable and equivalent for a 2D camera.
- HELPED (Bevy): `Single<...>` param + `FixedUpdate` at 60 Hz gave the fixed-step
  sim structure for free — no accumulator code, no input-eating bug possible (input
  latched in `Update` into a resource, consumed in `FixedUpdate`).
- Build: first compile of Bevy 0.19 ~6 min; incremental 0.35 s. Fine.
- Env: wgpu can't enumerate Metal inside the CLI sandbox ("Unable to find a GPU")
  — `cargo run` needs to run unsandboxed. raylib/OpenGL was unaffected.

## M2
- HELPED (Rust): worldgen ported from C near-verbatim; `wrapping_mul`/`wrapping_add`
  made the splitmix64 intent explicit where C wraps silently. Hash-matched C on
  first try, all seeds.
- COST (Bevy): rendering the world needed a real chunk system (ChunkMap resource,
  spawn/despawn hysteresis, child sprites) — ~80 lines where C needed zero, because
  tiles-as-entities must be bounded. Bevy's sprite batcher then eats 1k+ sprites
  per chunk without noticing.
- NOTE: `HashMap` comes from `bevy::platform::collections` now (std re-export moved
  again between versions).
- HELPED (Bevy): `despawn()` recursing through `ChildOf` children means chunk
  cleanup is one call.
