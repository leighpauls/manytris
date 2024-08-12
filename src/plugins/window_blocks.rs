use crate::consts;
use crate::field::Pos;
use crate::plugins::assets;
use crate::plugins::assets::RenderAssets;
use crate::plugins::block_render::{BlockBundle, BlockColor, BlockComponent};
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use crate::tetromino::Tetromino;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_windows.in_set(StartupSystems::AfterRoot))
        .add_systems(
            Update,
            (update_preview_window_blocks, update_hold_window_blocks)
                .in_set(UpdateSystems::PreRender),
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

fn setup_windows(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    q_root: Query<Entity, With<GameRoot>>,
) {
    let root = q_root.single();
    let spawn_blocks_fn = |parent: &mut ChildBuilder| {
        spawn_window_block_children(parent, &ra);
    };

    for i in 0..consts::NUM_PREVIEWS {
        commands
            .spawn(PreviewWindowBundle::new(i))
            .set_parent(root)
            .with_children(spawn_blocks_fn);
    }

    commands
        .spawn(HoldWindowBundle::new())
        .set_parent(root)
        .with_children(spawn_blocks_fn);
}

type BlockQuery<'world, 'state, 'a> = Query<'world, 'state, &'a mut BlockComponent>;

fn update_preview_window_blocks(
    q_root: Query<&GameRoot>,
    q_windows: Query<(&PreviewWindowComponent, &Children)>,
    mut q_blocks: BlockQuery,
) {
    const ARRAY_REPEAT_VALUE: std::option::Option<Tetromino> = None;
    let previews = if let Some(active_game) = &q_root.single().active_game {
        active_game.game.previews().map(Some)
    } else {
        [ARRAY_REPEAT_VALUE; 6]
    };

    for (window, children) in &q_windows {
        let preview = previews[window.preview_idx].clone();
        update_child_block_colors(preview.as_ref(), children, &mut q_blocks);
    }
}

fn update_hold_window_blocks(
    q_root: Query<&GameRoot>,
    q_window: Query<&Children, With<HoldWindowComponent>>,
    mut q_blocks: BlockQuery,
) {
    let held = if let Some(active_game) = &q_root.single().active_game {
        active_game.game.held_tetromino()
    } else {
        None
    };

    update_child_block_colors(held.as_ref(), q_window.single(), &mut q_blocks);
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
                Some(t) if t.contains(&block.pos) => BlockColor::Occupied(t.shape),
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
