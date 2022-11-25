pub use bevy::log;
pub use bevy::math::*;
pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use bevy_sprite3d::*;

pub mod config {
    pub const GROUND_SIZE: f32 = 10.0;
    pub const GROUND_DEPTH: f32 = 0.05;
    pub const PLAYER_SPEED: f32 = 5.0;
}
