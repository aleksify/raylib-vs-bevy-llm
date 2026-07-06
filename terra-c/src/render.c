#include "render.h"
#include "world.h"
#include "assets.h"
#include <math.h>
#include <stddef.h>

// Fallback palette when assets are missing; also feeds particle colors
static const Color TILE_COLORS[TILE_COUNT] = {
    { 0, 0, 0, 0 },        // AIR
    { 133, 87, 35, 255 },  // DIRT
    { 91, 154, 60, 255 },  // GRASS
    { 120, 120, 125, 255 },// STONE
    { 94, 62, 24, 255 },   // WOOD
    { 118, 190, 88, 180 }, // LEAVES
    { 155, 105, 185, 255 },// ORE
};

static const Color ENEMY_COLORS[3] = {
    { 90, 200, 120, 255 },  // slime
    { 110, 130, 110, 255 }, // zombie
    { 230, 190, 60, 255 },  // bee
};

static const char *TILE_SPRITES[TILE_COUNT] = {
    NULL, "tile_dirt", "tile_grass", "tile_stone",
    "tile_wood", "tile_leaves", "tile_ore",
};
static const char *ENEMY_SPRITES[3] = { "char_slime", "char_zombie", "char_bee" };

Color TileColor(uint8_t t) { return (t < TILE_COUNT) ? TILE_COLORS[t] : MAGENTA; }
Color EnemyColorOf(uint8_t type) { return ENEMY_COLORS[type]; }

static Color ItemColor(uint8_t item)
{
    if (item == ITEM_SWORD) return (Color){ 200, 200, 215, 255 };
    if (item == ITEM_BOW)   return (Color){ 190, 140, 80, 255 };
    if (item < TILE_COUNT)  return TILE_COLORS[item];
    return MAGENTA;
}

static void DrawSpriteRect(const SpriteDef *s, Rectangle dest, Color tint)
{
    DrawTexturePro(g_assets.sheets[s->sheet], s->src, dest, (Vector2){ 0, 0 }, 0, tint);
}

// Sprite icon for an item id, NULL -> use ItemColor rect
static const SpriteDef *ItemSprite(uint8_t item)
{
    if (item == ITEM_SWORD) return GetSprite("item_sword");
    if (item == ITEM_BOW) return NULL; // no bow in Tiny Dungeon; rect icon
    if (item > TILE_AIR && item < TILE_COUNT && TILE_SPRITES[item])
        return GetSprite(TILE_SPRITES[item]);
    return NULL;
}

static void DrawWorld(const Game *g)
{
    Vector2 tl = GetScreenToWorld2D((Vector2){ 0, 0 }, g->camera);
    Vector2 br = GetScreenToWorld2D((Vector2){ (float)GetScreenWidth(), (float)GetScreenHeight() }, g->camera);
    int tx0 = (int)floorf(tl.x / TILE_SIZE); if (tx0 < 0) tx0 = 0;
    int ty0 = (int)floorf(tl.y / TILE_SIZE); if (ty0 < 0) ty0 = 0;
    int tx1 = (int)floorf(br.x / TILE_SIZE); if (tx1 >= WORLD_W) tx1 = WORLD_W - 1;
    int ty1 = (int)floorf(br.y / TILE_SIZE); if (ty1 >= WORLD_H) ty1 = WORLD_H - 1;

    const SpriteDef *spr[TILE_COUNT];
    for (int t = 0; t < TILE_COUNT; t++)
        spr[t] = TILE_SPRITES[t] ? GetSprite(TILE_SPRITES[t]) : NULL;

    for (int ty = ty0; ty <= ty1; ty++) {
        for (int tx = tx0; tx <= tx1; tx++) {
            uint8_t t = g->world.tiles[ty * WORLD_W + tx];
            if (t == TILE_AIR) continue;
            Rectangle dest = { tx * TILE_SIZE, ty * TILE_SIZE, TILE_SIZE, TILE_SIZE };
            if (spr[t]) DrawSpriteRect(spr[t], dest, WHITE);
            else DrawRectangleRec(dest, TILE_COLORS[t]);
        }
    }
}

static void DrawPlayer(const Game *g)
{
    const Player *p = &g->player;
    Color tint = (p->invulnT > 0) ? (Color){ 255, 255, 255, 120 } : WHITE;
    const SpriteDef *s = GetSprite("char_player");
    if (s) {
        Rectangle src = s->src;
        if (p->facing < 0) src.width = -src.width; // horizontal flip
        float cx = p->box.x + p->box.width / 2;
        Rectangle dest = { cx - 12, p->box.y + p->box.height - 24, 24, 24 };
        DrawTexturePro(g_assets.sheets[s->sheet], src, dest, (Vector2){ 0, 0 }, 0, tint);
    } else {
        Color c = { 235, 90, 70, tint.a };
        DrawRectangleRec(p->box, c);
    }
}

static void DrawEnemies(const Game *g)
{
    for (int i = 0; i < MAX_ENEMIES; i++) {
        const Enemy *e = &g->enemies[i];
        if (!e->active) continue;
        Color tint = (e->hurtFlash > 0) ? (Color){ 255, 110, 110, 255 } : WHITE;
        const SpriteDef *s = GetSprite(ENEMY_SPRITES[e->type]);
        if (s) {
            float side = (e->type == ENEMY_ZOMBIE) ? 24 : 16;
            float cx = e->box.x + e->box.width / 2;
            Rectangle dest = { cx - side / 2, e->box.y + e->box.height - side, side, side };
            DrawSpriteRect(s, dest, tint);
        } else {
            Color c = (e->hurtFlash > 0) ? RAYWHITE : ENEMY_COLORS[e->type];
            DrawRectangleRec(e->box, c);
        }
    }
}

static void DrawProjectiles(const Game *g)
{
    const SpriteDef *arrow = GetSprite("item_arrow");
    for (int i = 0; i < MAX_PROJECTILES; i++) {
        const Projectile *p = &g->projectiles[i];
        if (!p->active) continue;
        if (p->faction == FACTION_PLAYER && arrow) {
            // Sprite points up; +90 aligns it with the velocity angle
            float ang = atan2f(p->vel.y, p->vel.x) * RAD2DEG + 90.0f;
            DrawTexturePro(g_assets.sheets[arrow->sheet], arrow->src,
                           (Rectangle){ p->pos.x, p->pos.y, 12, 12 },
                           (Vector2){ 6, 6 }, ang, WHITE);
        } else {
            DrawRectangle((int)(p->pos.x - 2), (int)(p->pos.y - 2), 4, 4,
                          (Color){ 235, 225, 185, 255 });
        }
    }
}

static void DrawSwordSwing(const Game *g)
{
    if (g->player.swingT <= 0) return;
    float progress = 1.0f - g->player.swingT / SWING_TIME;
    float angle = (g->player.facing > 0) ? -60.0f + 120.0f * progress
                                         : 240.0f - 120.0f * progress;
    Vector2 hand = { g->player.box.x + g->player.box.width / 2,
                     g->player.box.y + g->player.box.height / 2 };
    const SpriteDef *s = GetSprite("item_sword");
    if (s) {
        // Sprite points up; +90 makes rotation 0 point along +x like the rect did
        DrawTexturePro(g_assets.sheets[s->sheet], s->src,
                       (Rectangle){ hand.x, hand.y, 16, 16 },
                       (Vector2){ 3, 13 }, angle + 90.0f, WHITE);
    } else {
        DrawRectanglePro((Rectangle){ hand.x, hand.y, 16, 3 },
                         (Vector2){ 0, 1.5f }, angle, LIGHTGRAY);
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
            Rectangle icon = { x + 8, y0 + 8, slot - 16, slot - 16 };
            const SpriteDef *spr = ItemSprite(s.id);
            if (spr) DrawSpriteRect(spr, icon, WHITE);
            else DrawRectangleRec(icon, ItemColor(s.id));
            if (s.count > 1)
                DrawText(TextFormat("%d", s.count), x + 4, y0 + slot - 13, 10, RAYWHITE);
        }
        DrawText(TextFormat("%d", i + 1), x + 3, y0 + 2, 10, (Color){ 255, 255, 255, 140 });
    }
}

void RenderGame(const Game *g)
{
    ClearBackground((Color){ 108, 158, 222, 255 }); // sky

    if (g->state == STATE_MENU) {
        const char *title = "TERRA";
        DrawText(title, (GetScreenWidth() - MeasureText(title, 60)) / 2, 240, 60, RAYWHITE);
        const char *hint = "press ENTER to play";
        DrawText(hint, (GetScreenWidth() - MeasureText(hint, 20)) / 2, 330, 20,
                 (Color){ 255, 255, 255, 200 });
        return;
    }

    BeginMode2D(g->camera);

    DrawWorld(g);

    for (int i = 0; i < MAX_DROPS; i++) {
        if (!g->drops[i].active) continue;
        const SpriteDef *spr = ItemSprite(g->drops[i].item);
        if (spr) DrawSpriteRect(spr, g->drops[i].box, WHITE);
        else DrawRectangleRec(g->drops[i].box, ItemColor(g->drops[i].item));
    }

    DrawEnemies(g);
    DrawProjectiles(g);
    DrawPlayer(g);
    DrawSwordSwing(g);

    for (int i = 0; i < MAX_PARTICLES; i++) {
        const Particle *p = &g->particles[i];
        if (!p->active) continue;
        Color c = p->color;
        c.a = (unsigned char)(255.0f * (p->life / p->maxLife)); // fade out
        DrawRectangle((int)(p->pos.x - 1), (int)(p->pos.y - 1), 3, 3, c);
    }

    if (g->aimInReach) {
        DrawRectangleLinesEx(
            (Rectangle){ g->aimX * TILE_SIZE, g->aimY * TILE_SIZE, TILE_SIZE, TILE_SIZE },
            1, WHITE);
    }

    EndMode2D();

    if (g->state == STATE_PAUSED) {
        DrawRectangle(0, 0, GetScreenWidth(), GetScreenHeight(), (Color){ 0, 0, 0, 120 });
        const char *t = "PAUSED";
        DrawText(t, (GetScreenWidth() - MeasureText(t, 40)) / 2, 320, 40, RAYWHITE);
    }

    // Hearts: 10 hearts x 10 HP
    for (int i = 0; i < 10; i++) {
        Color c = (g->player.hp > i * 10) ? (Color){ 220, 60, 60, 255 }
                                          : (Color){ 60, 60, 60, 180 };
        DrawRectangle(10 + i * 18, 34, 14, 14, c);
    }

    DrawHotbar(g);
    DrawFPS(10, 10);
}
