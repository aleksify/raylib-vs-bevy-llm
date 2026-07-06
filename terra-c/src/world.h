#ifndef TERRA_WORLD_H
#define TERRA_WORLD_H

#include "game.h"

void WorldFree(World *w);

uint8_t TileAt(const World *w, int tx, int ty);      // OOB -> TILE_STONE (solid)
void SetTile(World *w, int tx, int ty, uint8_t t);
bool TileIsSolid(uint8_t t);
bool SolidAt(const World *w, int tx, int ty);

// Per-axis swept AABB vs the solid-tile grid. Moves box by vel*dt, clamps on
// hit and zeroes the blocked velocity component. Sets *grounded when clamped
// while moving down. Assumes speeds < TILE_SIZE per step (no tunneling check).
void MoveAndCollide(const World *w, Rectangle *box, Vector2 *vel, float dt, bool *grounded);

#endif // TERRA_WORLD_H
