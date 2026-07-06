#include "mining.h"
#include "world.h"
#include "entities.h"
#include <math.h>

// Hits to break, by tile id (air, dirt, grass, stone, wood, leaves, ore)
static const int HARDNESS[TILE_COUNT] = { 0, 2, 2, 4, 3, 1, 4 };

static uint8_t DropForTile(uint8_t t)
{
    if (t == TILE_GRASS) return TILE_DIRT;
    if (t == TILE_LEAVES) return ITEM_NONE;
    return t;
}

void UpdateAim(Game *g)
{
    Vector2 m = GetScreenToWorld2D(GetMousePosition(), g->camera);
    g->aimWorld = m;
    g->aimX = (int)floorf(m.x / TILE_SIZE);
    g->aimY = (int)floorf(m.y / TILE_SIZE);
    float px = g->player.box.x + g->player.box.width / 2;
    float py = g->player.box.y + g->player.box.height / 2;
    float tx = g->aimX * TILE_SIZE + TILE_SIZE / 2.0f;
    float ty = g->aimY * TILE_SIZE + TILE_SIZE / 2.0f;
    float reach = PLAYER_REACH * TILE_SIZE;
    g->aimInReach = (px - tx) * (px - tx) + (py - ty) * (py - ty) <= reach * reach;
}

static DamageEntry *FindDamageEntry(Game *g, int x, int y)
{
    for (int i = 0; i < MAX_DAMAGE_ENTRIES; i++) {
        DamageEntry *e = &g->dmgTable[i];
        if (e->used && e->x == x && e->y == y) return e;
    }
    for (int i = 0; i < MAX_DAMAGE_ENTRIES; i++) {
        DamageEntry *e = &g->dmgTable[i];
        if (!e->used) {
            *e = (DamageEntry){ true, x, y, 0 };
            return e;
        }
    }
    // Table full: forget old targets, reuse slot 0
    for (int i = 0; i < MAX_DAMAGE_ENTRIES; i++) g->dmgTable[i].used = false;
    g->dmgTable[0] = (DamageEntry){ true, x, y, 0 };
    return &g->dmgTable[0];
}

static void TryMine(Game *g)
{
    uint8_t t = TileAt(&g->world, g->aimX, g->aimY);
    if (t == TILE_AIR) return;
    g->mineCd = MINE_COOLDOWN;

    DamageEntry *e = FindDamageEntry(g, g->aimX, g->aimY);
    e->dmg++;
    if (e->dmg < HARDNESS[t]) return;

    e->used = false;
    SetTile(&g->world, g->aimX, g->aimY, TILE_AIR);
    uint8_t drop = DropForTile(t);
    if (drop != ITEM_NONE) SpawnDrop(g, g->aimX, g->aimY, drop);
}

static void TryPlace(Game *g)
{
    ItemSlot *s = &g->inv.slots[g->inv.selected];
    if (s->id == ITEM_NONE || s->id > TILE_ORE || s->count == 0) return; // not a block
    if (TileAt(&g->world, g->aimX, g->aimY) != TILE_AIR) return;
    if (!SolidAt(&g->world, g->aimX + 1, g->aimY) &&
        !SolidAt(&g->world, g->aimX - 1, g->aimY) &&
        !SolidAt(&g->world, g->aimX, g->aimY + 1) &&
        !SolidAt(&g->world, g->aimX, g->aimY - 1)) return;
    Rectangle tile = { g->aimX * TILE_SIZE, g->aimY * TILE_SIZE, TILE_SIZE, TILE_SIZE };
    if (CheckCollisionRecs(tile, g->player.box)) return;
    // TODO(M5): also reject overlap with enemies

    SetTile(&g->world, g->aimX, g->aimY, s->id);
    g->placeCd = MINE_COOLDOWN;
    if (--s->count == 0) s->id = ITEM_NONE;
}

void UpdateMining(Game *g, const InputFrame *in, float dt)
{
    g->mineCd -= dt;
    g->placeCd -= dt;

    // LMB is context-sensitive: weapons swing/shoot (combat.c), blocks/empty mine
    uint8_t held = g->inv.slots[g->inv.selected].id;
    bool weapon = (held == ITEM_SWORD || held == ITEM_BOW);

    if (!in->mineHeld) g->mineCd = 0;                    // first click is instant
    else if (!weapon && g->mineCd <= 0 && g->aimInReach) TryMine(g);

    if (!in->placeHeld) g->placeCd = 0;
    else if (g->placeCd <= 0 && g->aimInReach) TryPlace(g);
}
