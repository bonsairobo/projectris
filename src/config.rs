use bevy::{
    math::Vec3,
    prelude::{KeyCode, Resource},
};

#[derive(Resource, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub grid_size: [usize; 2],
    pub grid_offset: f32,
    pub grid_tilt_angle: f32,
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub repeats_per_bag: usize,
    pub left_rotate_modifier: KeyCode,
    pub left_translate_modifier: KeyCode,
    pub right_translate_modifier: KeyCode,
    pub right_rotate_modifier: KeyCode,
}

impl Config {
    pub fn read_file(path: &str) -> Result<Self, ron::Error> {
        let reader = std::fs::File::open(path)?;

        ron::de::from_reader(reader)
    }
}
