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

## M3
- HURT (Bevy churn x2): the `2d` feature collection does NOT include bevy_ui —
  hotbar UI needed `features = ["2d", "ui"]`. And `TextFont::font_size` became
  the `FontSize` enum in 0.19 (`FontSize::Px(11.0)`).
- COST (Bevy): tile edits need explicit change propagation — `TileChanged`
  message → despawn chunk → manage_chunks respawns it (chained same frame).
  Whole-chunk rebuild (~1k sprites) at mining cadence is fine; C needed nothing.
- HELPED (Bevy): observer flow for pickup (`PickedUp` EntityEvent + `On<>`)
  worked first try; `resource_changed::<Inventory>` run condition means the
  hotbar UI system runs only on actual inventory changes.
- NICE: after the feature fix, the whole M3 batch (UI, messages, observer,
  gizmos) compiled and ran with zero further errors — the type system carried
  a big simultaneous change.

## M4
- HELPED (Bevy): swing state as a component (`Swing { t, hit }`) inserted/removed
  at runtime — "is the player swinging" is `Has<Swing>` in a query, and the
  sword visual is a child entity that despawns with the swing. Clean.
- COST (Bevy): projectile_update needs three queries over overlapping component
  sets (projectiles / enemies / player) — disjointness must be proven to the
  borrow checker via `Without<>` filters. C just indexes three arrays.
- NOTE: `Single<...>` fails the whole system if the entity is missing; for the
  optional case (player may have no Swing) it's `Option<Single<...>>`.
- Whole M4 batch compiled with only one dead-code warning. Zero runtime fixes.
