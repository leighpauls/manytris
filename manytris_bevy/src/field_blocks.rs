use bevy::prelude::*;

use crate::assets::RenderAssets;
use crate::block_render::{self, BlockColor, BlockComponent};
use crate::root::GameRoot;
use crate::states;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use manytris_core::consts;
use manytris_core::field::{OccupiedBlock, Pos};
use manytris_core::game_state::BlockDisplayState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_field_to_roots.in_set(UpdateSystems::PreRender),
            update_field_blocks
                .in_set(UpdateSystems::PreRender)
                .after(add_field_to_roots),
        )
            .run_if(states::headed)
            .run_if(in_state(PlayingState::Playing)),
    );
}

type BlockGrid = [[Entity; consts::W_US]; consts::H_US];

#[derive(Component)]
#[require(Transform, Visibility)]
struct FieldComponent {
    blocks: BlockGrid,
}

fn add_field_to_roots(
    mut commands: Commands,
    root_ent_q: Query<Entity, Added<GameRoot>>,
    ra: Res<RenderAssets>,
) {
    for ent in &root_ent_q {
        let blocks: BlockGrid = std::array::from_fn(|y| {
            std::array::from_fn(|x| {
                commands
                    .spawn(block_render::field_block_bundle(
                        Pos {
                            x: x as i32,
                            y: y as i32,
                        },
                        &ra,
                    ))
                    .id()
            })
        });
        let children: Vec<Entity> = blocks
            .iter()
            .flat_map(|row| row.clone().into_iter())
            .collect();
        commands
            .spawn(FieldComponent { blocks })
            .set_parent(ent)
            .add_children(&children);
    }
}

fn update_field_blocks(
    q_root: Query<(&GameRoot, &Children)>,
    q_field: Query<&FieldComponent>,
    mut q_blocks: Query<&mut BlockComponent>,
) {
    for (game_root, root_children) in q_root.iter() {
        for field_component in q_field.iter_many(root_children) {
            for (y, row) in field_component.blocks.iter().enumerate() {
                for (x, block_entity) in row.iter().enumerate() {
                    let mut block = q_blocks
                        .get_mut(*block_entity)
                        .expect("Missing block from field component");

                    use manytris_core::consts;
                    use BlockDisplayState::*;

                    let pos = Pos {
                        x: x as i32,
                        y: y as i32,
                    };
                    block.color = match game_root.active_game.game.get_display_state(&pos) {
                        Occupied(ob) => BlockColor::Occupied(ob),
                        Active(s) => BlockColor::Occupied(OccupiedBlock::FromShape(s)),
                        Shadow(s) => BlockColor::Shadow(s),
                        Empty => {
                            if pos.y < consts::H - consts::PREVIEW_H {
                                BlockColor::Empty
                            } else {
                                BlockColor::Invisible
                            }
                        }
                    };
                }
            }
        }
    }
}
