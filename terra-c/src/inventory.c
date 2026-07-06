#include "inventory.h"
#include <string.h>

void InvInit(Inventory *inv)
{
    memset(inv, 0, sizeof(*inv));
    inv->slots[0] = (ItemSlot){ ITEM_SWORD, 1 };
    inv->slots[1] = (ItemSlot){ ITEM_BOW, 1 };
    inv->selected = 0;
}

bool InvAdd(Inventory *inv, uint8_t item, int count)
{
    // Top up an existing stack first
    for (int i = 0; i < INV_SLOTS; i++) {
        ItemSlot *s = &inv->slots[i];
        if (s->id == item && s->count < STACK_MAX) {
            int space = STACK_MAX - s->count;
            int add = count < space ? count : space;
            s->count += add;
            count -= add;
            if (count == 0) return true;
        }
    }
    for (int i = 0; i < INV_SLOTS; i++) {
        ItemSlot *s = &inv->slots[i];
        if (s->id == ITEM_NONE) {
            s->id = item;
            s->count = (uint16_t)(count < STACK_MAX ? count : STACK_MAX);
            return true;
        }
    }
    return false;
}
