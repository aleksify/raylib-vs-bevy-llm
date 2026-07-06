#include "raylib.h"
#include "game.h"
#include "world.h"
#include "player.h"
#include "render.h"
#include <math.h>

static InputFrame GatherInput(void)
{
    InputFrame in = { 0 };
    if (IsKeyDown(KEY_A) || IsKeyDown(KEY_LEFT))  in.move -= 1.0f;
    if (IsKeyDown(KEY_D) || IsKeyDown(KEY_RIGHT)) in.move += 1.0f;
    in.jumpPressed = IsKeyPressed(KEY_SPACE);
    in.jumpDown = IsKeyDown(KEY_SPACE);
    return in;
}

int main(void)
{
    SetConfigFlags(FLAG_VSYNC_HINT);
    InitWindow(WINDOW_W, WINDOW_H, "terra (raylib)");

    Game game = { 0 };
    WorldInitFlat(&game.world);

    // Spawn on the surface at world center
    float spawnX = (WORLD_W / 2) * TILE_SIZE;
    float spawnY = (WORLD_H / 2) * TILE_SIZE - PLAYER_BOX_H;
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
        while (acc >= FIXED_DT) {
            PlayerUpdate(&game.player, &game.world, &in, FIXED_DT);
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
