use crate::PieceMaterials;

use bevy::prelude::*;

pub struct SceneAssets {
    pub left_cell_mesh: Handle<Mesh>,
    pub right_cell_mesh: Handle<Mesh>,
    pub cube_mesh: Handle<Mesh>,
    pub piece_materials: PieceMaterials,
}

pub fn create_scene_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let left_cell_mesh = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::ONE,
        flip: true,
    }));
    let right_cell_mesh = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::ONE,
        flip: false,
    }));
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    let piece_materials = PieceMaterials::new(&mut *materials);

    commands.insert_resource(SceneAssets {
        left_cell_mesh,
        right_cell_mesh,
        cube_mesh,
        piece_materials,
    });
}
