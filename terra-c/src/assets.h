#ifndef TERRA_ASSETS_H
#define TERRA_ASSETS_H

#include "raylib.h"
#include <stdbool.h>

#define MAX_SHEETS 4
#define MAX_SPRITES 32

typedef struct SpriteDef {
    char name[32];
    int sheet;      // index into Assets.sheets
    Rectangle src;
} SpriteDef;

typedef struct Assets {
    Texture2D sheets[MAX_SHEETS];
    char sheetPaths[MAX_SHEETS][160];
    int sheetCount;
    SpriteDef sprites[MAX_SPRITES];
    int spriteCount;
    bool loaded;
} Assets;

// The one allowed global (per CLAUDE.md): loaded Kenney atlas sheets.
extern Assets g_assets;

void LoadAssets(void); // reads ../assets/atlas_map.txt; g_assets.loaded=false on failure
const SpriteDef *GetSprite(const char *name); // NULL if missing (callers fall back to rects)
void UnloadAssets(void);

#endif // TERRA_ASSETS_H
