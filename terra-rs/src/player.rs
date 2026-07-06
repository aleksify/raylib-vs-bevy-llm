use bevy::prelude::*;

use crate::consts::*;
use crate::world::{move_and_collide, TileWorld};

#[derive(Component)]
pub struct Player;

/// AABB top-left in world pixels, y-down (matches terra-c's player.box)
#[derive(Component)]
pub struct PixelPos(pub Vec2);

/// AABB size in pixels; with PixelPos drives the Transform sync
#[derive(Component)]
pub struct BoxSize(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Health(pub i32);

/// -1 / 1
#[derive(Component)]
pub struct Facing(pub i32);

/// Input edges are latched in Update and consumed in FixedUpdate, because
/// just_pressed is per-frame and FixedUpdate can run 0..n times per frame.
#[derive(Resource, Default)]
pub struct InputFrame {
    pub move_x: f32,
    pub jump_pressed: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFrame>()
            .add_systems(Startup, spawn_player_and_camera)
            .add_systems(Update, (gather_input, sync_pixel_transforms, camera_follow).chain())
            .add_systems(
                FixedUpdate,
                player_physics.run_if(in_state(crate::state::GameState::Playing)),
            );
    }
}

fn spawn_player_and_camera(
    mut commands: Commands,
    world: Res<TileWorld>,
    assets: Res<crate::assets::GameAssets>,
) {
    // Spawn on the generated surface at world center
    let spawn = Vec2::new(
        (WORLD_W / 2) as f32 * TILE_SIZE,
        crate::worldgen::surface_y(&world, WORLD_W / 2) as f32 * TILE_SIZE - PLAYER_BOX_H,
    );
    let center = Vec3::new(
        spawn.x + PLAYER_BOX_W / 2.0,
        -(spawn.y + PLAYER_BOX_H / 2.0),
        0.0,
    );

    commands.insert_resource(crate::combat::SpawnPoint(spawn));
    commands.spawn((
        Player,
        PixelPos(spawn),
        BoxSize(Vec2::new(PLAYER_BOX_W, PLAYER_BOX_H)),
        Velocity(Vec2::ZERO),
        Grounded(false),
        Health(PLAYER_MAX_HP),
        Facing(1),
        crate::combat::Invuln(0.0),
        crate::combat::BowCd(0.0),
        assets.sprite("char_player", Vec2::splat(24.0)).unwrap_or_else(|| {
            Sprite::from_color(Color::srgb_u8(235, 90, 70), Vec2::new(PLAYER_BOX_W, PLAYER_BOX_H))
        }),
        Transform::from_translation(center.with_z(1.0)),
    ));

    // Camera zoom via transform scale (0.5 => 2x zoom), sidestepping
    // projection API churn between Bevy versions. Start on the player so the
    // first chunk load happens at spawn, not at the world origin.
    commands.spawn((
        Camera2d,
        Transform::from_translation(center)
            .with_scale(Vec3::new(1.0 / CAMERA_ZOOM, 1.0 / CAMERA_ZOOM, 1.0)),
    ));
}

fn gather_input(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<InputFrame>) {
    let mut mv = 0.0;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        mv -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        mv += 1.0;
    }
    input.move_x = mv;
    if keys.just_pressed(KeyCode::Space) {
        input.jump_pressed = true;
    }
}

fn player_physics(
    world: Res<TileWorld>,
    mut input: ResMut<InputFrame>,
    player: Single<(&mut PixelPos, &mut Velocity, &mut Grounded), With<Player>>,
    time: Res<Time>,
) {
    let (mut pos, mut vel, mut grounded) = player.into_inner();
    let dt = time.delta_secs();

    vel.0.x = input.move_x * PLAYER_SPEED;
    vel.0.y += GRAVITY * dt;

    if input.jump_pressed && grounded.0 {
        vel.0.y = PLAYER_JUMP_VEL;
        grounded.0 = false;
    }
    input.jump_pressed = false; // edge consumed

    let size = Vec2::new(PLAYER_BOX_W, PLAYER_BOX_H);
    grounded.0 = move_and_collide(&world, &mut pos.0, size, &mut vel.0, dt);
}

/// y-down pixel space -> Bevy y-up transform (sprites are center-anchored).
/// Covers the player, drops, and later enemies/projectiles.
fn sync_pixel_transforms(
    mut q: Query<(&PixelPos, &BoxSize, &mut Transform, Option<&Facing>, Option<&mut Sprite>)>,
) {
    for (pos, size, mut tf, facing, sprite) in &mut q {
        tf.translation.x = pos.0.x + size.0.x / 2.0;
        tf.translation.y = -(pos.0.y + size.0.y / 2.0);
        if let (Some(f), Some(mut s)) = (facing, sprite) {
            s.flip_x = f.0 < 0;
        }
    }
}

fn camera_follow(
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut shake: ResMut<crate::combat::Shake>,
    time: Res<Time>,
) {
    // Whole-pixel camera target: no sprite seams/shimmer at zoom 2
    camera.translation.x = player.translation.x.round();
    camera.translation.y = player.translation.y.round();
    if shake.0 > 0.0 {
        shake.0 -= time.delta_secs();
        let t = time.elapsed_secs();
        let a = 4.0 * (shake.0 / crate::combat::SHAKE_TIME);
        camera.translation.x += (t * 70.0).sin() * a;
        camera.translation.y += (t * 53.0).cos() * a;
    }
}
