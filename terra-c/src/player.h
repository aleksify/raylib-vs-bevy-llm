#ifndef TERRA_PLAYER_H
#define TERRA_PLAYER_H

#include "game.h"

void PlayerInit(Player *p, Vector2 spawnTopLeft);
void PlayerUpdate(Player *p, const World *w, const InputFrame *in, float dt);

#endif // TERRA_PLAYER_H
