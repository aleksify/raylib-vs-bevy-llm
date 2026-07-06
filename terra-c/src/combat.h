#ifndef TERRA_COMBAT_H
#define TERRA_COMBAT_H

#include "game.h"

void UpdateCombat(Game *g, const InputFrame *in, float dt);   // sword + bow
void UpdateProjectiles(Game *g, float dt);
void SpawnProjectile(Game *g, Vector2 pos, Vector2 vel, int dmg,
                     Faction faction, float gravityFactor);
void PlayerHurt(Game *g, int dmg, float fromX); // knockback away from fromX

#endif // TERRA_COMBAT_H
