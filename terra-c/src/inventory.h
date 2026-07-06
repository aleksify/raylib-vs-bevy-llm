#ifndef TERRA_INVENTORY_H
#define TERRA_INVENTORY_H

#include "game.h"

void InvInit(Inventory *inv);                       // sword slot 1, bow slot 2
bool InvAdd(Inventory *inv, uint8_t item, int count); // false if full

#endif // TERRA_INVENTORY_H
