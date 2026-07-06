use bevy::prelude::*;

use crate::consts::*;
use crate::inventory::{item_color, Inventory};
use crate::noise::rng_range;
use crate::player::{BoxSize, PixelPos, Player, Velocity};
use crate::world::{move_and_collide, TileWorld};

const DROP_SIZE: f32 = 8.0;

#[derive(Component)]
pub struct DropItem(pub u8);

#[derive(EntityEvent)]
pub struct PickedUp {
    entity: Entity,
}

/// Gameplay-only splitmix64 stream — never used by worldgen, so gameplay
/// randomness can't desync the world hash.
#[derive(Resource)]
pub struct GameRng(pub u64);

pub struct DropsPlugin;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_drops.run_if(in_state(crate::state::GameState::Playing)),
        )
        .add_observer(on_picked_up);
    }
}

pub fn spawn_drop(
    commands: &mut Commands,
    rng: &mut GameRng,
    assets: &crate::assets::GameAssets,
    tx: i32,
    ty: i32,
    item: u8,
) {
    let pos = Vec2::new(
        tx as f32 * TILE_SIZE + 4.0,
        ty as f32 * TILE_SIZE + 4.0,
    );
    commands.spawn((
        DropItem(item),
        PixelPos(pos),
        BoxSize(Vec2::splat(DROP_SIZE)),
        Velocity(Vec2::new(rng_range(&mut rng.0, -40, 40) as f32, -120.0)),
        assets
            .tile_sprite(item, Vec2::splat(DROP_SIZE))
            .unwrap_or_else(|| Sprite::from_color(item_color(item), Vec2::splat(DROP_SIZE))),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));
}

fn update_drops(
    mut commands: Commands,
    world: Res<TileWorld>,
    time: Res<Time>,
    player: Single<(&PixelPos, &BoxSize), With<Player>>,
    mut drops: Query<(Entity, &mut PixelPos, &mut Velocity), (With<DropItem>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    let (ppos, psize) = *player;
    let pc = ppos.0 + psize.0 / 2.0;

    for (entity, mut pos, mut vel) in &mut drops {
        let dc = pos.0 + DROP_SIZE / 2.0;
        let delta = pc - dc;
        let dist = delta.length();

        if dist < DROP_HOMING_RANGE && dist > 1.0 {
            // Home to the player, ignore gravity
            vel.0 = delta / dist * 180.0;
        } else {
            vel.0.y += GRAVITY * dt;
            // Air drag so thrown drops settle instead of sliding forever
            vel.0.x *= 0.98;
        }

        let pre_vy = vel.0.y;
        let grounded =
            move_and_collide(&world, &mut pos.0, Vec2::splat(DROP_SIZE), &mut vel.0, dt);
        if grounded && pre_vy > 80.0 {
            vel.0.y = -pre_vy * 0.4; // bounce
        }

        // AABB overlap with the player -> pickup
        let overlap = pos.0.x < ppos.0.x + psize.0.x
            && pos.0.x + DROP_SIZE > ppos.0.x
            && pos.0.y < ppos.0.y + psize.0.y
            && pos.0.y + DROP_SIZE > ppos.0.y;
        if overlap {
            commands.trigger(PickedUp { entity });
        }
    }
}

fn on_picked_up(
    event: On<PickedUp>,
    drops: Query<&DropItem>,
    mut inv: ResMut<Inventory>,
    mut commands: Commands,
) {
    let Ok(item) = drops.get(event.entity) else { return };
    if inv.add(item.0, 1) {
        commands.entity(event.entity).despawn();
    }
}
