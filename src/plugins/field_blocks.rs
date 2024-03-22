use crate::game_state;
use crate::game_state::{BlockDisplayState, Pos};
use crate::plugins::assets::RenderAssets;
use crate::plugins::block_render::{BlockBundle, BlockColor, BlockComponent};
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_field.in_set(StartupSystems::AfterRoot))
        .add_systems(Update, update_field_blocks.in_set(UpdateSystems::PreRender));
}

#[derive(Bundle)]
struct FieldBundle {
    transforms: SpatialBundle,
    field: FieldComponent,
}

#[derive(Component)]
struct FieldComponent;

impl FieldBundle {
    pub fn new() -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            field: FieldComponent,
        }
    }
}

fn setup_field(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    q_root: Query<Entity, With<GameRoot>>,
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

fn update_field_blocks(
    q_root: Query<&GameRoot>,
    q_field_children: Query<&Children, With<FieldComponent>>,
    mut q_blocks: Query<&mut BlockComponent>,
) {
    let game = &q_root.single().game;
    let children = q_field_children.single();

    for child_id in children {
        let mut block = q_blocks.get_mut(child_id.clone()).unwrap();

        use BlockDisplayState::*;
        block.color = match game.get_display_state(&block.pos) {
            Active(s) | Occupied(s) => BlockColor::Occupied(s),
            Shadow(s) => BlockColor::Shadow(s),
            Empty => {
                if block.pos.y < game_state::H - game_state::PREVIEW_H {
                    BlockColor::Empty
                } else {
                    BlockColor::Invisible
                }
            }
        };
    }
}
