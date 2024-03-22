use crate::assets::RenderAssets;
use crate::game_state::{BlockDisplayState, GameState, Pos};
use crate::root_entity::RootMarker;
use crate::shapes::{Rot, Shift};
use crate::{assets, game_state};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

pub fn setup_field(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    q_root: Query<Entity, With<RootMarker>>,
) {
    commands.spawn(Camera2dBundle::default());

    let root = q_root.single();

    commands
        .spawn(FieldBundle::new())
        .set_parent(root)
        .with_children(|parent| {
            for y in 0..game_state::H {
                for x in 0..game_state::W {
                    parent.spawn(BlockBundle::new(Pos { x, y }, &ra));
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
    if keys.just_pressed(KeyCode::KeyZ) {
        gs.rotate(Rot::Ccw);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        gs.rotate(Rot::Cw);
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

        use BlockDisplayState::*;
        let new_material = match field.game.get_display_state(&block.pos) {
            Active(s) | Occupied(s) => ra.occupied_materials[&s].clone(),
            Shadow(s) => ra.shadow_materials[&s].clone(),
            Empty => {
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

#[derive(Bundle)]
struct FieldBundle {
    transforms: SpatialBundle,
    field: FieldComponent,
}

#[derive(Component)]
pub struct FieldComponent {
    pub game: GameState,
}

#[derive(Bundle)]
pub struct BlockBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    block: BlockComponent,
}

#[derive(Component)]
pub struct BlockComponent {
    pub pos: Pos,
}

impl FieldBundle {
    pub fn new() -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            field: FieldComponent {
                game: GameState::new(),
            },
        }
    }
}

impl BlockBundle {
    pub fn new(pos: Pos, ra: &RenderAssets) -> Self {
        Self {
            mesh: MaterialMesh2dBundle {
                mesh: ra.block_mesh.clone(),
                transform: Transform::from_xyz(
                    assets::BLOCK_SIZE * (pos.x as f32 + 0.5),
                    assets::BLOCK_SIZE * (pos.y as f32 + 0.5),
                    0.,
                ),
                material: ra.empty_material.clone(),
                ..Default::default()
            },
            block: BlockComponent { pos },
        }
    }
}
