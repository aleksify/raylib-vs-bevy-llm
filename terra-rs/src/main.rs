use bevy::prelude::*;

mod consts;
mod noise;
mod player;
mod world;
mod worldgen;

use consts::*;

const DEFAULT_SEED: u64 = 1337;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(i) = args.iter().position(|a| a == "--dump-seed") {
        // Worldgen parity check vs terra-c: print hash, no window
        let seed: u64 = args[i + 1].parse().expect("--dump-seed <u64>");
        let w = worldgen::generate(seed);
        println!("{:016x}", noise::fnv1a64(&w.tiles));
        return;
    }

    let seed = args
        .iter()
        .position(|a| a == "--seed")
        .map(|i| args[i + 1].parse().expect("--seed <u64>"))
        .unwrap_or(DEFAULT_SEED);

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
        .insert_resource(worldgen::generate(seed))
        .add_plugins((world::WorldPlugin, player::PlayerPlugin))
        .run();
}
