use crate::assets::RenderAssets;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use crate::{assets, states};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use manytris_core::field::{OccupiedBlock, Pos};
use manytris_core::shapes::Shape;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        render_blocks
            .in_set(UpdateSystems::Render)
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::headed),
    );
}

#[derive(Bundle)]
pub struct BlockBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    block: BlockComponent,
}

#[derive(Component)]
pub struct BlockComponent {
    pub pos: Pos,
    pub color: BlockColor,
}

pub enum BlockColor {
    Empty,
    Invisible,
    Occupied(OccupiedBlock),
    Shadow(Shape),
}

fn render_blocks(
    mut q_blocks: Query<(&mut MeshMaterial2d<ColorMaterial>, &BlockComponent)>,
    ra: Res<RenderAssets>,
) {
    for (mut material, block) in q_blocks.iter_mut() {
        material.0 = match block.color {
            BlockColor::Empty => ra.empty_material.clone(),
            BlockColor::Invisible => ra.invisible_material.clone(),
            BlockColor::Occupied(ob) => ra.occupied_materials[&ob].clone(),
            BlockColor::Shadow(s) => ra.shadow_materials[&s].clone(),
        };
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
                material: MeshMaterial2d(ra.empty_material.clone()),
                ..Default::default()
            },
            block: BlockComponent {
                pos,
                color: BlockColor::Empty,
            },
        }
    }
}
