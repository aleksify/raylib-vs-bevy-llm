#include "assets.h"
#include <stdio.h>
#include <string.h>

Assets g_assets = { 0 };

static int SheetIndex(const char *baseDir, const char *rel)
{
    for (int i = 0; i < g_assets.sheetCount; i++)
        if (strcmp(g_assets.sheetPaths[i], rel) == 0) return i;
    if (g_assets.sheetCount == MAX_SHEETS) return -1;
    int i = g_assets.sheetCount++;
    snprintf(g_assets.sheetPaths[i], sizeof(g_assets.sheetPaths[i]), "%s", rel);
    char full[320];
    snprintf(full, sizeof(full), "%s/%s", baseDir, rel);
    g_assets.sheets[i] = LoadTexture(full);
    SetTextureFilter(g_assets.sheets[i], TEXTURE_FILTER_POINT); // pixel art
    return (g_assets.sheets[i].id != 0) ? i : -1;
}

void LoadAssets(void)
{
    const char *baseDir = "../assets";
    char mapPath[320];
    snprintf(mapPath, sizeof(mapPath), "%s/atlas_map.txt", baseDir);
    FILE *f = fopen(mapPath, "r");
    if (!f) { // also support running from the repo root
        baseDir = "assets";
        snprintf(mapPath, sizeof(mapPath), "%s/atlas_map.txt", baseDir);
        f = fopen(mapPath, "r");
    }
    if (!f) {
        TraceLog(LOG_WARNING, "assets: atlas_map.txt not found, using colored rects");
        return;
    }

    char line[512];
    while (fgets(line, sizeof(line), f)) {
        char name[32], rel[160];
        int x, y, w, h;
        if (line[0] == '#' || sscanf(line, "%31s %159s %d %d %d %d",
                                     name, rel, &x, &y, &w, &h) != 6) continue;
        if (g_assets.spriteCount == MAX_SPRITES) break;
        int sheet = SheetIndex(baseDir, rel);
        if (sheet < 0) continue;
        SpriteDef *s = &g_assets.sprites[g_assets.spriteCount++];
        snprintf(s->name, sizeof(s->name), "%s", name);
        s->sheet = sheet;
        s->src = (Rectangle){ (float)x, (float)y, (float)w, (float)h };
    }
    fclose(f);
    g_assets.loaded = g_assets.spriteCount > 0;
    TraceLog(LOG_INFO, "assets: %d sprites from %d sheets",
             g_assets.spriteCount, g_assets.sheetCount);
}

const SpriteDef *GetSprite(const char *name)
{
    if (!g_assets.loaded) return NULL;
    for (int i = 0; i < g_assets.spriteCount; i++)
        if (strcmp(g_assets.sprites[i].name, name) == 0) return &g_assets.sprites[i];
    return NULL;
}

void UnloadAssets(void)
{
    for (int i = 0; i < g_assets.sheetCount; i++) UnloadTexture(g_assets.sheets[i]);
    g_assets.sheetCount = g_assets.spriteCount = 0;
    g_assets.loaded = false;
}
