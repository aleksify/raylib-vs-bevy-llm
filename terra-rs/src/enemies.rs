use bevy::prelude::*;

/// Marker; M5 adds slime/zombie/bee types, spawner, and AI systems.
#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, _app: &mut App) {
        // M5
    }
}
