use bevy::prelude::*;

use crate::combat::{spawn_projectile, Faction, Invuln};
use crate::consts::*;
use crate::drops::GameRng;
use crate::noise::rng_range;
use crate::player::{BoxSize, Grounded, Health, PixelPos, Player, Velocity};
use crate::world::{move_and_collide, TileWorld};
use crate::worldgen::surface_y;

pub const MAX_ENEMIES: usize = 8;
const SPAWN_INTERVAL: f32 = 3.0;
const DESPAWN_TILES: f32 = 80.0;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum EnemyKind {
    Slime,
    Zombie,
    Bee,
}

impl EnemyKind {
    fn hp(self) -> i32 {
        match self { Self::Slime => 30, Self::Zombie => 50, Self::Bee => 20 }
    }
    fn contact_dmg(self) -> i32 {
        match self { Self::Slime => 10, Self::Zombie => 15, Self::Bee => 8 }
    }
    fn size(self) -> Vec2 {
        match self {
            Self::Slime => Vec2::new(14.0, 12.0),
            Self::Zombie => Vec2::new(12.0, 22.0),
            Self::Bee => Vec2::new(12.0, 10.0),
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Slime => Color::srgb_u8(90, 200, 120),
            Self::Zombie => Color::srgb_u8(110, 130, 110),
            Self::Bee => Color::srgb_u8(230, 190, 60),
        }
    }
}

/// Slime hop timer / bee shot cooldown
#[derive(Component)]
pub struct AiTimer(pub f32);

/// Bee sine-wave phase
#[derive(Component)]
pub struct Phase(pub f32);

/// >0 -> render white; set on hit
#[derive(Component)]
pub struct HurtFlash(pub f32);

#[derive(Resource)]
struct SpawnTimer(f32);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(0.0))
            .add_systems(
                FixedUpdate,
                (spawner, (slime_ai, zombie_ai, bee_ai), enemy_physics,
                 contact_damage, despawn_far)
                    .chain()
                    .run_if(in_state(crate::state::GameState::Playing)),
            )
            .add_systems(Update, tint_hurt);
    }
}

fn spawner(
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut rng: ResMut<GameRng>,
    world: Res<TileWorld>,
    assets: Res<crate::assets::GameAssets>,
    player: Single<&PixelPos, With<Player>>,
    enemies: Query<(), With<Enemy>>,
    mut commands: Commands,
) {
    timer.0 -= time.delta_secs();
    if timer.0 > 0.0 {
        return;
    }
    timer.0 = SPAWN_INTERVAL;
    if enemies.iter().count() >= MAX_ENEMIES {
        return;
    }

    // Surface tile 20-40 tiles from the player; >=20 tiles is off-screen at zoom 2
    let player_tx = ((player.0.x + PLAYER_BOX_W / 2.0) / TILE_SIZE) as i32;
    let dist = rng_range(&mut rng.0, 20, 40);
    let dir = if rng_range(&mut rng.0, 0, 1) == 1 { 1 } else { -1 };
    let tx = player_tx + dir * dist;
    if tx < 1 || tx >= WORLD_W - 1 {
        return;
    }
    let sy = surface_y(&world, tx);

    let kind = match rng_range(&mut rng.0, 0, 2) {
        0 => EnemyKind::Slime,
        1 => EnemyKind::Zombie,
        _ => EnemyKind::Bee,
    };
    let size = kind.size();
    let x = tx as f32 * TILE_SIZE + (TILE_SIZE - size.x) / 2.0;
    let y = if kind == EnemyKind::Bee {
        (sy - rng_range(&mut rng.0, 3, 6)) as f32 * TILE_SIZE // bees start airborne
    } else {
        sy as f32 * TILE_SIZE - size.y
    };

    commands.spawn((
        Enemy,
        kind,
        PixelPos(Vec2::new(x, y)),
        BoxSize(size),
        Velocity(Vec2::ZERO),
        Grounded(false),
        Health(kind.hp()),
        AiTimer(0.0),
        Phase(0.0),
        HurtFlash(0.0),
        {
            let name = match kind {
                EnemyKind::Slime => "char_slime",
                EnemyKind::Zombie => "char_zombie",
                EnemyKind::Bee => "char_bee",
            };
            let draw = if kind == EnemyKind::Zombie { 24.0 } else { 16.0 };
            assets.sprite(name, Vec2::splat(draw))
                .unwrap_or_else(|| Sprite::from_color(kind.color(), size))
        },
        Transform::from_xyz(0.0, 0.0, 0.8),
    ));
}

fn player_center(pos: &PixelPos) -> Vec2 {
    pos.0 + Vec2::new(PLAYER_BOX_W, PLAYER_BOX_H) / 2.0
}

fn slime_ai(
    time: Res<Time>,
    player: Single<&PixelPos, With<Player>>,
    mut q: Query<
        (&EnemyKind, &PixelPos, &BoxSize, &mut Velocity, &Grounded, &mut AiTimer),
        (With<Enemy>, Without<Player>),
    >,
) {
    let pc = player_center(&player);
    for (kind, pos, size, mut vel, grounded, mut timer) in &mut q {
        if *kind != EnemyKind::Slime || !grounded.0 {
            continue;
        }
        vel.0.x = 0.0; // slimes don't slide
        timer.0 -= time.delta_secs();
        if timer.0 <= 0.0 {
            timer.0 = 1.5;
            let dir = if pc.x >= pos.0.x + size.0.x / 2.0 { 1.0 } else { -1.0 };
            vel.0 = Vec2::new(dir * 90.0, -260.0); // hop toward the player
        }
    }
}

fn zombie_ai(
    player: Single<&PixelPos, With<Player>>,
    mut q: Query<
        (&EnemyKind, &PixelPos, &BoxSize, &mut Velocity),
        (With<Enemy>, Without<Player>),
    >,
) {
    let pc = player_center(&player);
    for (kind, pos, size, mut vel) in &mut q {
        if *kind != EnemyKind::Zombie {
            continue;
        }
        let dir = if pc.x >= pos.0.x + size.0.x / 2.0 { 1.0 } else { -1.0 };
        vel.0.x = dir * 50.0;
    }
}

fn bee_ai(
    time: Res<Time>,
    player: Single<&PixelPos, With<Player>>,
    mut q: Query<
        (&EnemyKind, &PixelPos, &BoxSize, &mut Velocity, &mut AiTimer, &mut Phase),
        (With<Enemy>, Without<Player>),
    >,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let pc = player_center(&player);
    for (kind, pos, size, mut vel, mut timer, mut phase) in &mut q {
        if *kind != EnemyKind::Bee {
            continue;
        }
        // No gravity: drift toward the player with a sine-wave wobble
        phase.0 += dt;
        let ec = pos.0 + size.0 / 2.0;
        let d = pc - ec;
        let len = d.length();
        if len > 1.0 {
            vel.0.x = d.x / len * 60.0;
            vel.0.y = d.y / len * 60.0 + (phase.0 * 5.0).sin() * 40.0;
        }
        timer.0 -= dt;
        if timer.0 <= 0.0 && len < 20.0 * TILE_SIZE {
            timer.0 = 2.0;
            spawn_projectile(&mut commands, ec, d / len * 150.0, 8,
                             Faction::Enemy, 0.0); // slow, straight shot
        }
    }
}

fn enemy_physics(
    time: Res<Time>,
    world: Res<TileWorld>,
    mut q: Query<
        (&EnemyKind, &mut PixelPos, &BoxSize, &mut Velocity, &mut Grounded, &mut HurtFlash),
        (With<Enemy>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();
    for (kind, mut pos, size, mut vel, mut grounded, mut flash) in &mut q {
        if *kind != EnemyKind::Bee {
            vel.0.y += GRAVITY * dt;
        }
        let want_vx = vel.0.x;
        grounded.0 = move_and_collide(&world, &mut pos.0, size.0, &mut vel.0, dt);

        // Zombie blocked by a wall while walking -> jump it
        if *kind == EnemyKind::Zombie && grounded.0 && want_vx != 0.0 && vel.0.x == 0.0 {
            vel.0.y = -260.0;
        }
        if flash.0 > 0.0 {
            flash.0 -= dt;
        }
    }
}

fn contact_damage(
    enemies: Query<(&EnemyKind, &PixelPos, &BoxSize), (With<Enemy>, Without<Player>)>,
    player: Single<
        (&PixelPos, &BoxSize, &mut Health, &mut Velocity, &mut Invuln),
        With<Player>,
    >,
    mut shake: ResMut<crate::combat::Shake>,
) {
    let (ppos, psize, mut hp, mut vel, mut invuln) = player.into_inner();
    if invuln.0 > 0.0 {
        return;
    }
    for (kind, epos, esize, ) in &enemies {
        let overlap = epos.0.x < ppos.0.x + psize.0.x
            && epos.0.x + esize.0.x > ppos.0.x
            && epos.0.y < ppos.0.y + psize.0.y
            && epos.0.y + esize.0.y > ppos.0.y;
        if !overlap {
            continue;
        }
        invuln.0 = HURT_INVULN;
        hp.0 -= kind.contact_dmg();
        shake.0 = crate::combat::SHAKE_TIME;
        let pcx = ppos.0.x + psize.0.x / 2.0;
        let ecx = epos.0.x + esize.0.x / 2.0;
        vel.0 = Vec2::new(if pcx < ecx { -160.0 } else { 160.0 }, -160.0);
        break; // one hit per invulnerability window
    }
}

fn despawn_far(
    player: Single<&PixelPos, With<Player>>,
    enemies: Query<(Entity, &PixelPos), (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
) {
    for (entity, pos) in &enemies {
        if (pos.0.x - player.0.x).abs() > DESPAWN_TILES * TILE_SIZE {
            commands.entity(entity).despawn();
        }
    }
}

fn tint_hurt(mut q: Query<(&HurtFlash, &mut Sprite), With<Enemy>>) {
    for (flash, mut sprite) in &mut q {
        // Sprites tint red on hit (can't over-whiten a texture with a tint)
        sprite.color = if flash.0 > 0.0 {
            Color::srgb_u8(255, 110, 110)
        } else {
            Color::WHITE
        };
    }
}
