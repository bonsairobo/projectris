use bevy::math::{Quat, Vec3};

#[derive(Clone, Debug)]
pub struct Rotation {
    pub matrix: [[i32; 3]; 3],
    pub quat: Quat,
}

impl Rotation {
    pub fn rotate_x_pos_90() -> Self {
        Self {
            matrix: [[1, 0, 0], [0, 0, -1], [0, 1, 0]],
            quat: Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2),
        }
    }
    pub fn rotate_x_neg_90() -> Self {
        Self {
            matrix: [[1, 0, 0], [0, 0, 1], [0, -1, 0]],
            quat: Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2),
        }
    }
    pub fn rotate_z_pos_90() -> Self {
        Self {
            matrix: [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
            quat: Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_2),
        }
    }
    pub fn rotate_z_neg_90() -> Self {
        Self {
            matrix: [[0, 1, 0], [-1, 0, 0], [0, 0, 1]],
            quat: Quat::from_axis_angle(Vec3::Z, -std::f32::consts::FRAC_PI_2),
        }
    }
}
