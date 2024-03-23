use crate::field::Pos;
use crate::plugins::assets;
use crate::plugins::assets::RenderAssets;
use crate::plugins::system_sets::UpdateSystems;
use crate::shapes::Shape;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, render_blocks.in_set(UpdateSystems::Render));
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
    Occupied(Shape),
    Shadow(Shape),
}

fn render_blocks(
    mut q_blocks: Query<(&mut Handle<ColorMaterial>, &BlockComponent)>,
    ra: Res<RenderAssets>,
) {
    for (mut material, block) in q_blocks.iter_mut() {
        *material = match block.color {
            BlockColor::Empty => ra.empty_material.clone(),
            BlockColor::Invisible => ra.invisible_material.clone(),
            BlockColor::Occupied(s) => ra.occupied_materials[&s].clone(),
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
                material: ra.empty_material.clone(),
                ..Default::default()
            },
            block: BlockComponent {
                pos,
                color: BlockColor::Empty,
            },
        }
    }
}
