use bevy::prelude::*;

use crate::consts;
use crate::field::{OccupiedBlock, Pos};
use crate::game_state::BlockDisplayState;
use crate::plugins::assets::RenderAssets;
use crate::plugins::block_render::{BlockBundle, BlockColor, BlockComponent};
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::UpdateSystems;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_field_blocks.in_set(UpdateSystems::PreRender));
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

pub fn spawn_field(commands: &mut Commands, ra: &Res<RenderAssets>, root_entity: Entity) {
    commands
        .spawn(FieldBundle::new())
        .set_parent(root_entity)
        .with_children(|parent| {
            for y in 0..consts::H {
                for x in 0..consts::W {
                    parent.spawn(BlockBundle::new(Pos { x, y }, ra));
                }
            }
        });
}

fn update_field_blocks(
    q_root: Query<(&GameRoot, &Children)>,
    q_field_children: Query<&Children, With<FieldComponent>>,
    mut q_blocks: Query<&mut BlockComponent>,
) {
    // Collect the game_root and associated block entities
    let iter = q_root
        .iter()
        .map(|(game_root, root_children)| {
            q_field_children
                .iter_many(root_children)
                .flatten()
                .map(move |block_entity| (game_root, block_entity))
        })
        .flatten();

    for (game_root, block_entity) in iter {
        let mut block = q_blocks.get_mut(block_entity.clone()).unwrap();

        use BlockDisplayState::*;
        block.color = match game_root.active_game.game.get_display_state(&block.pos) {
            Occupied(ob) => BlockColor::Occupied(ob),
            Active(s) => BlockColor::Occupied(OccupiedBlock::FromShape(s)),
            Shadow(s) => BlockColor::Shadow(s),
            Empty => {
                if block.pos.y < consts::H - consts::PREVIEW_H {
                    BlockColor::Empty
                } else {
                    BlockColor::Invisible
                }
            }
        };
    }
}
