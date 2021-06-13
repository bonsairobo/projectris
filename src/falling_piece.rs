use crate::{
    Config, GrabBag, Grid, PieceMaterials, PieceMovementResult, PieceType, Rotation, SceneAssets,
};

use bevy::prelude::*;
use building_blocks::core::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct FallingPiece {
    piece_type: PieceType,
    center_position: Point3i,
    offsets: [Point3i; 3],
}

impl FallingPiece {
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn translate(&mut self, offset: Point3i) {
        self.center_position += offset;
    }

    pub fn translate_n_rows(&mut self, n: i32) {
        self.translate(PointN([0, n, 0]));
    }

    pub fn rotate(&mut self, matrix: [[i32; 3]; 3]) {
        let x_map = PointN(matrix[0]);
        let y_map = PointN(matrix[1]);
        let z_map = PointN(matrix[2]);

        for p in self.offsets.iter_mut() {
            *p = PointN([x_map.dot(*p), y_map.dot(*p), z_map.dot(*p)]);
        }
    }

    pub fn cell_positions(&self) -> [Point3i; 4] {
        let mut positions = [self.center_position; 4];
        for i in 0..3 {
            positions[i + 1] = self.center_position + self.offsets[i];
        }

        positions
    }
}

pub fn spawn_falling_piece(
    grid_size: Point2i,
    grab_bag: &mut GrabBag,
    materials: &PieceMaterials,
    cube_mesh: Handle<Mesh>,
    grids_query: &mut Query<&mut Grid>,
    commands: &mut Commands,
) -> Entity {
    for mut grid in grids_query.iter_mut() {
        grid.eliminate_full_rows();
    }

    let center_position = PointN([grid_size.x() / 2, grid_size.y() - 1, grid_size.x() / 2]);
    let piece_type = grab_bag.choose_next_piece_type();
    let child_cubes = piece_type.cube_configuration();

    let center_cube = cube_pbr(
        piece_type,
        // Offset by 0.5 because the cube is centered at 0.
        Vec3::from(Point3f::from(center_position)) + Vec3::splat(0.5),
        materials,
        cube_mesh.clone(),
    );
    let child_cube_entities: Vec<Entity> = child_cubes
        .iter()
        .map(|cube_offset| {
            commands
                .spawn_bundle(cube_pbr(
                    piece_type,
                    Vec3::from(Point3f::from(PointN(*cube_offset))),
                    materials,
                    cube_mesh.clone(),
                ))
                .id()
        })
        .collect();

    let piece = FallingPiece {
        piece_type,
        center_position,
        offsets: [
            PointN(child_cubes[0]),
            PointN(child_cubes[1]),
            PointN(child_cubes[2]),
        ],
    };

    for mut grid in grids_query.iter_mut() {
        grid.activate();
        grid.write_piece(&piece);
    }

    commands
        .spawn()
        .insert(piece)
        .insert_bundle(center_cube)
        .push_children(&child_cube_entities)
        .id()
}

fn cube_pbr(
    piece_type: PieceType,
    offset: Vec3,
    materials: &PieceMaterials,
    mesh: Handle<Mesh>,
) -> PbrBundle {
    PbrBundle {
        material: materials.get_piece_material(piece_type),
        mesh,
        transform: Transform::from_translation(offset),
        ..Default::default()
    }
}

#[derive(Clone)]
pub enum FallingPieceEvent {
    Spawn,
    Drop,
    FastDrop,
    Translate(Point3i),
    Rotate(Rotation),
}

pub fn update_falling_piece(
    mut commands: Commands,
    mut events: EventReader<FallingPieceEvent>,
    mut falling_piece_query: Query<(Entity, &mut FallingPiece, &mut Transform)>,
    mut grid_query: Query<&mut Grid>,
    mut grab_bag: ResMut<GrabBag>,
    scene_assets: Res<SceneAssets>,
    config: Res<Config>,
) {
    for event in events.iter() {
        if let FallingPieceEvent::Spawn = event {
            spawn_falling_piece(
                config.grid_size,
                &mut grab_bag,
                &scene_assets.piece_materials,
                scene_assets.cube_mesh.clone(),
                &mut grid_query,
                &mut commands,
            );
            continue;
        }

        for (piece_entity, mut piece, mut tfm) in falling_piece_query.iter_mut() {
            erase_piece_from_active_grids(&piece, &mut grid_query);

            match event.clone() {
                FallingPieceEvent::Drop => try_drop_piece(&mut piece, &mut tfm, &mut grid_query),
                FallingPieceEvent::FastDrop => {
                    fast_drop_piece(&mut piece, &mut tfm, &mut grid_query)
                }
                FallingPieceEvent::Rotate(rotation) => {
                    try_rotate_piece(rotation, &mut piece, &mut tfm, &mut grid_query)
                }
                FallingPieceEvent::Translate(translation) => {
                    try_translate_piece(translation, &mut piece, &mut tfm, &mut grid_query)
                }
                FallingPieceEvent::Spawn => {
                    unreachable!()
                }
            }

            write_piece_to_active_grids(&piece, &mut grid_query);

            let any_active_grids = grid_query.iter_mut().any(|g| g.is_active());
            if !any_active_grids {
                commands.entity(piece_entity).despawn_recursive();
                spawn_falling_piece(
                    config.grid_size,
                    &mut grab_bag,
                    &scene_assets.piece_materials,
                    scene_assets.cube_mesh.clone(),
                    &mut grid_query,
                    &mut commands,
                );
            }
        }
    }
}

fn write_piece_to_active_grids(piece: &FallingPiece, grid_query: &mut Query<&mut Grid>) {
    for mut grid in grid_query.iter_mut() {
        if !grid.is_active() {
            continue;
        }
        grid.write_piece(piece);
    }
}

fn erase_piece_from_active_grids(piece: &FallingPiece, grid_query: &mut Query<&mut Grid>) {
    for mut grid in grid_query.iter_mut() {
        if !grid.is_active() {
            continue;
        }
        grid.erase_piece(piece);
    }
}

fn try_drop_piece(
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let mut new_piece = *piece;
    new_piece.translate_n_rows(-1);

    if check_movement(piece, &new_piece, true, grid_query) {
        tfm.translation -= Vec3::Y;
        *piece = new_piece;
    }
}

fn fast_drop_piece(
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    loop {
        let mut new_piece = *piece;
        new_piece.translate_n_rows(-1);

        if check_movement(piece, &new_piece, true, grid_query) {
            tfm.translation -= Vec3::Y;
            *piece = new_piece;
        }

        let any_grids_active = grid_query.iter_mut().any(|g| g.is_active());
        if !any_grids_active {
            return;
        }
    }
}

fn try_rotate_piece(
    rotation: Rotation,
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let mut new_piece = *piece;
    new_piece.rotate(rotation.matrix);

    if check_movement(piece, &new_piece, false, grid_query) {
        tfm.rotate(rotation.quat);
        *piece = new_piece;
    }
}

fn try_translate_piece(
    translation: Point3i,
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let mut new_piece = *piece;
    new_piece.translate(translation);

    if check_movement(piece, &new_piece, false, grid_query) {
        tfm.translation += Vec3::from(Point3f::from(translation));
        *piece = new_piece;
    }
}

fn check_movement(
    old_piece: &FallingPiece,
    new_piece: &FallingPiece,
    moved_by_gravity: bool,
    grid_query: &mut Query<&mut Grid>,
) -> bool {
    let mut move_accepted_in_all_active_grids = true;
    for mut grid in grid_query.iter_mut() {
        if !grid.is_active() {
            continue;
        }

        match grid.handle_piece_movement(new_piece) {
            PieceMovementResult::ValidMovement => {}
            PieceMovementResult::OutOfBounds | PieceMovementResult::HitOtherPiece => {
                if moved_by_gravity {
                    grid.deactivate();
                    grid.write_piece(old_piece);
                }

                move_accepted_in_all_active_grids = false;
            }
        }
    }

    move_accepted_in_all_active_grids
}
