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

#define MAX_DROPS        128
#define MAX_DAMAGE_ENTRIES 8
#define INV_SLOTS        8
#define STACK_MAX        999
#define MINE_COOLDOWN    0.25f
#define DROP_HOMING_RANGE 32.0f   // 2 tiles

#define ITEM_NONE        0
// item ids 1..6 == tile ids (placeable blocks)
#define ITEM_SWORD       100
#define ITEM_BOW         101

#define MAX_ENEMIES      8
#define MAX_PROJECTILES  64
#define SWING_TIME       0.3f
#define SWORD_DMG        20
#define SWORD_HITBOX     24.0f
#define BOW_COOLDOWN     0.5f
#define ARROW_SPEED      400.0f
#define ARROW_DMG        15
#define ARROW_LIFETIME   3.0f
#define ARROW_GRAVITY    0.5f   // gravity factor while flying
#define HURT_INVULN      0.5f

typedef enum {
    FACTION_PLAYER = 0,
    FACTION_ENEMY = 1,
} Faction;

typedef enum {
    ENEMY_SLIME = 0,
    ENEMY_ZOMBIE = 1,
    ENEMY_BEE = 2,
} EnemyType;

#define ENEMY_SPAWN_INTERVAL 3.0f
#define ENEMY_DESPAWN_TILES  80
#define ENEMY_MIN_SPAWN_TILES 20
#define ENEMY_MAX_SPAWN_TILES 40

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
    int facing;      // -1 / 1
    float swingT;    // >0 while sword swing active
    uint8_t swingHit;// per-swing already-hit bitmask over the enemy pool
    float bowCd;
    float invulnT;   // >0 while invulnerable after a hit
} Player;

typedef struct Projectile {
    bool active;
    Vector2 pos;     // center point; collides as a point vs tiles/boxes
    Vector2 vel;
    int dmg;
    uint8_t faction;
    float gravityFactor;
    float lifetime;
} Projectile;

typedef struct Enemy {
    bool active;
    uint8_t type;    // EnemyType
    Rectangle box;
    Vector2 vel;
    int hp;
    bool grounded;
    float hurtFlash;
    float aiTimer;   // slime hop / bee shot cooldown
    float phase;     // bee sine-wave phase
} Enemy;

// Input edges are latched once per render frame and fed to every fixed step,
// because IsKeyPressed() is per-frame and multiple fixed steps can run per frame.
typedef struct InputFrame {
    float move;        // -1..1
    bool jumpPressed;
    bool jumpDown;
    bool mineHeld;     // LMB
    bool placeHeld;    // RMB
} InputFrame;

typedef struct ItemSlot {
    uint8_t id;
    uint16_t count;
} ItemSlot;

typedef struct Inventory {
    ItemSlot slots[INV_SLOTS];
    int selected;
} Inventory;

typedef struct Drop {
    bool active;
    uint8_t item;
    Rectangle box; // 8x8
    Vector2 vel;
} Drop;

typedef struct DamageEntry {
    bool used;
    int x, y;
    int dmg;
} DamageEntry;

typedef struct Game {
    World world;
    Player player;
    Camera2D camera;
    Inventory inv;
    Drop drops[MAX_DROPS];
    Enemy enemies[MAX_ENEMIES];
    Projectile projectiles[MAX_PROJECTILES];
    DamageEntry dmgTable[MAX_DAMAGE_ENTRIES];
    float mineCd, placeCd;
    int aimX, aimY;      // targeted tile (from mouse), updated per frame
    Vector2 aimWorld;    // mouse in world pixels
    bool aimInReach;
    Vector2 spawnPos;    // player respawn point (box top-left)
    float enemySpawnT;   // spawner attempt timer
    uint64_t rng;        // gameplay-only splitmix64 stream (not worldgen)
} Game;

#endif // TERRA_GAME_H
