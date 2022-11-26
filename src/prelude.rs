pub use crate::state::*;
pub use bevy::log;
pub use bevy::math::*;
pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use bevy_sprite3d::*;

pub mod config {
    pub const GROUND_SIZE: f32 = 12.0;
    pub const GROUND_DEPTH: f32 = 1.0;
    pub const PLAYER_SPEED: f32 = 500.0;
    pub const PARCEL_SPAWN_RATE: u64 = 1000;
    pub const PARCEL_SPAWN_Y: f32 = 10.0;
    pub const PARCEL_MAX_LINVEL_X: f32 = 0.3;
    pub const PARCEL_MAX_LINVEL_Z: f32 = 0.3;
    pub const PARCEL_MIN_LINVEL_X: f32 = 0.2;
    pub const PARCEL_MIN_LINVEL_Z: f32 = 0.2;
    pub const PARCEL_MAX_ANGVEL: f32 = 0.0;
    pub const PARCEL_SIZE: f32 = 1.0;
}
