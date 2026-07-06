#ifndef TERRA_ENTITIES_H
#define TERRA_ENTITIES_H

#include "game.h"

void SpawnDrop(Game *g, int tx, int ty, uint8_t item);
void UpdateDrops(Game *g, float dt);

#endif // TERRA_ENTITIES_H
