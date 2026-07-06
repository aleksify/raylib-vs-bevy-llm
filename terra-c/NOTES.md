# terra-c friction diary

## M1
- HURT (C): include guard `WORLD_H` silently collided with the `WORLD_H` constant
  macro in game.h — world height became empty text, confusing redefinition warning.
  Renamed all guards `TERRA_*_H`. Classic C namespace hazard, cost ~5 min.
- HELPED (raylib): whole M1 loop (window + camera + input + draw) is ~30 lines of
  obvious calls, zero ceremony. First compile ran.
- Build: no cmake on machine; raylib's own `src/Makefile` built `libraylib.a` in
  ~1 min, plain Makefile suffices. Vendored release tarball (git clone of tag was
  blocked by sandbox; tarball simpler anyway).

## M2
- HELPED (C): worldgen is plain nested loops over a byte array — wrote once,
  compiled, hash-matched Rust on first try (after -ffp-contract=off).
- GOTCHA (clang): `-ffp-contract=on` is clang's default, contracts `a*b+c` to fma
  — would silently break float parity with Rust. `-ffp-contract=off` in CFLAGS.
- Rendering: no chunk system needed at all — per-frame visible-rect loop over
  ~1k tiles, rlgl batches it. C side rendering was zero extra work this milestone.

## M3
- HELPED (C): tile edits are literally `tiles[i] = AIR` — the immediate-mode
  renderer picks it up next frame for free. No change-propagation code at all
  (Rust side needed a TileChanged message + chunk rebuild).
- HELPED (raylib): `CheckCollisionRecs` + `GetScreenToWorld2D` covered pickup
  and mouse aim; hotbar HUD is ~20 lines of DrawRectangle/DrawText.
- Pattern: drop pool reuses MoveAndCollide; bounce = restore `-0.4 * pre_vy`
  after the collide call zeroes it. Pools keep everything allocation-free.

## M4
- HELPED (C): per-swing "already hit" is a uint8 bitmask over the 8-slot enemy
  pool — one line to set, one to test. The Rust equivalent is a HashSet<Entity>.
- NEAT (raylib): sword arc = one DrawRectanglePro rotating around the hand
  origin; angle math is 3 lines.
- GOTCHA (self-inflicted): enemy hurtFlash was set on hit but never decayed —
  would have rendered enemies permanently white. Caught in review, decays in
  UpdateCombat now.
- LMB dispatch (mine vs swing vs shoot by selected item) lives in two places
  (mining.c gates itself out, combat.c gates itself in) — must stay in sync.

## M5
- HELPED (C): all three AIs are one switch in one loop — slime is 10 lines,
  zombie 3 + a blocked-jump check after MoveAndCollide (compare wanted vx vs
  post-collide vx == 0; the collide routine gives this for free).
- Bee reuses the generic projectile pool with FACTION_ENEMY — zero new code
  for its shots hitting the player; that path existed since M4.
- Spawner: ~25 lines with the pool. WorldSurfaceY reused for spawn placement.
