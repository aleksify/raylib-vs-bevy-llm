#include "world.h"
#include <stdlib.h>
#include <math.h>

void WorldFree(World *w)
{
    free(w->tiles);
    w->tiles = NULL;
}

uint8_t TileAt(const World *w, int tx, int ty)
{
    if (tx < 0 || ty < 0 || tx >= WORLD_W || ty >= WORLD_H) return TILE_STONE;
    return w->tiles[ty * WORLD_W + tx];
}

void SetTile(World *w, int tx, int ty, uint8_t t)
{
    if (tx < 0 || ty < 0 || tx >= WORLD_W || ty >= WORLD_H) return;
    w->tiles[ty * WORLD_W + tx] = t;
}

bool TileIsSolid(uint8_t t)
{
    return t != TILE_AIR && t != TILE_LEAVES;
}

bool SolidAt(const World *w, int tx, int ty)
{
    return TileIsSolid(TileAt(w, tx, ty));
}

void MoveAndCollide(const World *w, Rectangle *box, Vector2 *vel, float dt, bool *grounded)
{
    // X axis
    float newX = box->x + vel->x * dt;
    if (vel->x != 0) {
        float edge = (vel->x > 0) ? newX + box->width : newX;
        int tx = (int)floorf(edge / TILE_SIZE);
        int ty0 = (int)floorf(box->y / TILE_SIZE);
        int ty1 = (int)floorf((box->y + box->height - 0.001f) / TILE_SIZE);
        for (int ty = ty0; ty <= ty1; ty++) {
            if (SolidAt(w, tx, ty)) {
                newX = (vel->x > 0) ? (float)(tx * TILE_SIZE) - box->width
                                    : (float)((tx + 1) * TILE_SIZE);
                vel->x = 0;
                break;
            }
        }
    }
    box->x = newX;

    // Y axis
    if (grounded) *grounded = false;
    float newY = box->y + vel->y * dt;
    if (vel->y != 0) {
        float edge = (vel->y > 0) ? newY + box->height : newY;
        int ty = (int)floorf(edge / TILE_SIZE);
        int tx0 = (int)floorf(box->x / TILE_SIZE);
        int tx1 = (int)floorf((box->x + box->width - 0.001f) / TILE_SIZE);
        for (int tx = tx0; tx <= tx1; tx++) {
            if (SolidAt(w, tx, ty)) {
                if (vel->y > 0) {
                    newY = (float)(ty * TILE_SIZE) - box->height;
                    if (grounded) *grounded = true;
                } else {
                    newY = (float)((ty + 1) * TILE_SIZE);
                }
                vel->y = 0;
                break;
            }
        }
    }
    box->y = newY;
}
