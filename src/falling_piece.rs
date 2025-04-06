use crate::{
    CellValue, Config, GrabBag, Grid, PieceCollisionResult, PieceMaterials, PieceType, Rotation,
    SceneAssets,
};
use bevy::prelude::*;

#[derive(Clone, Component, Copy, Debug)]
pub struct FallingPiece {
    piece_type: PieceType,
    center_position: IVec3,
    offsets: [IVec3; 3],
}

impl FallingPiece {
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn translate(&mut self, offset: IVec3) {
        self.center_position += offset;
    }

    pub fn translate_n_rows(&mut self, n: i32) {
        self.translate(IVec3::new(0, n, 0));
    }

    pub fn rotate(&mut self, matrix: [[i32; 3]; 3]) {
        let x_map = IVec3::from(matrix[0]);
        let y_map = IVec3::from(matrix[1]);
        let z_map = IVec3::from(matrix[2]);

        for p in self.offsets.iter_mut() {
            *p = IVec3::new(x_map.dot(*p), y_map.dot(*p), z_map.dot(*p));
        }
    }

    pub fn cell_positions(&self) -> [IVec3; 4] {
        let mut positions = [self.center_position; 4];
        for i in 0..3 {
            positions[i + 1] = self.center_position + self.offsets[i];
        }

        positions
    }
}

pub fn spawn_falling_piece(
    grid_shape: [usize; 2],
    grab_bag: &mut GrabBag,
    materials: &PieceMaterials,
    cube_mesh: Handle<Mesh>,
    grid_query: &mut Query<&mut Grid>,
    commands: &mut Commands,
) -> Entity {
    let [shape_x, shape_y] = grid_shape;
    let center_position = IVec3::new(shape_x as i32 / 2, shape_y as i32 - 1, shape_x as i32 / 2);
    let piece_type = grab_bag.choose_next_piece_type();
    let child_cubes = piece_type.cube_configuration();

    let center_cube = cube_pbr(
        piece_type,
        // Offset by 0.5 because the cube is centered at 0.
        center_position.as_vec3() + Vec3::splat(0.5),
        materials,
        cube_mesh.clone(),
    );
    let child_cube_entities: Vec<Entity> = child_cubes
        .iter()
        .map(|cube_offset| {
            commands
                .spawn(cube_pbr(
                    piece_type,
                    IVec3::from(*cube_offset).as_vec3(),
                    materials,
                    cube_mesh.clone(),
                ))
                .id()
        })
        .collect();

    let piece = FallingPiece {
        piece_type,
        center_position,
        offsets: child_cubes.map(Into::into),
    };

    for mut grid in grid_query.iter_mut() {
        grid.activate();
        grid.write_piece(&piece);
    }

    write_drop_hint_in_active_grids(&piece, grid_query);

    commands
        .spawn_empty()
        .insert(piece)
        .insert(center_cube)
        .add_children(&child_cube_entities)
        .id()
}

fn cube_pbr(
    piece_type: PieceType,
    offset: Vec3,
    materials: &PieceMaterials,
    mesh: Handle<Mesh>,
) -> (MeshMaterial3d<StandardMaterial>, Mesh3d, Transform) {
    (
        MeshMaterial3d(materials.get_piece_material(piece_type)),
        Mesh3d(mesh),
        Transform::from_translation(offset),
    )
}

#[derive(Clone, Debug, Event)]
pub enum FallingPieceEvent {
    Spawn,
    Drop,
    FastDrop,
    Translate(IVec3),
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
    for event in events.read() {
        // Reset the visible copy of the grid.
        for mut grid in grid_query.iter_mut() {
            grid.copy_master_to_visible();
        }

        if let FallingPieceEvent::Spawn = event {
            spawn_falling_piece(
                config.grid_size,
                &mut grab_bag,
                &scene_assets.piece_materials,
                scene_assets.cube_mesh.clone(),
                &mut grid_query,
                &mut commands,
            );
            return;
        }

        for (piece_entity, mut piece, mut tfm) in falling_piece_query.iter_mut() {
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
            if any_active_grids {
                write_drop_hint_in_active_grids(&piece, &mut grid_query);
            } else {
                commands.entity(piece_entity).despawn_recursive();
                spawn_falling_piece(
                    config.grid_size,
                    &mut grab_bag,
                    &scene_assets.piece_materials,
                    scene_assets.cube_mesh.clone(),
                    &mut grid_query,
                    &mut commands,
                );
                // This return is important. It's possible that there are more events to be processed, but they don't apply to
                // the piece entity we just despawned. That stale entity still exists in our query.
                return;
            }
        }
    }
}

fn write_piece_to_active_grids(piece: &FallingPiece, grid_query: &mut Query<&mut Grid>) {
    for mut grid in grid_query.iter_mut() {
        if grid.is_active() {
            grid.write_piece(piece);
        }
    }
}

fn write_drop_hint_in_active_grids(piece: &FallingPiece, grid_query: &mut Query<&mut Grid>) {
    for mut grid in grid_query.iter_mut() {
        if grid.is_active() {
            let dropped_piece = speculate_fast_drop_piece(piece, &grid);
            grid.write_piece_with_value(&dropped_piece, CellValue::DropHint);
        }
    }
}

fn try_drop_piece(
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let mut new_piece = *piece;
    new_piece.translate_n_rows(-1);

    if move_accepted_in_all_active_grids(piece, &new_piece, true, grid_query) {
        tfm.translation -= Vec3::Y;
        *piece = new_piece;
    }
}

fn fast_drop_piece(
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let rows_dropped = fast_drop_piece_in_all_active_grids(piece, true, grid_query);
    tfm.translation -= rows_dropped as f32 * Vec3::Y;
}

fn speculate_fast_drop_piece(piece: &FallingPiece, grid: &Grid) -> FallingPiece {
    let mut moved_piece = *piece;
    fast_drop_piece_in_grid(&mut moved_piece, grid);

    moved_piece
}

fn fast_drop_piece_in_all_active_grids(
    piece: &mut FallingPiece,
    commit_when_stuck: bool,
    grid_query: &mut Query<&mut Grid>,
) -> i32 {
    let mut rows_dropped = 0;

    loop {
        let mut new_piece = *piece;
        new_piece.translate_n_rows(-1);

        if move_accepted_in_all_active_grids(piece, &new_piece, commit_when_stuck, grid_query) {
            rows_dropped += 1;
            *piece = new_piece;
        }

        let any_grids_active = grid_query.iter_mut().any(|g| g.is_active());
        if !any_grids_active {
            break;
        }
    }

    rows_dropped
}

fn fast_drop_piece_in_grid(piece: &mut FallingPiece, grid: &Grid) {
    loop {
        piece.translate_n_rows(-1);
        if !move_accepted_in_grid(piece, grid) {
            piece.translate_n_rows(1);
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

    if move_accepted_in_all_active_grids(piece, &new_piece, false, grid_query) {
        tfm.rotation = rotation.quat * tfm.rotation;
        *piece = new_piece;
    }
}

fn try_translate_piece(
    translation: IVec3,
    piece: &mut FallingPiece,
    tfm: &mut Transform,
    grid_query: &mut Query<&mut Grid>,
) {
    let mut new_piece = *piece;
    new_piece.translate(translation);

    if move_accepted_in_all_active_grids(piece, &new_piece, false, grid_query) {
        tfm.translation += translation.as_vec3();
        *piece = new_piece;
    }
}

fn move_accepted_in_all_active_grids(
    old_piece: &FallingPiece,
    new_piece: &FallingPiece,
    commit_when_stuck: bool,
    grid_query: &mut Query<&mut Grid>,
) -> bool {
    let mut move_accepted_in_all_active_grids = true;
    for mut grid in grid_query.iter_mut() {
        if !grid.is_active() {
            continue;
        }

        if !move_accepted_in_grid(new_piece, &grid) {
            if commit_when_stuck {
                grid.write_piece(old_piece);
                grid.deactivate();
            }
            move_accepted_in_all_active_grids = false;
        }
    }

    move_accepted_in_all_active_grids
}

fn move_accepted_in_grid(new_piece: &FallingPiece, grid: &Grid) -> bool {
    match grid.check_piece_collision(new_piece) {
        PieceCollisionResult::NoCollision => true,
        PieceCollisionResult::OutOfBounds | PieceCollisionResult::HitOtherPiece => false,
    }
}
