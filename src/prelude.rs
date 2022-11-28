pub use crate::debug::*;
pub use crate::state::*;
pub use bevy::log;
pub use bevy::math::*;
pub use bevy::prelude::*;
pub use bevy_mod_outline::*;
pub use bevy_prototype_debug_lines::*;
pub use bevy_rapier3d::prelude::*;
pub use bevy_sprite3d::*;
pub use bevy_tweening::*;

pub mod config {
    pub const GROUND_SIZE: f32 = 12.0;
    pub const GROUND_DEPTH: f32 = 1.0;

    pub const PLAYER_SPEED: f32 = 1000.0;
    pub const PLAYER_THROW_FACTOR: f32 = 1.15;
    pub const PLAYER_MAX_THROW_MAQ: f32 = 12.0;

    pub const PARCEL_SPAWN_RATE: u64 = 3000;
    pub const PARCEL_SPAWN_Y: f32 = 14.0;
    pub const PARCEL_MAX_LINVEL_X: f32 = 0.3;
    pub const PARCEL_MAX_LINVEL_Z: f32 = 0.3;
    pub const PARCEL_MIN_LINVEL_X: f32 = 0.2;
    pub const PARCEL_MIN_LINVEL_Z: f32 = 0.2;
    pub const PARCEL_MAX_ANGVEL: f32 = 0.0;
    pub const PARCEL_SIZE: f32 = 1.0;

    pub const PARCEL_LEVEL_UP: u64 = 3;
    pub const PARCEL_LEVEL_UP_DECR: u64 = 350;
    pub const PARCEL_LEVEL_UP_MIN: u64 = 2300;

    pub const GAME_TIME: f32 = 128.0;
}
