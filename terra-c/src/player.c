#include "player.h"
#include "world.h"

void PlayerInit(Player *p, Vector2 spawnTopLeft)
{
    p->box = (Rectangle){ spawnTopLeft.x, spawnTopLeft.y, PLAYER_BOX_W, PLAYER_BOX_H };
    p->vel = (Vector2){ 0, 0 };
    p->grounded = false;
    p->hp = PLAYER_MAX_HP;
}

void PlayerUpdate(Player *p, const World *w, const InputFrame *in, float dt)
{
    p->vel.x = in->move * PLAYER_SPEED;
    p->vel.y += GRAVITY * dt;

    if (in->jumpPressed && p->grounded) {
        p->vel.y = PLAYER_JUMP_VEL;
        p->grounded = false;
    }

    MoveAndCollide(w, &p->box, &p->vel, dt, &p->grounded);
}
