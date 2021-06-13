use crate::PieceType;

use bevy::prelude::*;

pub struct SceneAssets {
    pub left_cell_mesh: Handle<Mesh>,
    pub right_cell_mesh: Handle<Mesh>,
    pub cube_mesh: Handle<Mesh>,
    pub piece_materials: PieceMaterials,
}

pub fn create_scene_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
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

    let piece_materials = PieceMaterials::new(&server, &mut materials);

    commands.insert_resource(SceneAssets {
        left_cell_mesh,
        right_cell_mesh,
        cube_mesh,
        piece_materials,
    });
}

pub struct PieceMaterials {
    cell_materials: Vec<Handle<StandardMaterial>>,
    piece_materials: Vec<Handle<StandardMaterial>>,
    empty_cell_material: Handle<StandardMaterial>,
    drop_hint_material: Handle<StandardMaterial>,
}

impl PieceMaterials {
    pub fn new(server: &AssetServer, materials: &mut Assets<StandardMaterial>) -> Self {
        let empty_cell_material = materials.add(cell_material(Color::GRAY));
        let drop_hint_material = materials.add(cell_material(Color::DARK_GRAY));

        let piece_colors = [
            Color::RED,
            Color::GREEN,
            Color::BLUE,
            Color::YELLOW,
            Color::CYAN,
            Color::PINK,
            Color::ORANGE,
            Color::PURPLE,
        ];

        let cell_materials = piece_colors
            .iter()
            .cloned()
            .map(|c| materials.add(cell_material(c)))
            .collect();

        let bordered_texture: Handle<Texture> = server.load("BorderedTile.png");
        let piece_materials = piece_colors
            .iter()
            .cloned()
            .map(|c| materials.add(piece_material(c, bordered_texture.clone())))
            .collect();

        Self {
            cell_materials,
            piece_materials,
            empty_cell_material,
            drop_hint_material,
        }
    }

    pub fn get_piece_material(&self, piece_type: PieceType) -> Handle<StandardMaterial> {
        self.piece_materials[piece_type as usize].clone()
    }

    pub fn get_cell_material(&self, piece_type: PieceType) -> Handle<StandardMaterial> {
        self.cell_materials[piece_type as usize].clone()
    }

    pub fn empty_cell_material(&self) -> Handle<StandardMaterial> {
        self.empty_cell_material.clone()
    }

    pub fn drop_hint_material(&self) -> Handle<StandardMaterial> {
        self.drop_hint_material.clone()
    }
}

fn cell_material(color: Color) -> StandardMaterial {
    let mut m = StandardMaterial::from(color);
    m.unlit = true;

    m
}

fn piece_material(color: Color, texture: Handle<Texture>) -> StandardMaterial {
    let mut m = StandardMaterial::from(texture);
    m.unlit = true;
    m.base_color = color;

    m
}
