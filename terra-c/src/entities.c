#include "entities.h"
#include "world.h"
#include "worldgen.h"
#include "noise.h"
#include "inventory.h"
#include "combat.h"
#include <math.h>

// hp / contact damage / AABB size, indexed by EnemyType
static const int ENEMY_HP[3] = { 30, 50, 20 };
static const int ENEMY_CONTACT[3] = { 10, 15, 8 };
static const Vector2 ENEMY_SIZE[3] = { { 14, 12 }, { 12, 22 }, { 12, 10 } };

void SpawnDrop(Game *g, int tx, int ty, uint8_t item)
{
    for (int i = 0; i < MAX_DROPS; i++) {
        Drop *d = &g->drops[i];
        if (d->active) continue;
        d->active = true;
        d->item = item;
        d->box = (Rectangle){ tx * TILE_SIZE + 4.0f, ty * TILE_SIZE + 4.0f, 8, 8 };
        d->vel = (Vector2){ (float)RngRange(&g->rng, -40, 40), -120.0f };
        return;
    }
}

void UpdateDrops(Game *g, float dt)
{
    Vector2 pc = {
        g->player.box.x + g->player.box.width / 2,
        g->player.box.y + g->player.box.height / 2,
    };
    for (int i = 0; i < MAX_DROPS; i++) {
        Drop *d = &g->drops[i];
        if (!d->active) continue;

        Vector2 dc = { d->box.x + 4, d->box.y + 4 };
        float dx = pc.x - dc.x, dy = pc.y - dc.y;
        float dist = sqrtf(dx * dx + dy * dy);

        if (dist < DROP_HOMING_RANGE && dist > 1.0f) {
            // Home to the player, ignore gravity
            d->vel.x = dx / dist * 180.0f;
            d->vel.y = dy / dist * 180.0f;
        } else {
            d->vel.y += GRAVITY * dt;
            // Air drag so thrown drops settle instead of sliding forever
            d->vel.x *= 0.98f;
        }

        float preVy = d->vel.y;
        bool grounded = false;
        MoveAndCollide(&g->world, &d->box, &d->vel, dt, &grounded);
        if (grounded && preVy > 80.0f) d->vel.y = -preVy * 0.4f; // bounce

        if (CheckCollisionRecs(d->box, g->player.box)) {
            if (InvAdd(&g->inv, d->item, 1)) d->active = false;
        }
    }
}

static void SpawnEnemy(Game *g, uint8_t type, float x, float y)
{
    for (int i = 0; i < MAX_ENEMIES; i++) {
        Enemy *e = &g->enemies[i];
        if (e->active) continue;
        *e = (Enemy){
            .active = true,
            .type = type,
            .box = { x, y, ENEMY_SIZE[type].x, ENEMY_SIZE[type].y },
            .hp = ENEMY_HP[type],
        };
        return;
    }
}

void UpdateEnemySpawner(Game *g, float dt)
{
    g->enemySpawnT -= dt;
    if (g->enemySpawnT > 0) return;
    g->enemySpawnT = ENEMY_SPAWN_INTERVAL;

    int alive = 0;
    for (int i = 0; i < MAX_ENEMIES; i++) alive += g->enemies[i].active;
    if (alive >= MAX_ENEMIES) return;

    // Surface tile 20-40 tiles from the player; >=20 tiles is off-screen at zoom 2
    int playerTx = (int)((g->player.box.x + g->player.box.width / 2) / TILE_SIZE);
    int dist = RngRange(&g->rng, ENEMY_MIN_SPAWN_TILES, ENEMY_MAX_SPAWN_TILES);
    int dir = RngRange(&g->rng, 0, 1) ? 1 : -1;
    int tx = playerTx + dir * dist;
    if (tx < 1 || tx >= WORLD_W - 1) return;
    int sy = WorldSurfaceY(&g->world, tx);

    uint8_t type = (uint8_t)RngRange(&g->rng, 0, 2);
    float x = tx * TILE_SIZE + (TILE_SIZE - ENEMY_SIZE[type].x) / 2;
    float y = (type == ENEMY_BEE)
        ? (sy - RngRange(&g->rng, 3, 6)) * TILE_SIZE   // bees start airborne
        : sy * TILE_SIZE - ENEMY_SIZE[type].y;
    SpawnEnemy(g, type, x, y);
}

void UpdateEnemies(Game *g, float dt)
{
    Vector2 pc = {
        g->player.box.x + g->player.box.width / 2,
        g->player.box.y + g->player.box.height / 2,
    };

    for (int i = 0; i < MAX_ENEMIES; i++) {
        Enemy *e = &g->enemies[i];
        if (!e->active) continue;

        Vector2 ec = { e->box.x + e->box.width / 2, e->box.y + e->box.height / 2 };
        float dx = pc.x - ec.x, dy = pc.y - ec.y;
        float dir = (dx >= 0) ? 1.0f : -1.0f;

        if (fabsf(dx) > ENEMY_DESPAWN_TILES * TILE_SIZE) { e->active = false; continue; }

        switch (e->type) {
        case ENEMY_SLIME:
            e->vel.y += GRAVITY * dt;
            if (e->grounded) {
                e->vel.x = 0;
                e->aiTimer -= dt;
                if (e->aiTimer <= 0) {
                    e->aiTimer = 1.5f;
                    e->vel.y = -260.0f;      // hop toward the player
                    e->vel.x = dir * 90.0f;
                }
            }
            break;
        case ENEMY_ZOMBIE: {
            e->vel.y += GRAVITY * dt;
            e->vel.x = dir * 50.0f;
            break;
        }
        case ENEMY_BEE: {
            // No gravity: drift toward the player with a sine-wave wobble
            e->phase += dt;
            float len = sqrtf(dx * dx + dy * dy);
            if (len > 1.0f) {
                e->vel.x = dx / len * 60.0f;
                e->vel.y = dy / len * 60.0f + sinf(e->phase * 5.0f) * 40.0f;
            }
            e->aiTimer -= dt;
            if (e->aiTimer <= 0 && len < 20 * TILE_SIZE) {
                e->aiTimer = 2.0f;
                SpawnProjectile(g, ec,
                    (Vector2){ dx / len * 150.0f, dy / len * 150.0f },
                    8, FACTION_ENEMY, 0.0f); // slow, straight shot
            }
            break;
        }
        }

        float wantVx = e->vel.x;
        MoveAndCollide(&g->world, &e->box, &e->vel, dt, &e->grounded);

        // Zombie blocked by a wall while walking -> jump it
        if (e->type == ENEMY_ZOMBIE && e->grounded &&
            wantVx != 0 && e->vel.x == 0) {
            e->vel.y = -260.0f;
        }

        if (CheckCollisionRecs(e->box, g->player.box))
            PlayerHurt(g, ENEMY_CONTACT[e->type], ec.x);
    }
}
