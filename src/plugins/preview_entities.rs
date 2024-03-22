use crate::game_state::{Pos, Tetromino};
use crate::plugins::assets;
use crate::plugins::assets::RenderAssets;
use crate::plugins::entities::{BlockBundle, BlockComponent, FieldComponent};
use crate::plugins::root_entity::RootMarker;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use bevy::prelude::*;
use crate::{game_state, upcoming};

pub fn preview_plugin(app: &mut App) {
    app.add_systems(Startup, setup_windows.in_set(StartupSystems::AfterRoot))
        .add_systems(
            Update,
            (update_preview_window, update_hold_window).in_set(UpdateSystems::Render),
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
    q_root: Query<Entity, With<RootMarker>>,
) {
    let root = q_root.single();
    let spawn_blocks_fn = |parent: &mut ChildBuilder| {
        spawn_window_block_children(parent, &ra);
    };

    for i in 0..upcoming::NUM_PREVIEWS {
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

type BlockQuery<'world, 'state, 'a> =
    Query<'world, 'state, (&'a mut Handle<ColorMaterial>, &'a BlockComponent)>;

fn update_preview_window(
    q_field: Query<&FieldComponent>,
    q_windows: Query<(&PreviewWindowComponent, &Children)>,
    mut q_blocks: BlockQuery,
    ra: Res<RenderAssets>,
) {
    let field = q_field.single();
    let previews = field.game.previews();

    for (window, children) in &q_windows {
        let preview = &previews[window.preview_idx];
        update_child_block_colors(Some(preview), children, &mut q_blocks, &ra);
    }
}

fn update_hold_window(
    q_field: Query<&FieldComponent>,
    q_window: Query<&Children, With<HoldWindowComponent>>,
    mut q_blocks: BlockQuery,
    ra: Res<RenderAssets>,
) {
    let held = q_field.single().game.held_tetromino();
    update_child_block_colors(held.as_ref(), q_window.single(), &mut q_blocks, &ra);
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
    ra: &RenderAssets,
) {
    for child in children {
        if let Ok((mut material, block)) = q_blocks.get_mut(*child) {
            *material = match preview {
                Some(t) if t.contains(&block.pos) => ra.occupied_materials[&t.shape].clone(),
                _ => ra.invisible_material.clone(),
            };
        }
    }
}

impl PreviewWindowBundle {
    fn new(preview_idx: usize) -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                assets::BLOCK_SIZE * (game_state::W + 1) as f32,
                assets::BLOCK_SIZE * (game_state::H - 3 - 4 * preview_idx as i32) as f32,
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
                assets::BLOCK_SIZE * (game_state::H - 3) as f32,
                0.,
            )),
            hold: HoldWindowComponent(),
        }
    }
}
