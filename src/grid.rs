use crate::{Config, FallingPiece, PieceMaterials, PieceType, SceneAssets};
use bevy::prelude::*;

// The `master` copy is never show to the player; it's only used for background
// calculations that don't want the falling piece getting in the way.
//
// The `visible` copy is shown to the player.
#[derive(Component)]
pub struct Grid {
    extent: Extent,
    master: Vec<CellValue>,
    visible: Vec<CellValue>,
    entities: Vec<Entity>,
    projection: Box<dyn Projection>,
    active: bool,
}

pub trait Projection: Fn(IVec3) -> IVec2 + 'static + Send + Sync {}
impl<T> Projection for T where T: Fn(IVec3) -> IVec2 + 'static + Send + Sync {}

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

#[derive(Component)]
pub struct GridCell;

impl Grid {
    pub fn width(&self) -> i32 {
        self.extent.shape.x
    }

    pub fn height(&self) -> i32 {
        self.extent.shape.y
    }

    fn row_extent(&self, row: i32) -> Extent {
        Extent::new([0, row].into(), [self.width(), 1].into())
    }

    fn edit_visible(&mut self) -> &mut [CellValue] {
        &mut self.visible
    }

    fn edit_master(&mut self) -> &mut [CellValue] {
        &mut self.master
    }

    fn read_master_channel(&self) -> &[CellValue] {
        &self.master
    }

    pub fn copy_master_to_visible(&mut self) {
        self.visible.copy_from_slice(&self.master);
    }

    fn copy_visible_to_master(&mut self) {
        self.master.copy_from_slice(&self.visible);
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
        let shape = self.extent.shape;
        let row = self.row_extent(row);
        let master_cells = self.edit_master();
        for p in row.iter_points() {
            let p_val = master_cells[index2(shape, p)];
            master_cells[index2(shape, p - IVec2::new(0, 1))] = p_val;
        }
    }

    fn clear_row(&mut self, row: i32) {
        let shape = self.extent.shape;
        let row = self.row_extent(row);
        let master_cells = self.edit_master();
        for p in row.iter_points() {
            master_cells[index2(shape, p)] = CellValue::Empty;
        }
    }

    fn row_is_full(&self, row: i32) -> bool {
        let shape = self.extent.shape;
        let row = self.row_extent(row);
        let master_cells = self.read_master_channel();
        for p in row.iter_points() {
            if let CellValue::Empty = master_cells[index2(shape, p)] {
                return false;
            }
        }
        true
    }

    fn any_cells_colliding(&self, check_cells: &[IVec2]) -> bool {
        let shape = self.extent.shape;
        let master_cells = self.read_master_channel();
        check_cells
            .iter()
            .cloned()
            .any(|p| master_cells[index2(shape, p)].is_piece())
    }

    fn any_cells_out_of_bounds(&self, check_cells: &[IVec2]) -> bool {
        check_cells
            .iter()
            .cloned()
            .any(|p| !self.extent.contains(p))
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
        let shape = self.extent.shape;
        let projected_cells = self.project_piece(piece);
        let visible_cells = self.edit_visible();
        for cell_p in projected_cells.iter().cloned() {
            visible_cells[index2(shape, cell_p)] = value;
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

    fn project_piece(&self, piece: &FallingPiece) -> Vec<IVec2> {
        piece
            .cell_positions()
            .iter()
            .map(|p| (self.projection)(*p))
            .collect()
    }

    fn sync_cell_materials(
        &self,
        materials: &PieceMaterials,
        cell_material_query: &mut Query<(&GridCell, &mut MeshMaterial3d<StandardMaterial>)>,
    ) {
        let shape = self.extent.shape;
        for p in self.extent.iter_points() {
            let i = index2(shape, p);
            let cell_entity = self.entities[i];
            let visible_value = self.visible[i];
            let (_, mut material) = cell_material_query.get_mut(cell_entity).unwrap();
            material.0 = materials.get_cell_material(visible_value);
        }
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

    let left_grid_projection = Box::new(|p: IVec3| p.xy());
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

    let right_grid_projection = Box::new(|p: IVec3| p.zy());
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
    grid_size: [usize; 2],
    projection: Box<dyn Projection>,
    grid_transform: Transform,
    commands: &mut Commands,
    piece_materials: &PieceMaterials,
    cell_mesh: Handle<Mesh>,
) -> Entity {
    let shape = IVec2::new(grid_size[0] as i32, grid_size[1] as i32);
    let extent = Extent::new(IVec2::ZERO, shape);
    let n_cells = extent.size();
    let master = vec![CellValue::Empty; n_cells];
    let visible = vec![CellValue::Empty; n_cells];
    let entities = spawn_cells(extent, commands, piece_materials, cell_mesh);

    commands
        .spawn_empty()
        .add_children(&entities)
        .insert(Grid {
            extent,
            entities,
            master,
            visible,
            projection,
            active: true,
        })
        .insert(GlobalTransform::default())
        .insert(grid_transform)
        .id()
}

fn spawn_cells(
    extent: Extent,
    commands: &mut Commands,
    materials: &PieceMaterials,
    cell_mesh: Handle<Mesh>,
) -> Vec<Entity> {
    let n_cells = extent.size();
    let mut entities = vec![Entity::PLACEHOLDER; n_cells];

    for p in extent.iter_points() {
        let i = index2(extent.shape, p);
        let entity = commands
            .spawn(GridCell)
            .insert(Mesh3d(cell_mesh.clone()))
            .insert(MeshMaterial3d(materials.empty_cell_material()))
            // We have to offset by 0.5 because the cell meshes are centered at (0, 0).
            .insert(Transform::from_xyz(p.x as f32 + 0.5, p.y as f32 + 0.5, 0.0))
            .id();
        entities[i] = entity;
    }
    entities
}

pub fn synchronize_grid_materials(
    grids_query: Query<&Grid>,
    assets: Res<SceneAssets>,
    mut cell_material_query: Query<(&GridCell, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for grid in grids_query.iter() {
        grid.sync_cell_materials(&assets.piece_materials, &mut cell_material_query);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Extent {
    min: IVec2,
    shape: IVec2,
}

impl Extent {
    fn new(min: IVec2, shape: IVec2) -> Self {
        Self { min, shape }
    }

    fn size(&self) -> usize {
        (self.shape.x * self.shape.y) as usize
    }

    fn max(&self) -> IVec2 {
        (self.min + self.shape) - IVec2::ONE
    }

    fn contains(&self, p: IVec2) -> bool {
        let max = self.max();
        self.min.x < p.x && p.x <= max.x && self.min.y < p.y && p.y <= max.y
    }

    fn iter_points(&self) -> impl Iterator<Item = IVec2> {
        let &Self { min, shape } = self;
        let sup = min + shape;
        (min.y..sup.y).flat_map(move |y| (min.x..sup.x).map(move |x| IVec2::new(x, y)))
    }
}

fn index2(shape: IVec2, p: IVec2) -> usize {
    (shape.x * p.y + p.x) as usize
}
