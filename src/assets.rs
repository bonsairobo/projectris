use crate::{CellValue, PieceType};
use bevy::{color::palettes::css, prelude::*};

#[derive(Resource)]
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
    let left_cell_mesh = meshes.add(
        Mesh::from(Rectangle {
            half_size: 0.5 * Vec2::ONE,
        })
        .with_inverted_winding()
        .unwrap(),
    );
    let right_cell_mesh = meshes.add(Mesh::from(Rectangle {
        half_size: 0.5 * Vec2::ONE,
    }));
    let cube_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: 0.5 * Vec3::ONE,
    }));

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
        let empty_cell_material = materials.add(cell_material(css::GRAY.into()));
        let drop_hint_material = materials.add(cell_material(css::DARK_GRAY.into()));

        let piece_colors = [
            css::RED,
            css::GREEN,
            css::BLUE,
            css::YELLOW,
            css::LIGHT_CYAN,
            css::PINK,
            css::ORANGE,
            css::PURPLE,
        ]
        .map(Color::from);

        let cell_materials = piece_colors
            .iter()
            .cloned()
            .map(|c| materials.add(cell_material(c)))
            .collect();

        let bordered_texture: Handle<Image> = server.load("BorderedTile.png");
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

    pub fn get_cell_material(&self, cell_value: CellValue) -> Handle<StandardMaterial> {
        match cell_value {
            CellValue::Piece(piece_type) => self.cell_materials[piece_type as usize].clone(),
            CellValue::DropHint => self.drop_hint_material(),
            CellValue::Empty => self.empty_cell_material(),
        }
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

fn piece_material(color: Color, texture: Handle<Image>) -> StandardMaterial {
    let mut m = StandardMaterial::from(texture);
    m.unlit = true;
    m.base_color = color;
    m
}
