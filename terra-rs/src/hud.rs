use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::player::{Health, Player};

const HEART_FULL: Color = Color::srgb_u8(220, 60, 60);
const HEART_EMPTY: Color = Color::srgba_u8(60, 60, 60, 180);

#[derive(Component)]
struct Heart(usize);

#[derive(Component)]
struct FpsText;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, (spawn_hearts, spawn_fps_text))
            .add_systems(Update, (refresh_hearts, refresh_fps));
    }
}

fn spawn_fps_text(mut commands: Commands) {
    commands.spawn((
        FpsText,
        Text::new("FPS"),
        TextFont { font_size: FontSize::Px(14.0), ..default() },
        TextColor(Color::srgb(0.4, 1.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn refresh_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut text: Single<&mut Text, With<FpsText>>,
) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
    {
        text.0 = format!("{fps:.0} FPS");
    }
}

fn spawn_hearts(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(34.0),
            left: Val::Px(10.0),
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            // 10 hearts x 10 HP
            for i in 0..10 {
                row.spawn((
                    Heart(i),
                    Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
                    BackgroundColor(HEART_FULL),
                ));
            }
        });
}

fn refresh_hearts(
    player: Query<&Health, (With<Player>, Changed<Health>)>,
    mut hearts: Query<(&Heart, &mut BackgroundColor)>,
) {
    let Ok(hp) = player.single() else { return };
    for (heart, mut bg) in &mut hearts {
        bg.0 = if hp.0 > (heart.0 as i32) * 10 { HEART_FULL } else { HEART_EMPTY };
    }
}
