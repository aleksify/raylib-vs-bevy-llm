# Terra — a lightweight Terraria clone, twice

Two implementations of the exact same 2D sandbox game, built to compare C vs Rust for
game development:

| Directory  | Language | Library / Engine | Version |
|------------|----------|------------------|---------|
| `terra-c/` | C (C99)  | raylib           | **6.0** |
| `terra-rs/`| Rust     | Bevy             | **0.19**|

The two versions must stay **feature-identical**. Every gameplay feature is specified
once (below) and implemented twice. When adding a feature, implement it in both, or
leave a `TODO(parity)` note in the lagging one.

## ⚠️ Version note (READ THIS FIRST)

Checked 2026-07-06:

- **raylib latest stable is 6.0** (released 2026-04-23). Biggest additions since 5.5:
  software renderer backend (rlsw), Win32 native backend, redesigned fullscreen/High-DPI,
  file-system API module, text-management API (~30 new functions). Breaking changes vs
  5.5 that could bite us: `SetSoundPan()`/`SetMusicPan()` now range `[-1.0..1.0]`,
  `DrawCircleGradient()` takes a `Vector2` center, shader load failure returns 0 instead
  of a fallback. The core 2D API (`DrawTextureRec`, `Camera2D`, input, audio) is
  unchanged and stable.
- **Bevy latest stable is 0.19** (released 2026-06-19). Bevy's API churns hard between
  minor versions — **do not use pre-0.15 patterns from memory or old tutorials.** Dead
  patterns you must NOT write: `SpriteBundle`/`Camera2dBundle` (bundles are gone —
  required components since 0.15), `EventReader`/`EventWriter`/`add_event` for buffered
  events (renamed `MessageReader`/`MessageWriter`/`add_message` in 0.17),
  `Trigger<E>` in observers (renamed `On<E>` in 0.17). New in 0.19 and worth using:
  **BSN scenes** (`bsn!` macro + `commands.spawn_scene()`), resources stored as
  singleton-entity components, observer run conditions
  (`app.add_observer(sys.run_if(...))`), delayed commands
  (`commands.delayed().secs(1.0)`), `DiagnosticsOverlayPlugin`.
- **When any Bevy code fails to compile with a "not found" / trait error, assume an API
  rename first.** Check the 0.18→0.19 migration guide
  (https://bevy.org/learn/migration-guides/) and docs.rs for `bevy 0.19` before
  rewriting logic. Never trust example code from before 2025 without verifying.

---

# Game design

Scope: a *lightweight* Terraria. Small fixed-size world, a handful of blocks, one melee
weapon, one ranged weapon, three enemy types, a hotbar. No crafting, no bosses, no
lighting engine, no liquids, no NPCs, no save files (v1).

## Shared constants (identical in both versions)

```
TILE_SIZE        = 16 px            (Kenney pixel assets are 16x16)
WORLD_W          = 1024 tiles
WORLD_H          = 256  tiles
CHUNK_SIZE       = 32   tiles       (render/collision chunking)
FIXED_DT         = 1/60 s           (fixed-timestep simulation)
GRAVITY          = 900  px/s²
PLAYER_SPEED     = 140  px/s
PLAYER_JUMP_VEL  = -320 px/s
PLAYER_REACH     = 5    tiles       (max distance for break/place)
PLAYER_HP        = 100
WINDOW           = 1280x720, camera zoom 2.0 (so ~40x22 tiles visible)
```

## Tiles

Stored as a flat `u8`/`uint8_t` array of size `WORLD_W * WORLD_H`, row-major,
index `y * WORLD_W + x`. `(0,0)` is top-left; y grows downward.

| ID | Tile   | Solid | Drops itself | Notes                        |
|----|--------|-------|--------------|------------------------------|
| 0  | Air    | no    | —            |                              |
| 1  | Dirt   | yes   | yes          |                              |
| 2  | Grass  | yes   | drops dirt   | dirt with grass top          |
| 3  | Stone  | yes   | yes          |                              |
| 4  | Wood   | yes   | yes          | tree trunks; placeable plank |
| 5  | Leaves | no    | no           | decorative, walk-through     |
| 6  | Ore    | yes   | yes          | 2-tile+ veins in stone       |

## Procedural generation (identical algorithm + seed → identical world)

Both versions implement the same generator so worlds are comparable:

1. **Heightmap**: 1D fractal value noise (fBm, 4 octaves, lacunarity 2.0, gain 0.5),
   surface height per column ∈ `[WORLD_H*0.25 .. WORLD_H*0.45]`. Implement value noise
   by hand (~40 lines) in both languages — do NOT pull a noise library on either side;
   part of the comparison is writing the same math in both.
2. **Strata**: top solid tile = Grass, next 6–10 tiles = Dirt, everything below = Stone.
3. **Caves**: cellular automata on the stone layer — seed 45% walls from a
   `splitmix64`-style seeded PRNG (same PRNG algorithm both sides, NOT `rand()`/
   `rand::random` — determinism must match across languages), 5 smoothing passes with
   the 4/5 rule, carve resulting open cells to Air. Skip the top 10 tiles below dirt so
   caves don't breach the surface everywhere.
4. **Ore**: ~200 random walks (drunkard's walk, length 4–10) through Stone, converting
   to Ore.
5. **Trees**: on ~10% of grass columns (min 4 columns apart): 4–7 Wood trunk tiles up,
   3x3 Leaves blob on top.

## Player

- AABB 12x22 px (slightly smaller than 1x2 tiles to avoid corner snags).
- Left/right move, jump only when grounded. `A`/`D` or arrows + `Space`.
- **Tile collision, both versions the same way**: integrate velocity per axis, move X
  then resolve, move Y then resolve (swept per-axis AABB vs the solid-tile grid — check
  only the 2–4 tiles overlapping each leading edge). No physics engine on either side.
- Takes contact damage from enemies (10 HP, 0.5 s invulnerability + knockback).
  Death respawns at world spawn (surface at `WORLD_W/2`), restores HP.

## Mining & placing

- Mouse aim. Left-click: damage the targeted tile (tiles have hardness: dirt/grass 2
  hits, wood 3, stone/ore 4; hit cooldown 0.25 s while held). Broken tile spawns a
  **drop entity** (physics-y bouncing item that homes to player within 2 tiles, picked
  up on contact → inventory).
- Right-click: place selected hotbar block if target is Air, within reach, adjacent to
  a solid tile, and not overlapping player/enemies.
- Targeted tile gets a highlight outline; crack overlay by damage stage (optional v1).

## Inventory / hotbar

- 8 slots, stack max 999. Keys `1..8` select, mouse wheel cycles.
- Slot renders item icon + count. Slot 1 starts with Sword, slot 2 with Bow.
- No drag-drop inventory screen in v1 — hotbar only.

## Combat

- **Sword** (melee): on click, 0.3 s swing; damage applied via a 24x24 px hitbox in
  front of the player (facing = last horizontal input or mouse side). 20 dmg,
  knockback. Visual: rotate the sword sprite ~120° over the swing around the hand
  anchor point.
- **Bow** (ranged): fires arrow projectile toward cursor, speed 400 px/s, gravity-
  affected (0.5x gravity), 15 dmg, despawns on tile hit or 3 s lifetime. 0.5 s cooldown.
- **Projectiles** are generic: position, velocity, damage, faction (player/enemy),
  gravity factor, lifetime. Enemy variants reuse the same struct/component.

## Enemies

Spawn: max 8 alive; every 3 s try to spawn on a surface tile 20–40 tiles from the
player, off-screen. Despawn beyond 80 tiles. On death: flash + poof, drop nothing (v1).

| Enemy  | HP | Behavior                                                             |
|--------|----|----------------------------------------------------------------------|
| Slime  | 30 | Hops toward player: every 1.5 s, jump with vx toward player. Contact damage 10. |
| Zombie | 50 | Walks toward player, jumps when blocked by a 1-tile wall. Contact 15. |
| Bee    | 20 | Flies (no gravity), sine-wave drift toward player, shoots a slow projectile every 2 s (8 dmg). |

All enemies use the same tile-collision routine as the player (Bee skips gravity).

## Art & audio — Kenney (kenney.nl, CC0)

Download into a shared top-level `assets/` directory; both versions load from it
(relative path `../assets/` from each project, or copied at build).

- **Tiles + player + enemies**: *Pixel Platformer* pack (16x16 tilemap + characters).
- **Weapons/items/arrow/extra monsters**: *Tiny Dungeon* pack (16x16).
- **UI (hotbar frames, hearts)**: *Pixel UI Pack* or slices from Pixel Platformer.
- **SFX**: *Kenney Digital Audio* / *Impact Sounds* (dig, place, swing, hurt, shoot).

Both versions must render pixel-perfect: nearest-neighbor filtering, integer-ish camera
(round camera target to whole pixels at zoom 2 to avoid seams/shimmer). Prefer drawing
from the packs' atlas sheets with source rects over hundreds of individual PNGs; keep a
single shared `assets/atlas_map.txt` (name → x,y,w,h) both versions parse, so sprite
coordinates are defined once.

## Milestones (do them in order, each in both languages before moving on)

1. **M1 — Skeleton**: window, fixed-timestep loop, camera, player AABB moving/jumping
   on a hardcoded flat tile floor.
2. **M2 — World**: full procedural generation, chunked rendering, camera follow,
   collision against generated world.
3. **M3 — Mining**: break/place with reach + highlight, drops, hotbar, pickup.
4. **M4 — Combat**: sword swing, bow + projectiles, damage/HP, death/respawn, HUD
   (hearts + hotbar).
5. **M5 — Enemies**: all three enemy types, spawner, contact damage, knockback, SFX.
6. **M6 — Polish**: particles on block break, screen shake on hurt, main-menu/pause
   state, FPS overlay, tune constants.

---

# terra-c — raylib 6.0 (C)

## Setup & build

- Pin raylib 6.0 by vendoring the release tarball (no cmake on this machine; plain
  Makefile keeps the C side dependency-free). `make` fetches + builds it on first run:
  extracts to `terra-c/vendor/raylib`, builds `libraylib.a` via raylib's own
  `src/Makefile` (`PLATFORM=PLATFORM_DESKTOP`), links with
  `-framework CoreVideo -framework IOKit -framework Cocoa -framework OpenGL` (macOS).
  `vendor/` is throwaway build state — never edit it, re-fetchable any time.

- Files: `main.c`, `world.c/h`, `worldgen.c/h`, `player.c/h`, `entities.c/h`
  (enemies + projectiles + drops), `combat.c/h`, `inventory.c/h`, `render.c/h`,
  `assets.c/h`, `noise.c/h`. One `game.h` with the shared constants and the `Game`
  struct. Keep it boring; no frameworks on top of raylib.

## Best practices for raylib in this project (mini-skill)

- **Own the loop.** raylib is a library, not an engine — the game loop, state, and
  update order are yours. Use a fixed-timestep accumulator; render as fast as vsync
  allows and interpolate is overkill here — simplest correct form:

  ```c
  float acc = 0;
  while (!WindowShouldClose()) {
      acc += GetFrameTime();
      while (acc >= FIXED_DT) { Update(&game, FIXED_DT); acc -= FIXED_DT; }
      BeginDrawing(); Render(&game); EndDrawing();
  }
  ```

  Poll input with the `IsKeyDown`/`IsKeyPressed`/`IsMouseButtonPressed` family inside
  `Update`, but beware: `IsKeyPressed` is per-*frame*, and multiple fixed updates can
  run per frame — latch "pressed" edges once per frame before the inner while-loop and
  pass them into `Update`, or a fast frame will eat/duplicate presses.
- **Zero dynamic allocation in the steady state.** All entity arrays are fixed-size
  pools in the `Game` struct (`Enemy enemies[MAX_ENEMIES]`, `Projectile
  projectiles[64]`, `Drop drops[128]`) with an `active` flag; spawn = find inactive
  slot, despawn = clear flag. The world is one `malloc` at startup. This is the
  idiomatic C answer to what ECS does in Bevy and makes the perf comparison honest.
- **One `Game` struct, passed by pointer.** No globals except the loaded `Assets`.
  Makes update functions testable and state visible.
- **Camera2D does all the work.** `camera.target` = player center (rounded to whole
  pixels), `camera.offset` = half screen, `camera.zoom = 2.0f`. Everything in world
  space between `BeginMode2D(camera)`/`EndMode2D()`; HUD after `EndMode2D()` in screen
  space. Use `GetScreenToWorld2D(GetMousePosition(), camera)` for mouse aim.
- **Atlas + source rects, never per-sprite textures.** Load each Kenney sheet once
  (`LoadTexture`), draw with `DrawTextureRec(tex, srcRect, pos, WHITE)` or
  `DrawTexturePro` when rotating (sword swing). Call
  `SetTextureFilter(tex, TEXTURE_FILTER_POINT)` on every texture — pixel art.
- **Only draw what's visible.** Compute the visible tile rect from the camera
  (`GetScreenToWorld2D` of the two screen corners → tile bounds, clamp to world) and
  loop just those ~40x23 tiles. raylib batches draw calls through rlgl automatically as
  long as you don't switch textures — so draw all tiles (one atlas), then all entities
  (second atlas), then UI.
- **Text/debug**: `DrawFPS(10,10)`; raylib 6.0's expanded text API (`TextFormat` etc.)
  for HUD numbers. `TextFormat` uses a static ring buffer — don't hold the pointer.
- **Audio**: `InitAudioDevice()` once; `Sound` for SFX (`LoadSound`/`PlaySound`),
  `Music` for streaming background track — `UpdateMusicStream` every frame. Remember
  6.0 pan range is `[-1..1]`.
- **Randomness**: our own `splitmix64` in `noise.c` for worldgen (determinism parity
  with Rust); raylib's `GetRandomValue` is fine for gameplay-only rolls (spawn timing).

## Feature implementation notes (C)

- **World**: `uint8_t *tiles` + accessors `TileAt(w,x,y)` (returns solid Air for
  out-of-bounds — kills all edge cases) and `SetTile`. Hardness/damage tracking: tiny
  fixed array of "tiles being damaged" `(x, y, damage)` (8 entries, reset when the
  player retargets) — don't store damage per tile.
- **Rendering the world**: per-frame visible-rect loop drawing `DrawTextureRec` per
  tile is ~1k quads — trivial for rlgl's batcher. No chunk caching needed in C; if we
  ever want it, render chunks to `RenderTexture2D`s and redraw a chunk only when a tile
  in it changes.
- **Collision**: `MoveAndCollide(Rectangle *box, Vector2 *vel, float dt)` in
  `world.c` — move X, scan overlapped tile columns, clamp + zero `vel.x` on hit; then
  same for Y; sets a `grounded` out-flag when clamping downward. Shared by player,
  zombies, slimes, drops.
- **Entities**: plain structs + pools (above). Each type gets `UpdateSlimes(Game*)`,
  etc. — dumb loops over pools; no function pointers or vtables, C's simplicity is the
  point of the comparison.
- **Combat**: sword = a `swing_t` timer on the player; while active, AABB-overlap test
  vs enemy pool with a per-swing `already_hit` bitmask so one swing hits once.
  Projectiles: one pool, `faction` field decides collision target (player vs enemies).
- **Worldgen**: pure functions in `worldgen.c` operating on the tile array +
  `splitmix64` state. Must produce byte-identical output to the Rust version for the
  same seed — write a dump-world-hash debug command (`--dump-seed N` prints FNV-1a hash
  of the tile array) in BOTH versions to verify parity.
- **HUD**: hearts = `DrawTextureRec` heart icon xN; hotbar = 8 slot frames + item
  icons + count via `DrawText`. All in screen space after `EndMode2D`.
- **Game states**: `enum GameState { STATE_MENU, STATE_PLAYING, STATE_PAUSED }` +
  switch in the loop. That's all a menu needs.

---

# terra-rs — Bevy 0.19 (Rust)

## Setup & build

```toml
[dependencies]
bevy = { version = "0.19", default-features = false, features = ["2d"] }
# "2d" is a 0.18+ scenario feature-collection: sprites, audio, text, UI, winit, etc.

[features]
dev = ["bevy/dynamic_linking", "bevy/bevy_dev_tools"]

# Fast iterative builds:
[profile.dev]
opt-level = 1
[profile.dev.package."*"]
opt-level = 3
```

Develop with `cargo run --features dev` (dynamic linking cuts link time massively);
never ship with it. Modules: `main.rs`, `world.rs`, `worldgen.rs`, `player.rs`,
`enemies.rs`, `projectiles.rs`, `combat.rs`, `inventory.rs`, `render.rs`, `hud.rs`,
`noise.rs` — each gameplay module is a Bevy `Plugin`.

**No gameplay crates** — no `bevy_ecs_tilemap`, no `avian`/`bevy_rapier`, no `noise`
crate. Same hand-rolled collision, noise, and PRNG as the C version, or the language
comparison is meaningless. Dev-only diagnostic crates are fine.

## Best practices for Bevy 0.19 in this project (mini-skill)

- **App skeleton**:

  ```rust
  App::new()
      .add_plugins(DefaultPlugins
          .set(ImagePlugin::default_nearest())        // pixel art: no filtering
          .set(WindowPlugin { /* 1280x720, title */ ..default() }))
      .init_state::<GameState>()                       // Menu / Playing / Paused
      .insert_resource(Time::<Fixed>::from_hz(60.0))
      .add_plugins((WorldPlugin, PlayerPlugin, EnemyPlugin, /* ... */))
      .run();
  ```

- **Required components, not bundles.** Spawn `Sprite { image, texture_atlas, .. }` +
  `Transform` and Bevy inserts `Visibility` etc. automatically. `Camera2d` is a
  component: `commands.spawn((Camera2d, Projection::Orthographic(ortho)))` with
  `ortho.scale = 0.5` for 2x zoom. If you find yourself typing `...Bundle`, you're
  writing pre-0.15 Bevy — stop and check docs.
- **Fixed vs frame schedules.** All simulation (movement, collision, AI, combat
  timers, spawner) in `FixedUpdate`; input *reading* and rendering-side systems
  (camera follow, HUD) in `Update`. Bevy interpolates nothing for you — since we run
  render-visible movement in FixedUpdate at 60 Hz on a 60 Hz display this is fine;
  don't add interpolation complexity.
- **Ordering within FixedUpdate** via system sets:

  ```rust
  #[derive(SystemSet, ...)] enum SimSet { Input, Ai, Physics, Combat, Cleanup }
  app.configure_sets(FixedUpdate,
      (SimSet::Input, SimSet::Ai, SimSet::Physics, SimSet::Combat, SimSet::Cleanup)
          .chain().run_if(in_state(GameState::Playing)));
  ```

- **Messages vs observer events (0.17+ naming — get this right):**
  - Buffered, many-per-tick, polled: **Messages** — `#[derive(Message)]`,
    `app.add_message::<TileChanged>()`, `MessageWriter<TileChanged>`,
    `MessageReader<TileChanged>`. Use for `TileChanged`, `DamageDealt`.
  - Reactive, targeted at an entity: **observers** — `#[derive(EntityEvent)]`,
    `commands.trigger(...)`, handler takes `On<Death>`. Use for `Death` (spawn poof,
    free the slot) and pickup. 0.19 lets observers take `.run_if(...)`.
- **Marker components + small value components** (`Health(i32)`, `Velocity(Vec2)`,
  `Grounded(bool)`, `Faction`, markers `Player`, `Enemy`, `ProjectileMarker`). Query
  by markers; share systems via components, e.g. one `apply_gravity` system over
  `Query<&mut Velocity, With<GravityAffected>>` covers player, zombies, drops, arrows.
- **`Single` for the player.** `player: Single<(&Transform, &mut Velocity), With<Player>>`
  — panics-free access to exactly-one entities; skips the system if absent.
- **The tile world is a `Resource`, not entities.** `struct TileWorld { tiles: Vec<u8> }`
  with the same `tile_at`/`set_tile` accessors as C. Tiles-as-entities is a trap at
  256k tiles. Only *visible chunk sprites* are entities (below).
- **Asset loading**: `AssetServer::load` + `TextureAtlasLayout` built from the shared
  `atlas_map.txt`; store handles in a `GameAssets` resource populated in a `Startup`
  system, gate `GameState::Playing` on load completion (check
  `asset_server.is_loaded_with_dependencies` or just a loading state).
- **0.19 niceties worth using**: `bsn!` + `commands.spawn_scene()` for structured
  spawns like the hotbar UI tree (8 child slots); `commands.delayed().secs(0.5)` for
  invulnerability-end instead of hand-rolled timer components where it's cleaner;
  `DiagnosticsOverlay::fps()` for the FPS overlay milestone.
- **Change detection is your friend**: HUD systems run on `Changed<Health>` /
  `Changed<Inventory>`-filtered queries instead of every frame.

## Feature implementation notes (Bevy)

- **World render — chunked sprites**: entities only for tiles in loaded chunks.
  `ChunkMap: Resource` maps chunk coord → spawned chunk root entity; a `FixedUpdate`
  (or `Update`) system loads chunks within camera radius +1 and despawns beyond +2
  (hysteresis). Chunk root = `(Transform, Visibility, ChunkCoord)`, tiles spawned as
  child sprites via the `children![]` / BSN spawn — ~1024 sprites per chunk, ~6 chunks
  live; Bevy's sprite batcher eats this easily. On `TileChanged` message: find the
  chunk root, despawn/respawn just that tile's child (store a per-chunk
  `HashMap<(u8,u8), Entity>` on the chunk root component).
- **Collision**: same per-axis AABB-vs-grid routine as C, as a free function in
  `world.rs` taking `&TileWorld`; a `move_and_collide` system runs it over
  `Query<(&mut Transform, &mut Velocity, &Collider, Option<&mut Grounded>)>` in
  `SimSet::Physics`. No physics crate.
- **Mining/placing**: `Update` system reads mouse, converts via
  `camera.viewport_to_world_2d(cam_transform, cursor)` (returns `Result` — handle it),
  writes an `AimTarget` resource; `FixedUpdate` system applies hits/places, emits
  `TileChanged`. Highlight = one `Gizmos` rect (`gizmos.rect_2d`) — dev-cheap and
  fine for v1.
- **Drops**: entities with `Sprite + Velocity + GravityAffected + DropItem(TileId)`;
  homing system checks distance to player; pickup via distance check emitting a
  triggered `PickedUp` event on the drop → observer adds to `Inventory` resource and
  despawns.
- **Combat**: sword swing = spawn a short-lived `MeleeHitbox { dmg, hit: HashSet<Entity> }`
  child of the player (`ChildOf` relationship gives it player-relative transform for
  free); overlap system in `SimSet::Combat` applies damage. Sword visual = child sprite
  whose `Transform::rotation` is driven by swing timer. Projectiles = `(Sprite,
  Velocity, ProjectileMarker, Faction, Damage, Lifetime(Timer))`; one movement system,
  one tile-hit system, one entity-hit system.
- **Enemies**: per-type marker + one AI system each (`slime_ai`, `zombie_ai`,
  `bee_ai`) in `SimSet::Ai` writing `Velocity`; shared gravity/collision/contact-damage
  systems. Spawner = `FixedUpdate` system with a `Timer` in a resource. Death =
  observer on `Death` event: spawn particles, despawn (despawns recurse to children
  automatically via `ChildOf`).
- **HUD**: `bevy_ui` — root `Node`, hearts row + hotbar row of 8 slots (spawn with
  `bsn!`), `ImageNode` icons + `Text` counts; update systems filtered on
  `Changed<Health>` / hotbar-selection resource change. World-space damage numbers:
  `Text2d`.
- **Worldgen**: same `splitmix64` + value-noise code, pure functions in
  `worldgen.rs`; `--dump-seed N` (read via `std::env::args`) printing the same FNV-1a
  world hash as the C build — run both to verify parity per milestone.
- **States**: `GameState { Menu, Playing, Paused }` via `init_state`; menu/pause UI
  spawned `OnEnter(state)` + despawned `OnExit(state)` (use `DespawnOnExit(state)`-
  style component or manual cleanup system); sim sets gated with
  `run_if(in_state(GameState::Playing))`.

---

# Comparison notes (why the rules above exist)

- Fairness rules: identical features, identical algorithms (noise, PRNG, collision),
  no third-party gameplay libraries on either side, shared assets + atlas map. The
  variables under test are the *languages* and the *library models* (immediate-mode C
  loop + pools vs ECS scheduling + plugins), not ecosystem crates.
- Things worth measuring at the end: LOC per feature, compile/iterate time, time-to-
  implement per milestone (keep a log), frame time with 8 enemies + 100 projectiles +
  full-screen tile redraw, binary size, memory footprint, and subjective notes on
  refactoring ease (e.g., adding knockback after the fact).
- Keep a `NOTES.md` in each subproject: friction diary, one bullet whenever the
  language/library helped or hurt. That's the actual deliverable of this experiment.
