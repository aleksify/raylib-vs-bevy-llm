#include "combat.h"
#include "world.h"
#include <math.h>

void SpawnProjectile(Game *g, Vector2 pos, Vector2 vel, int dmg,
                     Faction faction, float gravityFactor)
{
    for (int i = 0; i < MAX_PROJECTILES; i++) {
        Projectile *p = &g->projectiles[i];
        if (p->active) continue;
        *p = (Projectile){ true, pos, vel, dmg, (uint8_t)faction,
                           gravityFactor, ARROW_LIFETIME };
        return;
    }
}

void PlayerHurt(Game *g, int dmg, float fromX)
{
    Player *p = &g->player;
    if (p->invulnT > 0) return;
    p->invulnT = HURT_INVULN;
    p->hp -= dmg;
    float pcx = p->box.x + p->box.width / 2;
    p->vel.x = (pcx < fromX) ? -160.0f : 160.0f;
    p->vel.y = -160.0f;
    if (p->hp <= 0) {
        // Death: respawn at world spawn, restore HP
        p->hp = PLAYER_MAX_HP;
        p->box.x = g->spawnPos.x;
        p->box.y = g->spawnPos.y;
        p->vel = (Vector2){ 0, 0 };
        p->invulnT = 1.0f;
    }
}

static void HitEnemy(Enemy *e, int dmg, float kbDir)
{
    e->hp -= dmg;
    e->vel.x = kbDir * 180.0f;
    e->vel.y = -120.0f;
    e->hurtFlash = 0.15f;
    if (e->hp <= 0) e->active = false; // TODO(M5): death poof + SFX
}

void UpdateCombat(Game *g, const InputFrame *in, float dt)
{
    Player *p = &g->player;
    if (in->move > 0) p->facing = 1;
    else if (in->move < 0) p->facing = -1;
    if (p->invulnT > 0) p->invulnT -= dt;
    if (p->bowCd > 0) p->bowCd -= dt;
    for (int i = 0; i < MAX_ENEMIES; i++)
        if (g->enemies[i].hurtFlash > 0) g->enemies[i].hurtFlash -= dt;

    uint8_t held = g->inv.slots[g->inv.selected].id;
    Vector2 pc = { p->box.x + p->box.width / 2, p->box.y + p->box.height / 2 };

    if (held == ITEM_SWORD && in->mineHeld && p->swingT <= 0) {
        p->swingT = SWING_TIME;
        p->swingHit = 0;
        p->facing = (g->aimWorld.x >= pc.x) ? 1 : -1; // swing toward mouse
    }

    if (held == ITEM_BOW && in->mineHeld && p->bowCd <= 0) {
        p->bowCd = BOW_COOLDOWN;
        Vector2 d = { g->aimWorld.x - pc.x, g->aimWorld.y - pc.y };
        float len = sqrtf(d.x * d.x + d.y * d.y);
        if (len < 1.0f) { d = (Vector2){ (float)p->facing, 0 }; len = 1.0f; }
        SpawnProjectile(g, pc,
            (Vector2){ d.x / len * ARROW_SPEED, d.y / len * ARROW_SPEED },
            ARROW_DMG, FACTION_PLAYER, ARROW_GRAVITY);
    }

    if (p->swingT > 0) {
        p->swingT -= dt;
        Rectangle hb = {
            (p->facing > 0) ? p->box.x + p->box.width : p->box.x - SWORD_HITBOX,
            pc.y - SWORD_HITBOX / 2,
            SWORD_HITBOX, SWORD_HITBOX,
        };
        for (int i = 0; i < MAX_ENEMIES; i++) {
            Enemy *e = &g->enemies[i];
            if (!e->active || (p->swingHit & (1u << i))) continue;
            if (!CheckCollisionRecs(hb, e->box)) continue;
            p->swingHit |= (1u << i); // one swing hits each enemy once
            HitEnemy(e, SWORD_DMG, (float)p->facing);
        }
    }
}

void UpdateProjectiles(Game *g, float dt)
{
    for (int i = 0; i < MAX_PROJECTILES; i++) {
        Projectile *pr = &g->projectiles[i];
        if (!pr->active) continue;

        pr->lifetime -= dt;
        if (pr->lifetime <= 0) { pr->active = false; continue; }

        pr->vel.y += GRAVITY * pr->gravityFactor * dt;
        pr->pos.x += pr->vel.x * dt;
        pr->pos.y += pr->vel.y * dt;

        int tx = (int)floorf(pr->pos.x / TILE_SIZE);
        int ty = (int)floorf(pr->pos.y / TILE_SIZE);
        if (SolidAt(&g->world, tx, ty)) { pr->active = false; continue; }

        if (pr->faction == FACTION_PLAYER) {
            for (int e = 0; e < MAX_ENEMIES; e++) {
                Enemy *en = &g->enemies[e];
                if (!en->active || !CheckCollisionPointRec(pr->pos, en->box)) continue;
                HitEnemy(en, pr->dmg, pr->vel.x >= 0 ? 1.0f : -1.0f);
                pr->active = false;
                break;
            }
        } else {
            if (CheckCollisionPointRec(pr->pos, g->player.box)) {
                PlayerHurt(g, pr->dmg, pr->pos.x);
                pr->active = false;
            }
        }
    }
}
