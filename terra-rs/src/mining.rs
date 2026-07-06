use bevy::prelude::*;

use crate::consts::*;
use crate::drops::{spawn_drop, GameRng};
use crate::inventory::Inventory;
use crate::enemies::Enemy;
use crate::player::{BoxSize, PixelPos, Player};
use crate::world::{Tile, TileChanged, TileWorld};

// Hits to break, by tile id (air, dirt, grass, stone, wood, leaves, ore)
const HARDNESS: [i32; 7] = [0, 2, 2, 4, 3, 1, 4];

fn drop_for_tile(t: u8) -> u8 {
    if t == Tile::Grass as u8 {
        Tile::Dirt as u8
    } else if t == Tile::Leaves as u8 {
        ITEM_NONE
    } else {
        t
    }
}

#[derive(Resource, Default)]
pub struct AimTarget {
    pub tile: IVec2,
    pub world_px: Vec2, // mouse in world pixels (y-down)
    pub in_reach: bool,
}

#[derive(Clone, Copy)]
struct DamageEntry {
    x: i32,
    y: i32,
    dmg: i32,
}

/// Mirrors terra-c: 8-entry damage table + the two action cooldowns
#[derive(Resource, Default)]
struct MineState {
    mine_cd: f32,
    place_cd: f32,
    table: [Option<DamageEntry>; 8],
}

pub struct MiningPlugin;

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        let playing = in_state(crate::state::GameState::Playing);
        app.init_resource::<AimTarget>()
            .init_resource::<MineState>()
            .add_systems(Update, (update_aim, draw_highlight).chain().run_if(playing.clone()))
            .add_systems(FixedUpdate, (mine, place).run_if(playing));
    }
}

fn update_aim(
    window: Single<&Window>,
    cam: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    player: Single<(&PixelPos, &BoxSize), With<Player>>,
    mut aim: ResMut<AimTarget>,
) {
    let (camera, cam_tf) = *cam;
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else { return };
    // Bevy y-up -> pixel y-down
    let px = Vec2::new(world.x, -world.y);
    aim.world_px = px;
    aim.tile = (px / TILE_SIZE).floor().as_ivec2();

    let (ppos, psize) = *player;
    let pc = ppos.0 + psize.0 / 2.0;
    let tc = aim.tile.as_vec2() * TILE_SIZE + TILE_SIZE / 2.0;
    let reach = PLAYER_REACH as f32 * TILE_SIZE;
    aim.in_reach = pc.distance_squared(tc) <= reach * reach;
}

fn draw_highlight(aim: Res<AimTarget>, mut gizmos: Gizmos) {
    if !aim.in_reach {
        return;
    }
    let center = Vec2::new(
        aim.tile.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        -(aim.tile.y as f32 * TILE_SIZE + TILE_SIZE / 2.0),
    );
    gizmos.rect_2d(center, Vec2::splat(TILE_SIZE), Color::WHITE);
}

impl MineState {
    fn entry(&mut self, x: i32, y: i32) -> &mut DamageEntry {
        let pos = self
            .table
            .iter()
            .position(|e| matches!(e, Some(e) if e.x == x && e.y == y));
        let idx = match pos {
            Some(i) => i,
            None => match self.table.iter().position(Option::is_none) {
                Some(i) => {
                    self.table[i] = Some(DamageEntry { x, y, dmg: 0 });
                    i
                }
                None => {
                    // Table full: forget old targets, reuse slot 0
                    self.table = [None; 8];
                    self.table[0] = Some(DamageEntry { x, y, dmg: 0 });
                    0
                }
            },
        };
        self.table[idx].as_mut().unwrap()
    }
}

fn mine(
    mouse: Res<ButtonInput<MouseButton>>,
    aim: Res<AimTarget>,
    inv: Res<Inventory>,
    mut state: ResMut<MineState>,
    mut world: ResMut<TileWorld>,
    mut changed: MessageWriter<TileChanged>,
    mut rng: ResMut<GameRng>,
    assets: Res<crate::assets::GameAssets>,
    mut commands: Commands,
    time: Res<Time>,
) {
    state.mine_cd -= time.delta_secs();
    if !mouse.pressed(MouseButton::Left) {
        state.mine_cd = 0.0; // first click is instant
        return;
    }
    // LMB is context-sensitive: weapons swing/shoot (combat.rs), blocks/empty mine
    let held = inv.slots[inv.selected].id;
    if held == ITEM_SWORD || held == ITEM_BOW {
        return;
    }
    if state.mine_cd > 0.0 || !aim.in_reach {
        return;
    }
    let (tx, ty) = (aim.tile.x, aim.tile.y);
    let t = world.tile_at(tx, ty);
    if t == Tile::Air as u8 {
        return;
    }
    state.mine_cd = MINE_COOLDOWN;

    let e = state.entry(tx, ty);
    e.dmg += 1;
    if e.dmg < HARDNESS[t as usize] {
        return;
    }

    state.table.iter_mut().for_each(|e| {
        if matches!(e, Some(e) if e.x == tx && e.y == ty) {
            *e = None;
        }
    });
    world.set_tile(tx, ty, Tile::Air as u8);
    changed.write(TileChanged { tile: aim.tile });
    crate::particles::spawn_burst(&mut commands, &mut rng,
        (aim.tile.as_vec2() + 0.5) * TILE_SIZE, crate::world::tile_color(t), 8);
    let drop = drop_for_tile(t);
    if drop != ITEM_NONE {
        spawn_drop(&mut commands, &mut rng, &assets, tx, ty, drop);
    }
}

fn place(
    mouse: Res<ButtonInput<MouseButton>>,
    aim: Res<AimTarget>,
    mut state: ResMut<MineState>,
    mut world: ResMut<TileWorld>,
    mut inv: ResMut<Inventory>,
    mut changed: MessageWriter<TileChanged>,
    player: Single<(&PixelPos, &BoxSize), With<Player>>,
    enemies: Query<(&PixelPos, &BoxSize), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    state.place_cd -= time.delta_secs();
    if !mouse.pressed(MouseButton::Right) {
        state.place_cd = 0.0;
        return;
    }
    if state.place_cd > 0.0 || !aim.in_reach {
        return;
    }
    let (tx, ty) = (aim.tile.x, aim.tile.y);

    let sel = inv.selected;
    let slot = inv.slots[sel];
    if slot.id == ITEM_NONE || slot.id > Tile::Ore as u8 || slot.count == 0 {
        return; // not a block
    }
    if world.tile_at(tx, ty) != Tile::Air as u8 {
        return;
    }
    if !world.is_solid(tx + 1, ty)
        && !world.is_solid(tx - 1, ty)
        && !world.is_solid(tx, ty + 1)
        && !world.is_solid(tx, ty - 1)
    {
        return;
    }
    let (ppos, psize) = *player;
    let tile_min = Vec2::new(tx as f32 * TILE_SIZE, ty as f32 * TILE_SIZE);
    let overlap = tile_min.x < ppos.0.x + psize.0.x
        && tile_min.x + TILE_SIZE > ppos.0.x
        && tile_min.y < ppos.0.y + psize.0.y
        && tile_min.y + TILE_SIZE > ppos.0.y;
    if overlap {
        return;
    }
    for (epos, esize) in &enemies {
        let hit = tile_min.x < epos.0.x + esize.0.x
            && tile_min.x + TILE_SIZE > epos.0.x
            && tile_min.y < epos.0.y + esize.0.y
            && tile_min.y + TILE_SIZE > epos.0.y;
        if hit {
            return;
        }
    }

    world.set_tile(tx, ty, slot.id);
    changed.write(TileChanged { tile: aim.tile });
    state.place_cd = MINE_COOLDOWN;
    let s = &mut inv.slots[sel];
    s.count -= 1;
    if s.count == 0 {
        s.id = ITEM_NONE;
    }
}
