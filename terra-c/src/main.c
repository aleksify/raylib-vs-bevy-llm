#include "raylib.h"
#include "game.h"
#include "world.h"
#include "worldgen.h"
#include "player.h"
#include "mining.h"
#include "entities.h"
#include "inventory.h"
#include "render.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>

#define DEFAULT_SEED 1337ULL

static InputFrame GatherInput(void)
{
    InputFrame in = { 0 };
    if (IsKeyDown(KEY_A) || IsKeyDown(KEY_LEFT))  in.move -= 1.0f;
    if (IsKeyDown(KEY_D) || IsKeyDown(KEY_RIGHT)) in.move += 1.0f;
    in.jumpPressed = IsKeyPressed(KEY_SPACE);
    in.jumpDown = IsKeyDown(KEY_SPACE);
    in.mineHeld = IsMouseButtonDown(MOUSE_BUTTON_LEFT);
    in.placeHeld = IsMouseButtonDown(MOUSE_BUTTON_RIGHT);
    return in;
}

static void UpdateHotbarSelection(Inventory *inv)
{
    for (int i = 0; i < INV_SLOTS; i++)
        if (IsKeyPressed(KEY_ONE + i)) inv->selected = i;
    float wheel = GetMouseWheelMove();
    if (wheel > 0) inv->selected = (inv->selected + INV_SLOTS - 1) % INV_SLOTS;
    else if (wheel < 0) inv->selected = (inv->selected + 1) % INV_SLOTS;
}

int main(int argc, char **argv)
{
    uint64_t seed = DEFAULT_SEED;
    for (int i = 1; i < argc - 1; i++) {
        if (strcmp(argv[i], "--dump-seed") == 0) {
            // Worldgen parity check vs terra-rs: print hash, no window
            World w = { 0 };
            WorldGenerate(&w, strtoull(argv[i + 1], NULL, 10));
            printf("%016" PRIx64 "\n", WorldHash(&w));
            WorldFree(&w);
            return 0;
        }
        if (strcmp(argv[i], "--seed") == 0) seed = strtoull(argv[i + 1], NULL, 10);
    }

    SetConfigFlags(FLAG_VSYNC_HINT);
    InitWindow(WINDOW_W, WINDOW_H, "terra (raylib)");

    Game game = { 0 };
    WorldGenerate(&game.world, seed);
    InvInit(&game.inv);
    game.rng = seed ^ 0xDEADBEEFULL; // gameplay stream, separate from worldgen

    // Spawn on the generated surface at world center
    int spawnCol = WORLD_W / 2;
    float spawnX = spawnCol * TILE_SIZE;
    float spawnY = WorldSurfaceY(&game.world, spawnCol) * TILE_SIZE - PLAYER_BOX_H;
    PlayerInit(&game.player, (Vector2){ spawnX, spawnY });

    game.camera = (Camera2D){
        .offset = { WINDOW_W / 2.0f, WINDOW_H / 2.0f },
        .zoom = CAMERA_ZOOM,
    };

    float acc = 0.0f;
    while (!WindowShouldClose()) {
        acc += GetFrameTime();
        if (acc > 0.25f) acc = 0.25f; // avoid spiral of death after stalls

        InputFrame in = GatherInput();
        UpdateHotbarSelection(&game.inv);
        UpdateAim(&game);
        while (acc >= FIXED_DT) {
            PlayerUpdate(&game.player, &game.world, &in, FIXED_DT);
            UpdateMining(&game, &in, FIXED_DT);
            UpdateDrops(&game, FIXED_DT);
            in.jumpPressed = false; // edge consumed by first step
            acc -= FIXED_DT;
        }

        // Whole-pixel camera target: no sprite seams/shimmer at zoom 2
        game.camera.target = (Vector2){
            roundf(game.player.box.x + game.player.box.width / 2),
            roundf(game.player.box.y + game.player.box.height / 2),
        };

        BeginDrawing();
        RenderGame(&game);
        EndDrawing();
    }

    WorldFree(&game.world);
    CloseWindow();
    return 0;
}
