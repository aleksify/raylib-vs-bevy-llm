// Shared constants — must match terra-c/src/game.h exactly

pub const TILE_SIZE: f32 = 16.0;
pub const WORLD_W: i32 = 1024;
pub const WORLD_H: i32 = 256;
#[allow(dead_code)]
pub const CHUNK_SIZE: i32 = 32;
pub const GRAVITY: f32 = 900.0;
pub const PLAYER_SPEED: f32 = 140.0;
pub const PLAYER_JUMP_VEL: f32 = -320.0;
#[allow(dead_code)]
pub const PLAYER_REACH: i32 = 5;
pub const PLAYER_MAX_HP: i32 = 100;
pub const WINDOW_W: f32 = 1280.0;
pub const WINDOW_H: f32 = 720.0;
pub const CAMERA_ZOOM: f32 = 2.0;

pub const PLAYER_BOX_W: f32 = 12.0;
pub const PLAYER_BOX_H: f32 = 22.0;
