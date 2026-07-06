use bevy::prelude::*;

mod consts;
mod player;
mod world;

use consts::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest()) // pixel art: no filtering
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "terra (bevy)".into(),
                        // WindowResolution is u32 physical pixels since 0.17
                        resolution: (WINDOW_W as u32, WINDOW_H as u32).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb_u8(108, 158, 222))) // sky
        .add_plugins((world::WorldPlugin, player::PlayerPlugin))
        .run();
}
