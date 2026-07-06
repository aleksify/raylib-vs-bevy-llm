#include "entities.h"
#include "world.h"
#include "noise.h"
#include "inventory.h"
#include <math.h>

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
