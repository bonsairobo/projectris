use bevy::math::Vec3;
use building_blocks::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub grid_size: Point2i,
    pub grid_offset: f32,
    pub grid_tilt_angle: f32,
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub repeats_per_bag: usize,
}

impl Config {
    pub fn read_file(path: &str) -> Result<Self, ron::Error> {
        let reader = std::fs::File::open(path)?;

        ron::de::from_reader(reader)
    }
}
