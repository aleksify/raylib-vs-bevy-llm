use bevy::prelude::*;

use crate::consts::*;
use crate::drops::GameRng;
use crate::noise::rng_float;
use crate::player::{BoxSize, PixelPos, Velocity};
use crate::state::GameState;

#[derive(Component)]
pub struct Particle {
    life: f32,
    max: f32,
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_particles.run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn spawn_burst(
    commands: &mut Commands,
    rng: &mut GameRng,
    center: Vec2,
    color: Color,
    count: usize,
) {
    for _ in 0..count {
        let ang = rng_float(&mut rng.0) * std::f32::consts::TAU;
        let spd = 40.0 + rng_float(&mut rng.0) * 80.0;
        let life = 0.4 + rng_float(&mut rng.0) * 0.3;
        commands.spawn((
            Particle { life, max: life },
            PixelPos(center),
            BoxSize(Vec2::splat(3.0)),
            Velocity(Vec2::new(ang.cos() * spd, ang.sin() * spd - 40.0)),
            Sprite::from_color(color, Vec2::splat(3.0)),
            Transform::from_xyz(0.0, 0.0, 0.9),
        ));
    }
}

fn update_particles(
    time: Res<Time>,
    mut q: Query<(Entity, &mut PixelPos, &mut Velocity, &mut Particle, &mut Sprite)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut pos, mut vel, mut p, mut sprite) in &mut q {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        vel.0.y += GRAVITY * 0.5 * dt; // float a bit
        let v = vel.0;
        pos.0 += v * dt;
        sprite.color = sprite.color.with_alpha(p.life / p.max); // fade out
    }
}
