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
