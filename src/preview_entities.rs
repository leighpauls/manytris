use crate::assets::RenderAssets;
use crate::entities::{BlockBundle, BlockComponent, FieldComponent};
use crate::game_state::{Pos, Tetromino};
use crate::root_entity::RootMarker;
use crate::{assets, game_state, upcoming};
use bevy::prelude::*;

#[derive(Bundle)]
pub struct PreviewWindowBundle {
    transforms: SpatialBundle,
    preview: PreviewWindowComponent,
}

#[derive(Component)]
pub struct PreviewWindowComponent {
    preview_idx: usize,
}

pub fn setup_previews(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    q_root: Query<Entity, With<RootMarker>>,
) {
    let root = q_root.single();
    for i in 0..upcoming::NUM_PREVIEWS {
        commands
            .spawn(PreviewWindowBundle::new(i))
            .set_parent(root)
            .with_children(|parent| {
                for y in 0..3 {
                    for x in 0..4 {
                        parent.spawn(BlockBundle::new(Pos { x, y }, &ra));
                    }
                }
            });
    }
}

pub fn update_preview_window(
    q_field: Query<&FieldComponent>,
    q_windows: Query<(&PreviewWindowComponent, &Children)>,
    mut q_blocks: Query<(&mut Handle<ColorMaterial>, &BlockComponent)>,
    ra: Res<RenderAssets>,
) {
    let field = q_field.single();
    let previews = field.game.previews();

    for (window, children) in &q_windows {
        let preview = &previews[window.preview_idx];
        for child in children {
            if let Ok((mut material, block)) = q_blocks.get_mut(*child) {
                *material = if preview.contains(&block.pos) {
                    ra.occupied_material.clone()
                } else {
                    ra.invisible_material.clone()
                };
            }
        }
    }
}

impl PreviewWindowBundle {
    fn new(preview_idx: usize) -> Self {
        Self {
            transforms: SpatialBundle::from_transform(Transform::from_xyz(
                assets::BLOCK_SIZE * (game_state::W + 1) as f32,
                assets::BLOCK_SIZE
                    * (game_state::H - game_state::PREVIEW_H - 4 * preview_idx as i32) as f32,
                0.,
            )),
            preview: PreviewWindowComponent { preview_idx },
        }
    }
}
