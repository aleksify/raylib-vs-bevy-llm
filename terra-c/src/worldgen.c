#include "worldgen.h"
#include "world.h"
#include "noise.h"
#include <stdlib.h>

// Generation runs in fixed stages, each on its own splitmix64 stream
// (seed+0..3) so adding calls to one stage never shifts another stage's
// randomness. Stage order and iteration order must match terra-rs exactly.

void WorldGenerate(World *w, uint64_t seed)
{
    w->tiles = calloc((size_t)WORLD_W * WORLD_H, 1);

    static int surface[WORLD_W];
    static int caveMin[WORLD_W];

    // 1+2: heightmap (pure fbm of column x) + strata
    uint64_t rStrata = seed;
    for (int x = 0; x < WORLD_W; x++) {
        float h = Fbm1(seed, (float)x * 0.015f);
        int sy = (int)(WORLD_H * 0.25f + h * (WORLD_H * 0.20f));
        surface[x] = sy;
        int dirtLen = RngRange(&rStrata, 6, 10);
        caveMin[x] = sy + dirtLen + 11; // skip 10 tiles below the dirt layer
        for (int y = sy; y < WORLD_H; y++) {
            uint8_t t;
            if (y == sy)                t = TILE_GRASS;
            else if (y <= sy + dirtLen) t = TILE_DIRT;
            else                        t = TILE_STONE;
            w->tiles[y * WORLD_W + x] = t;
        }
    }

    // 3: caves — cellular automata on the stone layer, 45% fill, 5 passes,
    // 4/5 rule (3x3 count incl. self, >=5 -> wall), OOB counts as wall
    uint8_t *walls = malloc((size_t)WORLD_W * WORLD_H);
    uint8_t *next = malloc((size_t)WORLD_W * WORLD_H);
    uint64_t rCaves = seed + 1;
    for (int y = 0; y < WORLD_H; y++) {
        for (int x = 0; x < WORLD_W; x++) {
            int i = y * WORLD_W + x;
            walls[i] = (y >= caveMin[x]) ? (RngFloat(&rCaves) < 0.45f) : 1;
        }
    }
    for (int pass = 0; pass < 5; pass++) {
        for (int y = 0; y < WORLD_H; y++) {
            for (int x = 0; x < WORLD_W; x++) {
                int i = y * WORLD_W + x;
                if (y < caveMin[x]) { next[i] = 1; continue; }
                int n = 0;
                for (int dy = -1; dy <= 1; dy++) {
                    for (int dx = -1; dx <= 1; dx++) {
                        int nx = x + dx, ny = y + dy;
                        if (nx < 0 || ny < 0 || nx >= WORLD_W || ny >= WORLD_H) n++;
                        else n += walls[ny * WORLD_W + nx];
                    }
                }
                next[i] = (n >= 5);
            }
        }
        uint8_t *tmp = walls; walls = next; next = tmp;
    }
    for (int y = 0; y < WORLD_H; y++)
        for (int x = 0; x < WORLD_W; x++)
            if (y >= caveMin[x] && !walls[y * WORLD_W + x])
                w->tiles[y * WORLD_W + x] = TILE_AIR;
    free(walls);
    free(next);

    // 4: ore — 200 drunkard's walks through stone
    uint64_t rOre = seed + 2;
    for (int i = 0; i < 200; i++) {
        int x = RngRange(&rOre, 0, WORLD_W - 1);
        int y = RngRange(&rOre, 0, WORLD_H - 1);
        int len = RngRange(&rOre, 4, 10);
        for (int s = 0; s < len; s++) {
            if (TileAt(w, x, y) == TILE_STONE) SetTile(w, x, y, TILE_ORE);
            switch (RngRange(&rOre, 0, 3)) {
                case 0: x++; break;
                case 1: x--; break;
                case 2: y++; break;
                default: y--; break;
            }
        }
    }

    // 5: trees — ~10% of grass columns, min 4 apart
    uint64_t rTrees = seed + 3;
    int lastTree = -100;
    for (int x = 2; x < WORLD_W - 2; x++) {
        if (x - lastTree < 4) continue;
        if (TileAt(w, x, surface[x]) != TILE_GRASS) continue;
        if (RngFloat(&rTrees) >= 0.10f) continue;
        lastTree = x;
        int trunk = RngRange(&rTrees, 4, 7);
        for (int t = 1; t <= trunk; t++) SetTile(w, x, surface[x] - t, TILE_WOOD);
        int cy = surface[x] - trunk - 1;
        for (int dy = -1; dy <= 1; dy++)
            for (int dx = -1; dx <= 1; dx++)
                if (TileAt(w, x + dx, cy + dy) == TILE_AIR)
                    SetTile(w, x + dx, cy + dy, TILE_LEAVES);
    }
}

int WorldSurfaceY(const World *w, int tx)
{
    for (int y = 0; y < WORLD_H; y++)
        if (TileIsSolid(TileAt(w, tx, y))) return y;
    return WORLD_H / 2;
}

uint64_t WorldHash(const World *w)
{
    return Fnv1a64(w->tiles, (size_t)WORLD_W * WORLD_H);
}
