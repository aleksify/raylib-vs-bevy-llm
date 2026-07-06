#ifndef TERRA_GAME_H
#define TERRA_GAME_H

#include "raylib.h"
#include <stdint.h>
#include <stdbool.h>

// Shared constants — must match terra-rs/src/consts.rs exactly
#define TILE_SIZE        16
#define WORLD_W          1024
#define WORLD_H          256
#define CHUNK_SIZE       32
#define FIXED_DT         (1.0f/60.0f)
#define GRAVITY          900.0f
#define PLAYER_SPEED     140.0f
#define PLAYER_JUMP_VEL  (-320.0f)
#define PLAYER_REACH     5
#define PLAYER_MAX_HP    100
#define WINDOW_W         1280
#define WINDOW_H         720
#define CAMERA_ZOOM      2.0f

#define PLAYER_BOX_W     12.0f
#define PLAYER_BOX_H     22.0f

typedef enum {
    TILE_AIR = 0,
    TILE_DIRT,
    TILE_GRASS,
    TILE_STONE,
    TILE_WOOD,
    TILE_LEAVES,
    TILE_ORE,
    TILE_COUNT
} TileId;

typedef struct World {
    uint8_t *tiles; // WORLD_W * WORLD_H, row-major, y*WORLD_W + x, y grows down
} World;

typedef struct Player {
    Rectangle box;   // AABB in world pixels, top-left origin
    Vector2 vel;
    bool grounded;
    int hp;
} Player;

// Input edges are latched once per render frame and fed to every fixed step,
// because IsKeyPressed() is per-frame and multiple fixed steps can run per frame.
typedef struct InputFrame {
    float move;        // -1..1
    bool jumpPressed;
    bool jumpDown;
} InputFrame;

typedef struct Game {
    World world;
    Player player;
    Camera2D camera;
} Game;

#endif // TERRA_GAME_H
