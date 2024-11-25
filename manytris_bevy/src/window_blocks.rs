use crate::assets::RenderAssets;
use crate::block_render::{BlockBundle, BlockColor, BlockComponent};
use crate::root::GameRoot;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use crate::{assets, states};
use bevy::prelude::*;
use manytris_core::consts;
use manytris_core::field::{OccupiedBlock, Pos};
use manytris_core::tetromino::Tetromino;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_windows_to_roots,
            update_preview_window_blocks.after(add_windows_to_roots),
            update_hold_window_blocks.after(add_windows_to_roots),
        )
            .in_set(UpdateSystems::PreRender)
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::headed),
    );
}

#[derive(Bundle)]
struct PreviewWindowBundle {
    transforms: SpatialBundle,
    preview: PreviewWindowComponent,
}

#[derive(Component)]
struct PreviewWindowComponent {
    preview_idx: usize,
}

#[derive(Bundle)]
struct HoldWindowBundle {
    transforms: SpatialBundle,
    hold: HoldWindowComponent,
}

#[derive(Component)]
struct HoldWindowComponent();

fn add_windows_to_roots(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    root_ent_q: Query<Entity, Added<GameRoot>>,
) {
    for root_entity in &root_ent_q {
        let spawn_blocks_fn = |parent: &mut ChildBuilder| {
            spawn_window_block_children(parent, &ra);
        };

        for i in 0..consts::NUM_PREVIEWS {
            commands
                .spawn(PreviewWindowBundle::new(i))
                .set_parent(root_entity)
                .with_children(spawn_blocks_fn);
        }

        commands
            .spawn(HoldWindowBundle::new())
            .set_parent(root_entity)
            .with_children(spawn_blocks_fn);
    }
}

type BlockQuery<'world, 'state, 'a> = Query<'world, 'state, &'a mut BlockComponent>;

fn update_preview_window_blocks(
    q_root: Query<(&GameRoot, &Children)>,
    q_windows: Query<(&PreviewWindowComponent, &Children)>,
    mut q_blocks: BlockQuery,
) {
    for (game_root, root_children) in q_root.iter() {
        let previews = game_root.active_game.game.previews();
        for (window, window_children) in q_windows.iter_many(root_children) {
            update_child_block_colors(
                Some(&previews[window.preview_idx]),
                window_children,
                &mut q_blocks,
            );
        }
    }
}

fn update_hold_window_blocks(
    q_root: Query<&GameRoot>,
    q_window: Query<(&Children, &Parent), With<HoldWindowComponent>>,
    mut q_blocks: BlockQuery,
) {
    for (window_children, window_parent) in q_window.iter() {
        let game_root = q_root.get(window_parent.get()).unwrap();
        let held = game_root.active_game.game.held_tetromino();

        update_child_block_colors(held.as_ref(), window_children, &mut q_blocks);
    }
}

fn spawn_window_block_children(parent: &mut ChildBuilder, ra: &RenderAssets) {
    for y in 0..3 {
        for x in 0..4 {
            parent.spawn(BlockBundle::new(Pos { x, y }, &ra));
        }
    }
}

fn update_child_block_colors(
    preview: Option<&Tetromino>,
    children: &Children,
    q_blocks: &mut BlockQuery,
) {
    for child in children {
        if let Ok(mut block) = q_blocks.get_mut(*child) {
            block.color = match preview {
                Some(t) if t.contains(&block.pos) => {
                    BlockColor::Occupied(OccupiedBlock::FromShape(t.shape))
                }
                _ => BlockColor::Invisible,
            };
        }
    }
}

impl PreviewWindowBundle {
    fn new(preview_idx: usize) -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                assets::BLOCK_SIZE * (consts::W + 1) as f32,
                assets::BLOCK_SIZE * (consts::H - 3 - 4 * preview_idx as i32) as f32,
                0.,
            )),
            preview: PreviewWindowComponent { preview_idx },
        }
    }
}

impl HoldWindowBundle {
    fn new() -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                -assets::BLOCK_SIZE * 5.,
                assets::BLOCK_SIZE * (consts::H - 3) as f32,
                0.,
            )),
            hold: HoldWindowComponent(),
        }
    }
}
