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

#[derive(Component)]
#[require(Transform, Visibility)]
struct FieldComponent;

fn add_field_to_roots(
    mut commands: Commands,
    root_ent_q: Query<Entity, Added<GameRoot>>,
    ra: Res<RenderAssets>,
) {
    for ent in &root_ent_q {
        commands
            .spawn(FieldComponent)
            .set_parent(ent)
            .with_children(|parent| {
                for y in 0..consts::H {
                    for x in 0..consts::W {
                        parent.spawn(block_render::block_bundle(Pos { x, y }, &ra));
                    }
                }
            });
    }
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
        let mut block = {
            let _span = info_span!("find_block").entered();
            q_blocks.get_mut(block_entity.clone()).unwrap()
        };

        let _span = info_span!("update_block_color").entered();
        use manytris_core::consts;
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
