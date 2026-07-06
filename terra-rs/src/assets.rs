use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Sprites parsed from the shared ../assets/atlas_map.txt (same file terra-c
/// reads). Missing file -> empty map, callers fall back to colored rects.
#[derive(Resource, Default)]
pub struct GameAssets {
    sheets: HashMap<String, Handle<Image>>,
    sprites: HashMap<String, (String, Rect)>,
}

impl GameAssets {
    /// Sprite for `name` at `size` px, or None (caller falls back to a rect)
    pub fn sprite(&self, name: &str, size: Vec2) -> Option<Sprite> {
        let (sheet, rect) = self.sprites.get(name)?;
        let image = self.sheets.get(sheet)?.clone();
        Some(Sprite {
            image,
            rect: Some(*rect),
            custom_size: Some(size),
            ..default()
        })
    }

    pub fn tile_sprite(&self, t: u8, size: Vec2) -> Option<Sprite> {
        const NAMES: [&str; 7] = [
            "", "tile_dirt", "tile_grass", "tile_stone",
            "tile_wood", "tile_leaves", "tile_ore",
        ];
        self.sprite(NAMES.get(t as usize)?, size)
    }
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        // PreStartup so every Startup spawner (player, hotbar) sees the atlas
        app.init_resource::<GameAssets>()
            .add_systems(PreStartup, load_assets);
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut assets: ResMut<GameAssets>) {
    // AssetPlugin file_path is ../assets, but the map itself is parsed with
    // std::fs (synchronously, before any spawns)
    let Ok(text) = std::fs::read_to_string("../assets/atlas_map.txt") else {
        warn!("assets: atlas_map.txt not found, using colored rects");
        return;
    };
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 6 || f[0].starts_with('#') {
            continue;
        }
        let (Ok(x), Ok(y), Ok(w), Ok(h)) =
            (f[2].parse::<f32>(), f[3].parse::<f32>(), f[4].parse::<f32>(), f[5].parse::<f32>())
        else {
            continue;
        };
        let sheet = f[1].to_string();
        assets
            .sheets
            .entry(sheet.clone())
            .or_insert_with(|| asset_server.load(&sheet));
        assets.sprites.insert(
            f[0].to_string(),
            (sheet, Rect::new(x, y, x + w, y + h)),
        );
    }
    info!("assets: {} sprites from {} sheets", assets.sprites.len(), assets.sheets.len());
}
