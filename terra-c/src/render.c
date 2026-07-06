#include "render.h"
#include "world.h"
#include <math.h>

// Placeholder colors until Kenney atlas lands
static const Color TILE_COLORS[TILE_COUNT] = {
    { 0, 0, 0, 0 },        // AIR
    { 133, 87, 35, 255 },  // DIRT
    { 91, 154, 60, 255 },  // GRASS
    { 120, 120, 125, 255 },// STONE
    { 94, 62, 24, 255 },   // WOOD
    { 118, 190, 88, 180 }, // LEAVES
    { 155, 105, 185, 255 },// ORE
};

static Color ItemColor(uint8_t item)
{
    if (item == ITEM_SWORD) return (Color){ 200, 200, 215, 255 };
    if (item == ITEM_BOW)   return (Color){ 190, 140, 80, 255 };
    if (item < TILE_COUNT)  return TILE_COLORS[item];
    return MAGENTA;
}

static void DrawWorld(const Game *g)
{
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
}

static void DrawHotbar(const Game *g)
{
    const int slot = 40, gap = 4;
    int total = INV_SLOTS * slot + (INV_SLOTS - 1) * gap;
    int x0 = (GetScreenWidth() - total) / 2;
    int y0 = GetScreenHeight() - slot - 12;
    for (int i = 0; i < INV_SLOTS; i++) {
        int x = x0 + i * (slot + gap);
        DrawRectangle(x, y0, slot, slot, (Color){ 0, 0, 0, 120 });
        Color border = (i == g->inv.selected) ? YELLOW : (Color){ 220, 220, 220, 160 };
        DrawRectangleLinesEx((Rectangle){ (float)x, (float)y0, slot, slot }, 2, border);
        ItemSlot s = g->inv.slots[i];
        if (s.id != ITEM_NONE) {
            DrawRectangle(x + 10, y0 + 10, slot - 20, slot - 20, ItemColor(s.id));
            if (s.count > 1)
                DrawText(TextFormat("%d", s.count), x + 4, y0 + slot - 13, 10, RAYWHITE);
        }
        DrawText(TextFormat("%d", i + 1), x + 3, y0 + 2, 10, (Color){ 255, 255, 255, 140 });
    }
}

void RenderGame(const Game *g)
{
    ClearBackground((Color){ 108, 158, 222, 255 }); // sky

    BeginMode2D(g->camera);

    DrawWorld(g);

    for (int i = 0; i < MAX_DROPS; i++) {
        if (!g->drops[i].active) continue;
        DrawRectangleRec(g->drops[i].box, ItemColor(g->drops[i].item));
    }

    DrawRectangleRec(g->player.box, (Color){ 235, 90, 70, 255 });

    if (g->aimInReach) {
        DrawRectangleLinesEx(
            (Rectangle){ g->aimX * TILE_SIZE, g->aimY * TILE_SIZE, TILE_SIZE, TILE_SIZE },
            1, WHITE);
    }

    EndMode2D();

    DrawHotbar(g);
    DrawFPS(10, 10);
}
