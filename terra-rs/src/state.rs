use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, transitions)
            .add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_overlay);
    }
}

fn transitions(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    match state.get() {
        GameState::Menu => {
            if keys.just_pressed(KeyCode::Enter) {
                next.set(GameState::Playing);
            }
        }
        GameState::Playing => {
            if keys.just_pressed(KeyCode::Escape) {
                next.set(GameState::Paused);
            }
        }
        GameState::Paused => {
            if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
                next.set(GameState::Playing);
            }
        }
    }
}

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(GameState::Menu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn((
                Text::new("TERRA"),
                TextFont { font_size: FontSize::Px(60.0), ..default() },
            ));
            c.spawn((
                Text::new("press ENTER to play"),
                TextFont { font_size: FontSize::Px(20.0), ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(GameState::Paused),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.47)),
        ))
        .with_children(|c| {
            c.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: FontSize::Px(40.0), ..default() },
            ));
        });
}
