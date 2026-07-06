use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::consts::*;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Tile {
    Air = 0,
    Dirt,
    Grass,
    Stone,
    Wood,
    Leaves,
    Ore,
}

/// The tile grid lives in a resource, not in entities. Simulation runs in
/// world pixel space with y growing DOWN (matching terra-c); conversion to
/// Bevy's y-up space happens only when writing Transforms.
#[derive(Resource)]
pub struct TileWorld {
    pub tiles: Vec<u8>,
}

impl TileWorld {
    /// OOB -> Stone (solid), kills all edge cases
    pub fn tile_at(&self, tx: i32, ty: i32) -> u8 {
        if tx < 0 || ty < 0 || tx >= WORLD_W || ty >= WORLD_H {
            return Tile::Stone as u8;
        }
        self.tiles[(ty * WORLD_W + tx) as usize]
    }

    pub fn set_tile(&mut self, tx: i32, ty: i32, t: u8) {
        if tx < 0 || ty < 0 || tx >= WORLD_W || ty >= WORLD_H {
            return;
        }
        self.tiles[(ty * WORLD_W + tx) as usize] = t;
    }

    pub fn is_solid(&self, tx: i32, ty: i32) -> bool {
        let t = self.tile_at(tx, ty);
        t != Tile::Air as u8 && t != Tile::Leaves as u8
    }
}

/// Per-axis swept AABB vs the solid-tile grid, identical to terra-c's
/// MoveAndCollide. `pos` is the AABB top-left in world pixels (y-down).
/// Returns `true` when clamped while moving down (grounded).
pub fn move_and_collide(
    world: &TileWorld,
    pos: &mut Vec2,
    size: Vec2,
    vel: &mut Vec2,
    dt: f32,
) -> bool {
    // X axis
    let mut new_x = pos.x + vel.x * dt;
    if vel.x != 0.0 {
        let edge = if vel.x > 0.0 { new_x + size.x } else { new_x };
        let tx = (edge / TILE_SIZE).floor() as i32;
        let ty0 = (pos.y / TILE_SIZE).floor() as i32;
        let ty1 = ((pos.y + size.y - 0.001) / TILE_SIZE).floor() as i32;
        for ty in ty0..=ty1 {
            if world.is_solid(tx, ty) {
                new_x = if vel.x > 0.0 {
                    (tx * TILE_SIZE as i32) as f32 - size.x
                } else {
                    ((tx + 1) * TILE_SIZE as i32) as f32
                };
                vel.x = 0.0;
                break;
            }
        }
    }
    pos.x = new_x;

    // Y axis
    let mut grounded = false;
    let mut new_y = pos.y + vel.y * dt;
    if vel.y != 0.0 {
        let edge = if vel.y > 0.0 { new_y + size.y } else { new_y };
        let ty = (edge / TILE_SIZE).floor() as i32;
        let tx0 = (pos.x / TILE_SIZE).floor() as i32;
        let tx1 = ((pos.x + size.x - 0.001) / TILE_SIZE).floor() as i32;
        for tx in tx0..=tx1 {
            if world.is_solid(tx, ty) {
                if vel.y > 0.0 {
                    new_y = (ty * TILE_SIZE as i32) as f32 - size.y;
                    grounded = true;
                } else {
                    new_y = ((ty + 1) * TILE_SIZE as i32) as f32;
                }
                vel.y = 0.0;
                break;
            }
        }
    }
    pos.y = new_y;

    grounded
}

// Placeholder colors until Kenney atlas lands (M2+)
pub fn tile_color(t: u8) -> Color {
    match t {
        1 => Color::srgb_u8(133, 87, 35),   // Dirt
        2 => Color::srgb_u8(91, 154, 60),   // Grass
        3 => Color::srgb_u8(120, 120, 125), // Stone
        4 => Color::srgb_u8(94, 62, 24),    // Wood
        6 => Color::srgb_u8(155, 105, 185), // Ore
        _ => Color::srgba_u8(118, 190, 88, 180), // Leaves
    }
}

// ---- Chunked rendering ------------------------------------------------
// Entities exist only for tiles in chunks near the camera. Chunks load
// within the view +1 chunk margin and despawn beyond +2 (hysteresis so a
// camera sitting on a boundary doesn't thrash).

#[derive(Resource, Default)]
pub struct ChunkMap(pub HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct ChunkCoord(#[allow(dead_code)] pub IVec2);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .add_systems(Update, manage_chunks);
    }
}

fn manage_chunks(
    mut commands: Commands,
    world: Res<TileWorld>,
    mut chunks: ResMut<ChunkMap>,
    camera: Single<&Transform, With<Camera2d>>,
) {
    // Camera center in y-down pixel space
    let cam_px = Vec2::new(camera.translation.x, -camera.translation.y);
    let half = Vec2::new(WINDOW_W, WINDOW_H) / (2.0 * CAMERA_ZOOM);
    let chunk_px = CHUNK_SIZE as f32 * TILE_SIZE;
    let min = ((cam_px - half) / chunk_px).floor().as_ivec2();
    let max = ((cam_px + half) / chunk_px).floor().as_ivec2();
    let last = IVec2::new(WORLD_W / CHUNK_SIZE - 1, WORLD_H / CHUNK_SIZE - 1);

    for cy in (min.y - 1).max(0)..=(max.y + 1).min(last.y) {
        for cx in (min.x - 1).max(0)..=(max.x + 1).min(last.x) {
            let cc = IVec2::new(cx, cy);
            if !chunks.0.contains_key(&cc) {
                let e = spawn_chunk(&mut commands, &world, cc);
                chunks.0.insert(cc, e);
            }
        }
    }

    let keep_min = min - IVec2::splat(2);
    let keep_max = max + IVec2::splat(2);
    chunks.0.retain(|cc, e| {
        let keep = cc.x >= keep_min.x && cc.x <= keep_max.x
            && cc.y >= keep_min.y && cc.y <= keep_max.y;
        if !keep {
            commands.entity(*e).despawn(); // recursive via ChildOf
        }
        keep
    });
}

fn spawn_chunk(commands: &mut Commands, world: &TileWorld, cc: IVec2) -> Entity {
    let origin = Vec3::new(
        (cc.x * CHUNK_SIZE) as f32 * TILE_SIZE,
        -((cc.y * CHUNK_SIZE) as f32 * TILE_SIZE),
        0.0,
    );
    commands
        .spawn((Transform::from_translation(origin), Visibility::default(), ChunkCoord(cc)))
        .with_children(|parent| {
            for ly in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let t = world.tile_at(cc.x * CHUNK_SIZE + lx, cc.y * CHUNK_SIZE + ly);
                    if t == Tile::Air as u8 {
                        continue;
                    }
                    parent.spawn((
                        Sprite::from_color(tile_color(t), Vec2::splat(TILE_SIZE)),
                        Transform::from_translation(Vec3::new(
                            lx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                            -(ly as f32 * TILE_SIZE + TILE_SIZE / 2.0),
                            0.0,
                        )),
                    ));
                }
            }
        })
        .id()
}
