#ifndef TERRA_RENDER_H
#define TERRA_RENDER_H

#include "game.h"

void RenderGame(const Game *g);

Color TileColor(uint8_t t);       // placeholder palette (also used by particles)
Color EnemyColorOf(uint8_t type);

#endif // TERRA_RENDER_H
