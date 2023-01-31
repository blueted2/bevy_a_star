#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display, sync::Arc,
};

use rand::Rng;

use itertools::Itertools;

use bevy::{prelude::*, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, time::FixedTimestep};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin)
        .register_type::<CellPos>()
        .add_startup_system(setup)
        .add_startup_system(spawn_grid)
        .add_system(update_cells)
        .add_system(grid_added)
        .add_system_set(
            SystemSet::new()
            .with_run_criteria(FixedTimestep::step(0.00001))
            .with_system(randomize_cells)
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Component, Reflect)]
struct CellPos(i32, i32);

#[derive(Component, Debug, Copy, Clone)]
struct Cell {
    is_wall: bool,
}

#[derive(Debug, Component)]
struct Grid {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[derive(Component)]
struct GridView {
    grid: Arc<Grid>,
}

#[derive(Component)]
struct GridEditor {
    grid: Arc<Grid>,
}

// struct AStar<'a> {
//     grid: &'a Grid,
//     open_set: HashSet<CellPos>,
//     came_from: HashMap<CellPos, CellPos>,

//     g_score: HashMap<CellPos, f32>,
//     f_score: HashMap<CellPos, f32>,
// }

#[derive(Debug)]
struct OutOfBounds {
    cell_pos: CellPos,
}

impl Display for OutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell_pos = self.cell_pos;
        write!(f, "out of bounds cell position: {cell_pos:?}")
    }
}
impl Error for OutOfBounds {}


impl Grid {
    fn new(width: u32, height: u32) -> Self {
        let cells: Vec<Cell> = (0..height*width)
            .map(|_| Cell {is_wall: false})
            .collect();

        Grid {
            width,
            height,
            cells,
        }
    }

    fn contains_pos(&self, cell_pos: CellPos) -> bool {
        let CellPos(x, y) = cell_pos;
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    fn cell_pos_to_index(&self, cell_pos: CellPos) -> Result<usize, OutOfBounds> {
        if !self.contains_pos(cell_pos) {
            return Err(OutOfBounds { cell_pos });
        }
        let CellPos(x, y) = cell_pos;
        let (x, y) = (x as u32, y as u32);

        Ok((self.width * y + x) as usize)
    }

    fn cell(&self, cell_pos: CellPos) -> Result<Cell, OutOfBounds> {
        let index = self.cell_pos_to_index(cell_pos)?;
        Ok(self.cells[index])
    }

    fn iter_cell_pos(&self) -> impl Iterator<Item = (CellPos, Cell)> + '_ {
        (0..self.width)
            .cartesian_product(0..self.height)
            .map(|(x, y)| {
                let cell_pos = CellPos(x as i32, y as i32);
                let cell = self
                    .cell(cell_pos)
                    .expect("Internal iterator operating on known size");

                (cell_pos, cell)
            })
    }

    fn set_cell(&mut self, cell_pos: CellPos, cell: Cell) -> Result<&mut Self, OutOfBounds> {
        *self.cell_mut(cell_pos)? = cell;
        Ok(self)
    }

    fn cell_mut(&mut self, cell_pos: CellPos) -> Result<&mut Cell, OutOfBounds> {
        let index = self.cell_pos_to_index(cell_pos)?;
        Ok(self.cells.get_mut(index).unwrap())
    }
}


// impl AStar<'_> {
//     fn new<'a>(grid: &'a Grid) -> AStar<'a> {
//         AStar {
//             grid,
//             open_set: HashSet::new(),
//             came_from: HashMap::new(),
//             g_score: HashMap::new(),
//             f_score: HashMap::new(),
//         }
//     }
// }

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Bundle)]
struct CellBundle {
    cell_pos: CellPos,
    name: Name,

    #[bundle]
    sprite: SpriteBundle,
}

// #[derive(Component)]
// struct AStartArc(Arc<AStar>);

fn spawn_grid(mut commands: Commands) {
    let grid = Grid::new(300, 300);

    let grid_editor = GridEditor {grid: Arc::new(grid)};

    

    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("Grid editor"))
        .insert(grid_editor);
}

fn grid_added(
    mut commands: Commands,
    new_grid: Query<(&GridEditor, Entity), Added<GridEditor>>
) {

    for (grid_editor, entity) in &new_grid {
        println!("grid editor added");
    }

    // let cell_bundles: Vec<_> = gridamazoni3
    //     .iter_cell_pos()
    //     .map(|(cell_pos, cell)| {
    //         let CellPos(x, y) = cell_pos;

    //         let color = match cell.is_wall {
    //             true => Color::BLUE,
    //             false => Color::RED,
    //         };

    //         let x_centered = x - (grid.width / 2) as i32;
    //         let y_centered = y - (grid.height / 2) as i32;
            

    //         CellBundle {
    //             cell_pos,
    //             name: Name::new(format!("({x}, {y})")),

    //             sprite: SpriteBundle {
    //                 transform: Transform::from_xyz((x_centered * 1) as f32, (y_centered * 1) as f32, 0.0),
    //                 sprite: Sprite {
    //                     color,
    //                     custom_size: Some(Vec2::new(1.0, 1.0)),
    //                     ..default()
    //                 },
    //                 ..default()
    //             },
    //         }
    //     })
    //     .collect();
}

fn update_cells(grid_query: Query<(&Grid, &Children)>, mut cells: Query<(&CellPos, &mut Sprite)>) {

    for (grid, cell_entities) in &grid_query {

        for &cell_entity in cell_entities {
            let (&cell_pos, mut sprite) = cells.get_mut(cell_entity).unwrap();
    
            let sprite = sprite.as_mut();
    
            sprite.color = match grid.cell(cell_pos).unwrap().is_wall {
                true => Color::BLUE,
                false => Color::RED,
            };
        }
    }


    // let (GridComponent(grid), children) = grid_query.single();

    // let grid = grid.lock().unwrap();

    // for &e in children {
    //     let (&cell_pos, mut sprite) = cells.get_mut(e).unwrap();

    //     let cell = grid.cell(cell_pos).unwrap();

    //     let color = match cell.is_wall {
    //         true => Color::BLUE,
    //         false => Color::RED,
    //     };

    //     sprite.color = color;
    // }
}


#[derive(Component)]
struct CellChangeEvent(CellPos);

fn randomize_cells(
    mut commands: Commands,
    mut grid: Query<(&mut GridEditor, Entity)>,
    // mut ev_cell_change: EventWriter<CellChangeEvent>,
) {

    let (mut grid_editor, entity) = grid.single_mut();

    let grid = Arc::get_mut(&mut grid_editor.grid).unwrap();

    let mut rng = rand::thread_rng();

    let width = grid.width;
    let height = grid.height;

    let x = rng.gen_range(0..width) as i32;
    let y = rng.gen_range(0..height) as i32;

    let cell_pos = CellPos(x, y);
    let is_wall = grid.cell(cell_pos).unwrap().is_wall;

    grid.cell_mut(cell_pos).unwrap().is_wall = !is_wall;

    // ev_cell_change.send(CellChangeEvent(cell_pos));

    commands.entity(entity).insert(CellChangeEvent(cell_pos));
}