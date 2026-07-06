#ifndef TERRA_WORLDGEN_H
#define TERRA_WORLDGEN_H

#include "game.h"

// Allocates w->tiles and generates the full world. Must produce byte-identical
// output to terra-rs worldgen::generate() for the same seed (verify with
// --dump-seed on both builds).
void WorldGenerate(World *w, uint64_t seed);

int WorldSurfaceY(const World *w, int tx); // first solid tile y in column

uint64_t WorldHash(const World *w);        // FNV-1a over the tile array

#endif // TERRA_WORLDGEN_H
