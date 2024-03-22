use crate::assets::RenderAssets;
use crate::game_state::{BlockDisplayState, GameState, Pos};
use crate::input::InputEvent;
use crate::root_entity::RootMarker;
use crate::system_sets::{StartupSystems, UpdateSystems};
use crate::{assets, game_state, input, root_entity};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

pub fn entities_plugin(app: &mut App) {
    app.add_systems(Startup, setup_field.in_set(StartupSystems::AfterRoot))
        .add_systems(
            Update,
            (
                update_field_tick.in_set(UpdateSystems::RootTick),
                update_block_colors.in_set(UpdateSystems::Render),
            ),
        );
}

fn setup_field(
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

fn update_field_tick(
    mut q_field: Query<&mut FieldComponent>,
    mut input_events: EventReader<InputEvent>,
) {
    let gs = &mut q_field.single_mut().game;

    for event in input_events.read() {
        use InputEvent::*;
        match event {
            ShiftEvent(s) => {
                gs.shift(*s);
            }
            RotateEvent(d) => {
                gs.rotate(*d);
            }
            DownEvent => {
                gs.down();
            }
            DropEvent => {
                gs.drop();
            }
            HoldEvent => {
                gs.hold();
            }
        }
    }
}

fn update_block_colors(
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
