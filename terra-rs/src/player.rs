use bevy::prelude::*;

use crate::consts::*;
use crate::world::{move_and_collide, TileWorld};

#[derive(Component)]
pub struct Player;

/// AABB top-left in world pixels, y-down (matches terra-c's player.box)
#[derive(Component)]
pub struct PixelPos(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
#[allow(dead_code)]
pub struct Health(pub i32);

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
            .add_systems(Update, (gather_input, sync_player_transform, camera_follow).chain())
            .add_systems(FixedUpdate, player_physics);
    }
}

fn spawn_player_and_camera(mut commands: Commands, world: Res<TileWorld>) {
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

    commands.spawn((
        Player,
        PixelPos(spawn),
        Velocity(Vec2::ZERO),
        Grounded(false),
        Health(PLAYER_MAX_HP),
        Sprite::from_color(
            Color::srgb_u8(235, 90, 70),
            Vec2::new(PLAYER_BOX_W, PLAYER_BOX_H),
        ),
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

/// y-down pixel space -> Bevy y-up transform (sprite is center-anchored)
fn sync_player_transform(
    player: Single<(&PixelPos, &mut Transform), With<Player>>,
) {
    let (pos, mut tf) = player.into_inner();
    tf.translation.x = pos.0.x + PLAYER_BOX_W / 2.0;
    tf.translation.y = -(pos.0.y + PLAYER_BOX_H / 2.0);
}

fn camera_follow(
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    // Whole-pixel camera target: no sprite seams/shimmer at zoom 2
    camera.translation.x = player.translation.x.round();
    camera.translation.y = player.translation.y.round();
}
