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
    // M1: flat floor. Replaced by procedural generation in M2.
    pub fn flat() -> Self {
        let surface = (WORLD_H / 2) as usize;
        let mut tiles = vec![Tile::Air as u8; (WORLD_W * WORLD_H) as usize];
        for y in surface..WORLD_H as usize {
            for x in 0..WORLD_W as usize {
                tiles[y * WORLD_W as usize + x] = if y == surface {
                    Tile::Grass as u8
                } else if y <= surface + 8 {
                    Tile::Dirt as u8
                } else {
                    Tile::Stone as u8
                };
            }
        }
        Self { tiles }
    }

    /// OOB -> Stone (solid), kills all edge cases
    pub fn tile_at(&self, tx: i32, ty: i32) -> u8 {
        if tx < 0 || ty < 0 || tx >= WORLD_W || ty >= WORLD_H {
            return Tile::Stone as u8;
        }
        self.tiles[(ty * WORLD_W + tx) as usize]
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

/// Tile grid coords -> Bevy world translation of the tile center.
pub fn tile_to_bevy(tx: i32, ty: i32) -> Vec3 {
    Vec3::new(
        tx as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        -(ty as f32 * TILE_SIZE + TILE_SIZE / 2.0),
        0.0,
    )
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

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileWorld::flat())
            .add_systems(Startup, spawn_flat_floor_sprites);
    }
}

/// M1 throwaway: sprites for solid tiles near spawn only. M2 replaces this
/// with camera-driven chunk loading.
fn spawn_flat_floor_sprites(mut commands: Commands, world: Res<TileWorld>) {
    let cx = WORLD_W / 2;
    let cy = WORLD_H / 2;
    for ty in (cy - 32).max(0)..(cy + 32).min(WORLD_H) {
        for tx in (cx - 80).max(0)..(cx + 80).min(WORLD_W) {
            let t = world.tile_at(tx, ty);
            if t == Tile::Air as u8 {
                continue;
            }
            commands.spawn((
                Sprite::from_color(tile_color(t), Vec2::splat(TILE_SIZE)),
                Transform::from_translation(tile_to_bevy(tx, ty)),
            ));
        }
    }
}
