use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

mod assets;
mod combat;
mod consts;
mod drops;
mod enemies;
mod hud;
mod inventory;
mod mining;
mod noise;
mod particles;
mod player;
mod state;
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

    // Self-test: skip menu, run N frames, save a screenshot, exit
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .map(|i| args[i + 1].clone());
    let frames = args
        .iter()
        .position(|a| a == "--frames")
        .map(|i| args[i + 1].parse().expect("--frames <n>"))
        .unwrap_or(60);

    let mut app = App::new();
    app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest()) // pixel art: no filtering
                .set(AssetPlugin {
                    file_path: "../assets".into(), // shared with terra-c
                    ..default()
                })
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
        .insert_resource(drops::GameRng(seed ^ 0xDEADBEEF)) // gameplay stream
        .add_plugins((
            assets::AssetsPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
            mining::MiningPlugin,
            drops::DropsPlugin,
            inventory::InventoryPlugin,
            combat::CombatPlugin,
            enemies::EnemyPlugin,
            hud::HudPlugin,
            state::StatePlugin,
            particles::ParticlesPlugin,
        ));

    if let Some(path) = screenshot {
        app.insert_state(state::GameState::Playing) // skip menu
            .insert_resource(ScreenshotCfg { path, frames_left: frames, grace: 20 })
            .add_systems(Update, screenshot_countdown);
    }

    app.run();
}

#[derive(Resource)]
struct ScreenshotCfg {
    path: String,
    frames_left: i32,
    grace: i32, // frames to let the capture flush before exiting
}

fn screenshot_countdown(
    mut cfg: ResMut<ScreenshotCfg>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    cfg.frames_left -= 1;
    if cfg.frames_left == 0 {
        let path = cfg.path.clone();
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
    if cfg.frames_left < 0 {
        cfg.grace -= 1;
        if cfg.grace <= 0 {
            exit.write(AppExit::Success);
        }
    }
}
