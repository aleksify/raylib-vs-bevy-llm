#include "render.h"
#include "world.h"
#include <math.h>

// Placeholder colors until Kenney atlas lands (M2+)
static const Color TILE_COLORS[TILE_COUNT] = {
    { 0, 0, 0, 0 },        // AIR
    { 133, 87, 35, 255 },  // DIRT
    { 91, 154, 60, 255 },  // GRASS
    { 120, 120, 125, 255 },// STONE
    { 94, 62, 24, 255 },   // WOOD
    { 118, 190, 88, 180 }, // LEAVES
    { 155, 105, 185, 255 },// ORE
};

void RenderGame(const Game *g)
{
    ClearBackground((Color){ 108, 158, 222, 255 }); // sky

    BeginMode2D(g->camera);

    // Visible tile bounds from camera corners
    Vector2 tl = GetScreenToWorld2D((Vector2){ 0, 0 }, g->camera);
    Vector2 br = GetScreenToWorld2D((Vector2){ (float)GetScreenWidth(), (float)GetScreenHeight() }, g->camera);
    int tx0 = (int)floorf(tl.x / TILE_SIZE); if (tx0 < 0) tx0 = 0;
    int ty0 = (int)floorf(tl.y / TILE_SIZE); if (ty0 < 0) ty0 = 0;
    int tx1 = (int)floorf(br.x / TILE_SIZE); if (tx1 >= WORLD_W) tx1 = WORLD_W - 1;
    int ty1 = (int)floorf(br.y / TILE_SIZE); if (ty1 >= WORLD_H) ty1 = WORLD_H - 1;

    for (int ty = ty0; ty <= ty1; ty++) {
        for (int tx = tx0; tx <= tx1; tx++) {
            uint8_t t = g->world.tiles[ty * WORLD_W + tx];
            if (t == TILE_AIR) continue;
            DrawRectangle(tx * TILE_SIZE, ty * TILE_SIZE, TILE_SIZE, TILE_SIZE, TILE_COLORS[t]);
        }
    }

    DrawRectangleRec(g->player.box, (Color){ 235, 90, 70, 255 });

    EndMode2D();

    DrawFPS(10, 10);
}
