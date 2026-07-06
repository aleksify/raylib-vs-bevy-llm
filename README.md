# Terra — one game, two engines

A lightweight Terraria clone implemented **twice, feature-identically**, to compare
C + raylib against Rust + Bevy for game development — with the twist that almost all
of the code was written by an LLM (Claude), making this also an experiment in which
stack is friendlier to AI-assisted development.

| | `terra-c/` | `terra-rs/` |
|---|---|---|
| Language | C99 | Rust |
| Library | raylib 6.0 (vendored) | Bevy 0.19 |
| Build | `make` (fetches raylib on first run) | `cargo build` |
| Run | `cd terra-c && make run` | `cd terra-rs && cargo run` |

## What's implemented (both versions, verified identical where it counts)

- **Procedural worldgen** — 1024×256 tiles: fBm value-noise heightmap, cellular-
  automata caves, drunkard's-walk ore veins, trees. Hand-rolled splitmix64 PRNG and
  noise on both sides (no libraries) — **worlds are byte-identical across languages**:
  `./terra --dump-seed 42` and `terra-rs --dump-seed 42` print the same FNV-1a hash
  (`7d8ba3bc5de357ba`). This required `-ffp-contract=off` on the C side so clang
  wouldn't fuse `a*b+c` into fma and diverge from Rust's float math.
- **Platformer physics** — fixed 60 Hz timestep, per-axis swept AABB vs the tile grid
  (same routine both sides, shared by player, enemies, drops).
- **Mining & building** — mouse-aimed break (per-tile hardness, hit cooldown, damage
  table) and place (reach, adjacency, no-overlap rules), bouncing drops that home to
  the player, 8-slot hotbar (keys 1–8 / wheel).
- **Combat** — sword swing with a directional hitbox and once-per-swing hit tracking,
  bow with gravity-affected arrows, generic faction-based projectiles, HP/invuln/
  knockback, death & respawn, hearts HUD.
- **Enemies** — slime (hops), zombie (walks, jumps 1-tile walls), bee (flies in a sine
  wave, shoots); spawner with distance/count rules, despawn when far.
- **Polish** — block-break particles, death poofs, screen shake, menu/pause states,
  FPS overlay.
- **Art** — Kenney's CC0 packs (Pixel Platformer, Tiny Dungeon, UI, audio) in a shared
  `assets/` dir; one hand-written `atlas_map.txt` parsed by both engines; colored-rect
  fallbacks everywhere so both games run asset-less too.
- **Self-test harness** — both binaries accept `--screenshot out.png --frames N`
  (simulate N frames, save a frame, exit) so an agent can eyeball its own work.

Not done: SFX wiring (packs are downloaded), feel-tuning, final release-build
measurements, and visual verification of the new sprite rendering (the screen was
locked during the last session — run the `--screenshot` commands with the display
unlocked).

Per-milestone friction diaries live in `terra-c/NOTES.md` and `terra-rs/NOTES.md`;
the build spec is `CLAUDE.md`.

## Numbers so far (debug builds, one machine — Apple M4)

| | C / raylib | Rust / Bevy |
|---|---|---|
| Source LOC | ~1,340 | ~2,260 |
| Cold build | ~1 min (incl. raylib) | ~6 min |
| Incremental rebuild | ~1 s | 0.5–4 s |
| Binary | 0.9 MB | 138 MB debug (release is far smaller) |
| Runtime bugs during development | 2 | 0 |
| Compile-time API fixes needed | 0 | ~5 |

## So… which is better for LLM development?

Honest verdict from the LLM that wrote both, having kept a diary the whole way:

**raylib's superpower is that the LLM's memory of it is simply correct.** The API has
been stable for years; every function I wrote from recall existed with the signature I
expected, in both raylib 5.x and 6.0. The immediate-mode model also means less
architecture to get wrong: state is a struct, the loop is yours, a feature is a
for-loop over a pool. The C side of every milestone was written essentially blind and
compiled first try. The cost is C itself: the two genuine bugs of this project were
both C bugs an LLM is prone to and a compiler won't catch — an include-guard macro
(`WORLD_H`) silently colliding with a constant of the same name, and a decay timer
that was set but never decremented (caught only by self-review, would have shipped).

**Bevy's superpower is that nothing broken survives to runtime.** The Rust side hit
five API-churn problems — `WindowResolution` changing units, the `2d` feature
collection not including UI, `font_size` becoming an enum, `DiagnosticsOverlay` being
feature-gated, plus assorted renames — and every single one surfaced as a compile
error with a usable message. Across six milestones the Rust build produced **zero
runtime bugs**: if it compiled, it behaved. For an agent that iterates by
write→compile→fix, that property is worth a lot; debugging a black window costs far
more tokens than fixing a type error. The costs: Bevy's API churns so hard that the
LLM's training data is actively poisonous (the project's CLAUDE.md maintains a "dead
patterns you must not write" list, and I grepped the actual 0.19 sources in
`~/.cargo/registry` before using any API I wasn't sure of — that workflow, *verify
against vendored source, not memory*, was the single biggest success factor), ~70%
more lines for the same features, borrow-checker ceremony like proving query
disjointness with `Without<>` filters, and a 6-minute cold build.

**Recommendation:**
- Small scope, fast iteration, or an LLM working without tooling access → **raylib**.
  Stable knowledge beats a safety net when the knowledge is actually right.
- Long-lived projects, large refactors, or agentic loops where the LLM runs the
  compiler itself → **Bevy**, *provided* the agent verifies current APIs against
  docs/source instead of trusting recall, and the project pins exact versions with
  migration notes in its context file.
- Either way: byte-identical cross-language worldgen was achievable with ~40 lines of
  hand-rolled noise per side and one compiler flag — determinism parity is a cheap,
  brutal correctness test and caught the fma divergence risk early. Recommended for
  any twin-implementation experiment.

## Credits

Art & audio: [Kenney](https://kenney.nl) (CC0). Engines: [raylib](https://raylib.com),
[Bevy](https://bevy.org). Code: Claude (Anthropic), driven by a human with taste.
