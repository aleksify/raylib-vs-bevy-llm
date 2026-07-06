use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::consts::*;
use crate::enemies::Enemy;
use crate::inventory::Inventory;
use crate::mining::AimTarget;
use crate::player::{BoxSize, Facing, Health, InputFrame, PixelPos, Player, Velocity};
use crate::world::TileWorld;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum Faction {
    Player,
    #[allow(dead_code)] // constructed by bee projectiles in M5
    Enemy,
}

#[derive(Component)]
pub struct Projectile {
    pub dmg: i32,
    pub gravity_factor: f32,
}

#[derive(Component)]
pub struct Lifetime(pub f32);

/// >0 while invulnerable after a hit
#[derive(Component)]
pub struct Invuln(pub f32);

#[derive(Component)]
pub struct BowCd(pub f32);

/// Present on the player only while a swing is active
#[derive(Component)]
pub struct Swing {
    t: f32,
    hit: HashSet<Entity>, // one swing hits each enemy once
}

#[derive(Component)]
struct SwordVisual;

/// Player respawn point (AABB top-left, y-down pixels)
#[derive(Resource)]
pub struct SpawnPoint(pub Vec2);

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (combat_input, swing_update, projectile_update, player_respawn).chain(),
        )
        .add_systems(Update, animate_sword);
    }
}

fn combat_input(
    mouse: Res<ButtonInput<MouseButton>>,
    input: Res<InputFrame>,
    inv: Res<Inventory>,
    aim: Res<AimTarget>,
    time: Res<Time>,
    player: Single<
        (Entity, &PixelPos, &BoxSize, &mut Facing, &mut BowCd, &mut Invuln, Has<Swing>),
        With<Player>,
    >,
    mut commands: Commands,
) {
    let (entity, pos, size, mut facing, mut bow_cd, mut invuln, swinging) =
        player.into_inner();

    if input.move_x > 0.0 {
        facing.0 = 1;
    } else if input.move_x < 0.0 {
        facing.0 = -1;
    }
    if invuln.0 > 0.0 {
        invuln.0 -= time.delta_secs();
    }
    if bow_cd.0 > 0.0 {
        bow_cd.0 -= time.delta_secs();
    }

    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    let held = inv.slots[inv.selected].id;
    let pc = pos.0 + size.0 / 2.0;

    if held == ITEM_SWORD && !swinging {
        facing.0 = if aim.world_px.x >= pc.x { 1 } else { -1 }; // swing toward mouse
        commands.entity(entity).insert(Swing { t: SWING_TIME, hit: default() });
        commands.spawn((
            SwordVisual,
            ChildOf(entity),
            Sprite::from_color(Color::srgb_u8(200, 200, 200), Vec2::new(16.0, 3.0)),
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
    }

    if held == ITEM_BOW && bow_cd.0 <= 0.0 {
        bow_cd.0 = BOW_COOLDOWN;
        let d = aim.world_px - pc;
        let dir = if d.length() < 1.0 {
            Vec2::new(facing.0 as f32, 0.0)
        } else {
            d.normalize()
        };
        spawn_projectile(&mut commands, pc, dir * ARROW_SPEED, ARROW_DMG,
                         Faction::Player, ARROW_GRAVITY);
    }
}

pub fn spawn_projectile(
    commands: &mut Commands,
    pos: Vec2,
    vel: Vec2,
    dmg: i32,
    faction: Faction,
    gravity_factor: f32,
) {
    commands.spawn((
        Projectile { dmg, gravity_factor },
        faction,
        Lifetime(ARROW_LIFETIME),
        PixelPos(pos - 2.0), // 4x4 box centered on pos
        BoxSize(Vec2::splat(4.0)),
        Velocity(vel),
        Sprite::from_color(Color::srgb_u8(235, 225, 185), Vec2::splat(4.0)),
        Transform::from_xyz(0.0, 0.0, 0.6),
    ));
}

fn swing_update(
    time: Res<Time>,
    player: Option<
        Single<(Entity, &PixelPos, &BoxSize, &Facing, &mut Swing), With<Player>>,
    >,
    mut enemies: Query<
        (Entity, &PixelPos, &BoxSize, &mut Health, &mut Velocity),
        (With<Enemy>, Without<Player>),
    >,
    visuals: Query<Entity, With<SwordVisual>>,
    mut commands: Commands,
) {
    let Some(player) = player else { return };
    let (entity, pos, size, facing, mut swing) = player.into_inner();

    swing.t -= time.delta_secs();
    if swing.t <= 0.0 {
        commands.entity(entity).remove::<Swing>();
        for v in &visuals {
            commands.entity(v).despawn();
        }
        return;
    }

    // 24x24 hitbox in front of the player
    let hb_min = Vec2::new(
        if facing.0 > 0 { pos.0.x + size.0.x } else { pos.0.x - SWORD_HITBOX },
        pos.0.y + size.0.y / 2.0 - SWORD_HITBOX / 2.0,
    );
    for (enemy, epos, esize, mut hp, mut vel) in &mut enemies {
        if swing.hit.contains(&enemy) {
            continue;
        }
        let overlap = hb_min.x < epos.0.x + esize.0.x
            && hb_min.x + SWORD_HITBOX > epos.0.x
            && hb_min.y < epos.0.y + esize.0.y
            && hb_min.y + SWORD_HITBOX > epos.0.y;
        if !overlap {
            continue;
        }
        swing.hit.insert(enemy);
        hp.0 -= SWORD_DMG;
        vel.0 = Vec2::new(facing.0 as f32 * 180.0, -120.0);
        if hp.0 <= 0 {
            commands.entity(enemy).despawn(); // TODO(M5): death poof + SFX
        }
    }
}

/// Blade sweeps ~120 degrees around the hand anchor (local to the player)
fn animate_sword(
    player: Option<Single<(&Facing, &Swing), With<Player>>>,
    mut visuals: Query<&mut Transform, With<SwordVisual>>,
) {
    let Some(player) = player else { return };
    let (facing, swing) = *player;
    let progress = 1.0 - swing.t / SWING_TIME;
    // Bevy angles are CCW-positive (y-up): mirror of the C version's values
    let deg = if facing.0 > 0 { 60.0 - 120.0 * progress } else { 120.0 + 120.0 * progress };
    let rot = Quat::from_rotation_z(deg.to_radians());
    for mut tf in &mut visuals {
        tf.rotation = rot;
        tf.translation = rot * Vec3::new(8.0, 0.0, 0.1); // blade center off the hand
    }
}

fn projectile_update(
    time: Res<Time>,
    world: Res<TileWorld>,
    mut projectiles: Query<(
        Entity,
        &mut PixelPos,
        &mut Velocity,
        &Projectile,
        &Faction,
        &mut Lifetime,
    )>,
    mut enemies: Query<
        (&PixelPos, &BoxSize, &mut Health, &mut Velocity),
        (With<Enemy>, Without<Projectile>),
    >,
    player: Single<
        (&PixelPos, &BoxSize, &mut Health, &mut Velocity, &mut Invuln),
        (With<Player>, Without<Projectile>, Without<Enemy>),
    >,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let (ppos, psize, mut php, mut pvel, mut invuln) = player.into_inner();

    for (entity, mut pos, mut vel, proj, faction, mut life) in &mut projectiles {
        life.0 -= dt;
        if life.0 <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        vel.0.y += GRAVITY * proj.gravity_factor * dt;
        pos.0 += vel.0 * dt;

        let center = pos.0 + 2.0;
        let tile = (center / TILE_SIZE).floor().as_ivec2();
        if world.is_solid(tile.x, tile.y) {
            commands.entity(entity).despawn();
            continue;
        }

        match faction {
            Faction::Player => {
                for (epos, esize, mut hp, mut evel) in &mut enemies {
                    let inside = center.x >= epos.0.x
                        && center.x <= epos.0.x + esize.0.x
                        && center.y >= epos.0.y
                        && center.y <= epos.0.y + esize.0.y;
                    if !inside {
                        continue;
                    }
                    hp.0 -= proj.dmg;
                    evel.0 = Vec2::new(if vel.0.x >= 0.0 { 180.0 } else { -180.0 }, -120.0);
                    commands.entity(entity).despawn();
                    break;
                }
            }
            Faction::Enemy => {
                let inside = center.x >= ppos.0.x
                    && center.x <= ppos.0.x + psize.0.x
                    && center.y >= ppos.0.y
                    && center.y <= ppos.0.y + psize.0.y;
                if inside && invuln.0 <= 0.0 {
                    invuln.0 = HURT_INVULN;
                    php.0 -= proj.dmg;
                    let pcx = ppos.0.x + psize.0.x / 2.0;
                    pvel.0 = Vec2::new(if pcx < center.x { -160.0 } else { 160.0 }, -160.0);
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn player_respawn(
    spawn: Res<SpawnPoint>,
    player: Single<
        (&mut PixelPos, &mut Health, &mut Velocity, &mut Invuln),
        With<Player>,
    >,
) {
    let (mut pos, mut hp, mut vel, mut invuln) = player.into_inner();
    if hp.0 > 0 {
        return;
    }
    // Death: respawn at world spawn, restore HP
    hp.0 = PLAYER_MAX_HP;
    pos.0 = spawn.0;
    vel.0 = Vec2::ZERO;
    invuln.0 = 1.0;
}
