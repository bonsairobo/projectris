use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceType {
    Stick,
    Box,
    Jay,
    Zee,
    Tee,
    Chi,
    Ral,
    Legs,
}

pub const ALL_PIECE_TYPES: [PieceType; 8] = [
    PieceType::Stick,
    PieceType::Box,
    PieceType::Jay,
    PieceType::Zee,
    PieceType::Tee,
    PieceType::Chi,
    PieceType::Ral,
    PieceType::Legs,
];

impl PieceType {
    pub fn cube_configuration(&self) -> [[i32; 3]; 3] {
        CUBE_CONFIGURATIONS[*self as usize]
    }
}

pub struct PieceMaterials {
    piece_materials: [Handle<StandardMaterial>; 8],
    empty_material: Handle<StandardMaterial>,
}

impl PieceMaterials {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let empty_material = materials.add(cell_material(Color::rgba(0.2, 0.2, 0.2, 0.5)));

        let piece_materials = [
            materials.add(cell_material(Color::rgb(0.0, 0.0, 0.0))),
            materials.add(cell_material(Color::rgb(1.0, 0.0, 0.0))),
            materials.add(cell_material(Color::rgb(0.0, 1.0, 0.0))),
            materials.add(cell_material(Color::rgb(0.0, 0.0, 1.0))),
            materials.add(cell_material(Color::rgb(1.0, 1.0, 0.0))),
            materials.add(cell_material(Color::rgb(1.0, 0.7, 0.0))),
            materials.add(cell_material(Color::rgb(0.0, 1.0, 1.0))),
            materials.add(cell_material(Color::rgb(1.0, 0.0, 1.0))),
        ];

        Self {
            piece_materials,
            empty_material,
        }
    }

    pub fn get(&self, piece_type: PieceType) -> Handle<StandardMaterial> {
        self.piece_materials[piece_type as usize].clone()
    }

    pub fn empty_material(&self) -> Handle<StandardMaterial> {
        self.empty_material.clone()
    }
}

fn cell_material(color: Color) -> StandardMaterial {
    let mut m = StandardMaterial::from(color);
    m.unlit = true;

    m
}

const CUBE_CONFIGURATIONS: [[[i32; 3]; 3]; 8] = [
    [[-1, 0, 0], [1, 0, 0], [2, 0, 0]],
    [[0, 0, 1], [1, 0, 0], [1, 0, 1]],
    [[-1, 0, 0], [1, 0, 0], [1, 0, 1]],
    [[-1, 0, 0], [0, 0, 1], [1, 0, 1]],
    [[-1, 0, 0], [1, 0, 0], [0, 0, 1]],
    [[-1, 0, 0], [0, 0, 1], [0, -1, 1]],
    [[1, 0, 0], [0, 0, 1], [0, -1, 1]],
    [[-1, 0, 0], [0, 0, 1], [0, -1, 0]],
];
