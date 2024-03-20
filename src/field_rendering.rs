use crate::game_state;
use crate::game_state::{BlockState, GameState, Pos, Shift, Tetromino};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

const BLOCK_SIZE: f32 = 30.0;
const BLOCK_BORDER: f32 = 3.0;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let rect = Rectangle::new(BLOCK_SIZE - BLOCK_BORDER, BLOCK_SIZE - BLOCK_BORDER);
    let block_mesh = Mesh2dHandle(meshes.add(rect));
    let empty_material = materials.add(Color::hsl(0., 0., 0.2));
    let occupied_material = materials.add(Color::hsl(0., 0.7, 0.7));
    let active_material = materials.add(Color::hsl(180., 0.7, 0.7));

    commands
        .spawn(RenderingFieldBundle::new(
            init_game(),
            empty_material.clone(),
            occupied_material,
            active_material,
        ))
        .with_children(|parent| {
            for y in 0..game_state::H {
                for x in 0..game_state::W {
                    parent.spawn(RenderingBlockBundle::new(
                        Pos { x, y },
                        block_mesh.clone(),
                        empty_material.clone(),
                    ));
                }
            }
        });
}

pub fn update_field(
    q_field: Query<(&RenderingField, &Children)>,
    mut q_blocks: Query<(&mut Handle<ColorMaterial>, &RenderingBlock)>,
) {
    let (field, children) = q_field.single();

    for child_id in children {
        let (mut material, block) = q_blocks.get_mut(child_id.clone()).unwrap();

        let new_material = match field.game.check_block(&block.pos) {
            BlockState::Active => field.active_material.clone(),
            BlockState::Occupied => field.occupied_material.clone(),
            BlockState::Empty => field.empty_material.clone(),
        };

        *material = new_material;
    }
}

fn init_game() -> GameState {
    let mut gs = GameState::new();
    gs.print();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.shift(Shift::Right);
    gs.print();
    gs.drop();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.cw();
    gs.print();

    gs
}

#[derive(Bundle)]
struct RenderingFieldBundle {
    transforms: SpatialBundle,
    field: RenderingField,
}

#[derive(Component)]
pub struct RenderingField {
    game: GameState,
    empty_material: Handle<ColorMaterial>,
    occupied_material: Handle<ColorMaterial>,
    active_material: Handle<ColorMaterial>,
}

#[derive(Bundle)]
struct RenderingBlockBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    block: RenderingBlock,
}

#[derive(Component)]
pub struct RenderingBlock {
    pos: Pos,
}

impl RenderingFieldBundle {
    pub fn new(
        game: GameState,
        empty_material: Handle<ColorMaterial>,
        occupied_material: Handle<ColorMaterial>,
        active_material: Handle<ColorMaterial>,
    ) -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                -game_state::W as f32 * BLOCK_SIZE / 2.,
                -game_state::H as f32 * BLOCK_SIZE / 2.,
                0.,
            )),
            field: RenderingField {
                game,
                empty_material,
                occupied_material,
                active_material,
            },
        }
    }
}

impl RenderingBlockBundle {
    pub fn new(pos: Pos, block_mesh: Mesh2dHandle, empty_material: Handle<ColorMaterial>) -> Self {
        Self {
            mesh: MaterialMesh2dBundle {
                mesh: block_mesh,
                material: empty_material,
                transform: Transform::from_xyz(
                    BLOCK_SIZE * pos.x as f32,
                    BLOCK_SIZE * pos.y as f32,
                    0.,
                ),
                ..Default::default()
            },
            block: RenderingBlock { pos },
        }
    }
}
