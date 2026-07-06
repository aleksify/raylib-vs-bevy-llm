#ifndef TERRA_MINING_H
#define TERRA_MINING_H

#include "game.h"

void UpdateAim(Game *g);                                    // per render frame
void UpdateMining(Game *g, const InputFrame *in, float dt); // per fixed step

#endif // TERRA_MINING_H
