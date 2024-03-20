use crate::game_state;
use crate::game_state::{BlockState, GameState, Pos, Shift};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

const BLOCK_SIZE: f32 = 30.0;
const BLOCK_BORDER: f32 = 3.0;

pub fn setup_assets(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(RenderAssets {
        empty_material: materials.add(Color::hsl(0., 0., 0.2)),
        occupied_material: materials.add(Color::hsl(0., 0.7, 0.7)),
        active_material: materials.add(Color::hsl(180., 0.7, 0.7)),
        invisible_material: materials.add(Color::hsla(0., 0., 0., 0.)),
    });
}

pub fn setup_field(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(Camera2dBundle::default());

    let rect = Rectangle::new(BLOCK_SIZE - BLOCK_BORDER, BLOCK_SIZE - BLOCK_BORDER);
    let block_mesh = Mesh2dHandle(meshes.add(rect));

    commands.spawn(FieldBundle::new()).with_children(|parent| {
        for y in 0..game_state::H {
            for x in 0..game_state::W {
                parent.spawn(BlockBundle::new(Pos { x, y }, block_mesh.clone()));
            }
        }
    });
}

pub fn update_for_input(mut q_field: Query<&mut FieldComponent>, keys: Res<ButtonInput<KeyCode>>) {
    let gs = &mut q_field.single_mut().game;
    if keys.just_pressed(KeyCode::ArrowLeft) {
        gs.shift(Shift::Left);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        gs.shift(Shift::Right);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        gs.down();
    }
    if keys.just_pressed(KeyCode::Space) {
        gs.drop();
    }
}

pub fn update_block_colors(
    q_field: Query<(&FieldComponent, &Children)>,
    mut q_blocks: Query<(&mut Handle<ColorMaterial>, &BlockComponent)>,
    ra: Res<RenderAssets>,
) {
    let (field, children) = q_field.single();

    for child_id in children {
        let (mut material, block) = q_blocks.get_mut(child_id.clone()).unwrap();

        let new_material = match field.game.check_block(&block.pos) {
            BlockState::Active => ra.active_material.clone(),
            BlockState::Occupied => ra.occupied_material.clone(),
            BlockState::Empty => {
                if block.pos.y < game_state::H - game_state::PREVIEW_H {
                    ra.empty_material.clone()
                } else {
                    ra.invisible_material.clone()
                }
            }
        };

        *material = new_material;
    }
}

#[derive(Resource)]
pub struct RenderAssets {
    empty_material: Handle<ColorMaterial>,
    occupied_material: Handle<ColorMaterial>,
    active_material: Handle<ColorMaterial>,
    invisible_material: Handle<ColorMaterial>,
}

#[derive(Bundle)]
struct FieldBundle {
    transforms: SpatialBundle,
    field: FieldComponent,
}

#[derive(Component)]
pub struct FieldComponent {
    game: GameState,
}

#[derive(Bundle)]
struct BlockBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    block: BlockComponent,
}

#[derive(Component)]
pub struct BlockComponent {
    pos: Pos,
}

impl FieldBundle {
    pub fn new() -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                -game_state::W as f32 * BLOCK_SIZE / 2.,
                -game_state::H as f32 * BLOCK_SIZE / 2.,
                0.,
            )),
            field: FieldComponent {
                game: GameState::new(),
            },
        }
    }
}

impl BlockBundle {
    pub fn new(pos: Pos, block_mesh: Mesh2dHandle) -> Self {
        Self {
            mesh: MaterialMesh2dBundle {
                mesh: block_mesh,
                transform: Transform::from_xyz(
                    BLOCK_SIZE * pos.x as f32,
                    BLOCK_SIZE * pos.y as f32,
                    0.,
                ),
                ..Default::default()
            },
            block: BlockComponent { pos },
        }
    }
}
