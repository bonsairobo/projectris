use crate::{Config, FallingPiece, PieceMaterials, PieceType, SceneAssets};

use bevy::prelude::*;
use building_blocks::{
    core::prelude::*,
    storage::{access_traits::*, Array2x1, Array2x3},
};
use std::mem::MaybeUninit;

pub struct Grid {
    cells: CellArray,
    projection: Box<dyn Projection>,
    active: bool,
}

pub trait Projection: Fn(Point3i) -> Point2i + 'static + Send + Sync {}
impl<T> Projection for T where T: Fn(Point3i) -> Point2i + 'static + Send + Sync {}

// Left value is the "master copy," which is never show to the player; it's only used for background calculations that don't
// want the falling piece getting in the way. Right value is the "visible copy" that is shown to the player.
pub type CellArray = Array2x3<Entity, CellValue, CellValue>;

#[derive(Clone, Copy, Debug)]
pub enum CellValue {
    Piece(PieceType),
    DropHint,
    Empty,
}

impl CellValue {
    pub fn is_piece(&self) -> bool {
        matches!(self, CellValue::Piece(_))
    }
}

pub struct GridCell;

impl Grid {
    pub fn width(&self) -> i32 {
        self.cells.extent().shape.x()
    }

    pub fn height(&self) -> i32 {
        self.cells.extent().shape.y()
    }

    fn row_extent(&self, row: i32) -> Extent2i {
        Extent2i::from_min_and_shape(PointN([0, row]), PointN([self.width(), 1]))
    }

    fn edit_visible_channel(&mut self) -> Array2x1<CellValue, &mut [CellValue]> {
        self.cells.borrow_channels_mut(|(_, _, v)| v)
    }

    fn edit_master_channel(&mut self) -> Array2x1<CellValue, &mut [CellValue]> {
        self.cells.borrow_channels_mut(|(_, v, _)| v)
    }

    fn read_master_channel(&self) -> Array2x1<CellValue, &[CellValue]> {
        self.cells.borrow_channels(|(_, v, _)| v)
    }

    pub fn copy_master_to_visible(&mut self) {
        let extent = *self.cells.extent();
        self.cells.for_each_mut(&extent, |_: (), (_, m, v)| {
            *v = *m;
        });
    }

    fn copy_visible_to_master(&mut self) {
        let extent = *self.cells.extent();
        self.cells.for_each_mut(&extent, |_: (), (_, m, v)| {
            *m = *v;
        });
    }

    fn commit(&mut self) {
        self.copy_visible_to_master();
        self.eliminate_full_rows();
        self.copy_master_to_visible();
    }

    fn eliminate_full_rows(&mut self) {
        let mut rows_to_check = self.height();
        let mut check_row = 0;

        while check_row < rows_to_check {
            if self.row_is_full(check_row) {
                self.clear_row(check_row);
                self.shift_rows_down(check_row + 1, rows_to_check);
                rows_to_check -= 1;
            } else {
                check_row += 1;
            }
        }
    }

    fn shift_rows_down(&mut self, start_row: i32, end_row: i32) {
        let max_row = self.height() - 1;
        for row in start_row..end_row {
            if row == max_row {
                self.clear_row(row);
                return;
            }
            self.shift_row_down(row);
        }
    }

    fn shift_row_down(&mut self, row: i32) {
        let row = self.row_extent(row);
        let mut master_cells = self.edit_master_channel();
        for p in row.iter_points() {
            let p_val = master_cells.get(p);
            *master_cells.get_mut(p - PointN([0, 1])) = p_val;
        }
    }

    fn clear_row(&mut self, row: i32) {
        let row = self.row_extent(row);
        self.edit_master_channel()
            .fill_extent(&row, CellValue::Empty);
    }

    fn row_is_full(&self, row: i32) -> bool {
        let row = self.row_extent(row);
        let master_cells = self.read_master_channel();

        for p in row.iter_points() {
            if let CellValue::Empty = master_cells.get(p) {
                return false;
            }
        }

        true
    }

    fn any_cells_colliding(&self, check_cells: &[Point2i]) -> bool {
        let master_cells = self.read_master_channel();

        check_cells
            .iter()
            .cloned()
            .any(|p| master_cells.get(p).is_piece())
    }

    fn any_cells_out_of_bounds(&self, check_cells: &[Point2i]) -> bool {
        check_cells
            .iter()
            .cloned()
            .any(|p| !self.cells.extent().contains(p))
    }

    pub fn check_piece_collision(&self, piece: &FallingPiece) -> PieceCollisionResult {
        let projected_cells = self.project_piece(piece);

        if self.any_cells_out_of_bounds(&projected_cells) {
            return PieceCollisionResult::OutOfBounds;
        }

        if self.any_cells_colliding(&projected_cells) {
            return PieceCollisionResult::HitOtherPiece;
        }

        PieceCollisionResult::NoCollision
    }

    pub fn write_piece(&mut self, piece: &FallingPiece) {
        self.write_piece_with_value(piece, CellValue::Piece(piece.piece_type()))
    }

    pub fn write_piece_with_value(&mut self, piece: &FallingPiece, value: CellValue) {
        let projected_cells = self.project_piece(piece);
        let mut visible_cells = self.edit_visible_channel();
        for cell_p in projected_cells.iter().cloned() {
            *visible_cells.get_mut(cell_p) = value;
        }
    }

    pub fn deactivate(&mut self) {
        self.commit();
        self.active = false;
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn project_piece(&self, piece: &FallingPiece) -> Vec<Point2i> {
        piece
            .cell_positions()
            .iter()
            .map(|p| (self.projection)(*p))
            .collect()
    }

    fn sync_cell_materials(
        &self,
        materials: &PieceMaterials,
        cell_material_query: &mut Query<(&GridCell, &mut Handle<StandardMaterial>)>,
    ) {
        self.cells.for_each(
            self.cells.extent(),
            |_: (), (cell_entity, _, visible_value)| {
                let (_, mut material) = cell_material_query.get_mut(cell_entity).unwrap();
                *material = materials.get_cell_material(visible_value);
            },
        );
    }
}

pub enum PieceCollisionResult {
    HitOtherPiece,
    OutOfBounds,
    NoCollision,
}

pub fn create_grids(config: &Config, scene_assets: &SceneAssets, commands: &mut Commands) {
    let grid_size = config.grid_size;

    // All cells are locally in the XY plane, so we rotate the parent entity for each grid to fall into the correct plane,
    // either XY or ZY.

    let left_grid_projection = Box::new(|p: Point3i| p.xy());
    let left_grid_transform = Transform {
        translation: config.grid_offset * -Vec3::Z,
        rotation: Quat::from_axis_angle(Vec3::Y, config.grid_tilt_angle),
        scale: Vec3::ONE,
    };
    spawn_grid(
        grid_size,
        left_grid_projection,
        left_grid_transform,
        commands,
        &scene_assets.piece_materials,
        scene_assets.left_cell_mesh.clone(),
    );

    let right_grid_projection = Box::new(|p: Point3i| p.zy());
    let right_grid_transform = Transform {
        translation: config.grid_offset * -Vec3::X,
        rotation: Quat::from_axis_angle(
            Vec3::Y,
            -(std::f32::consts::FRAC_PI_2 + config.grid_tilt_angle),
        ),
        scale: Vec3::ONE,
    };
    spawn_grid(
        grid_size,
        right_grid_projection,
        right_grid_transform,
        commands,
        &scene_assets.piece_materials,
        scene_assets.right_cell_mesh.clone(),
    );
}

fn spawn_grid(
    grid_size: Point2i,
    projection: Box<dyn Projection>,
    grid_transform: Transform,
    commands: &mut Commands,
    piece_materials: &PieceMaterials,
    cell_mesh: Handle<Mesh>,
) -> Entity {
    let cells = spawn_cells(grid_size, commands, piece_materials, cell_mesh);
    let children = all_cell_entities(&cells);

    commands
        .spawn()
        .insert(Grid {
            cells,
            projection,
            active: true,
        })
        .insert(GlobalTransform::identity())
        .insert(grid_transform)
        .push_children(&children)
        .id()
}

fn spawn_cells(
    shape: Point2i,
    commands: &mut Commands,
    materials: &PieceMaterials,
    cell_mesh: Handle<Mesh>,
) -> CellArray {
    let grid_extent = Extent2i::from_min_and_shape(Point2i::ZERO, shape);
    let mut cells: Array2x3<MaybeUninit<Entity>, MaybeUninit<CellValue>, MaybeUninit<CellValue>> =
        unsafe { Array2x3::maybe_uninit(grid_extent) };

    cells.for_each_mut(
        &grid_extent,
        |p: Point2i, (uninit_entity, uninit_master_value, uninit_visible_value)| {
            let entity = commands
                .spawn()
                .insert(GridCell)
                .insert_bundle(PbrBundle {
                    mesh: cell_mesh.clone(),
                    material: materials.empty_cell_material(),
                    // We have to offset by 0.5 because the cell meshes are centered at (0, 0).
                    transform: Transform::from_xyz(p.x() as f32 + 0.5, p.y() as f32 + 0.5, 0.0),
                    ..Default::default()
                })
                .id();
            unsafe {
                uninit_entity.as_mut_ptr().write(entity);
                uninit_master_value.as_mut_ptr().write(CellValue::Empty);
                uninit_visible_value.as_mut_ptr().write(CellValue::Empty);
            }
        },
    );

    unsafe { cells.assume_init() }
}

fn all_cell_entities(cells: &CellArray) -> Vec<Entity> {
    let mut entities = Vec::new();
    cells
        .borrow_channels(|(e, _, _)| e)
        .for_each(cells.extent(), |_: (), entity| {
            entities.push(entity);
        });

    entities
}

pub fn synchronize_grid_materials(
    grids_query: Query<&Grid>,
    assets: Res<SceneAssets>,
    mut cell_material_query: Query<(&GridCell, &mut Handle<StandardMaterial>)>,
) {
    for grid in grids_query.iter() {
        grid.sync_cell_materials(&assets.piece_materials, &mut cell_material_query);
    }
}
