use crate::assets::RenderAssets;
use crate::root::GameRoot;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use crate::{assets, states};
use bevy::prelude::*;
use manytris_core::consts;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_garbage_counters_to_root.in_set(UpdateSystems::PreRender),
            render_garbage_counter.in_set(UpdateSystems::Render),
        )
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::headed),
    );
}

#[derive(Component)]
#[require(Mesh2d, MeshMaterial2d::<ColorMaterial>)]
pub struct GarbageCountElementComponent {
    index: usize,
}

fn add_garbage_counters_to_root(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    root_ent_q: Query<Entity, Added<GameRoot>>,
) {
    for root_entity in &root_ent_q {
        for i in 0..(consts::H as usize - 2) {
            commands
                .spawn((
                    GarbageCountElementComponent { index: i },
                    ra.block_mesh.clone(),
                    Transform::from_xyz(
                        -assets::BLOCK_SIZE,
                        assets::BLOCK_SIZE * (i as f32 + 0.5),
                        0.,
                    ),
                    MeshMaterial2d(ra.empty_material.clone()),
                ))
                .set_parent(root_entity);
        }
    }
}

fn render_garbage_counter(
    q_root: Query<&GameRoot>,
    mut q_garbage_elements: Query<(
        &mut MeshMaterial2d<ColorMaterial>,
        &GarbageCountElementComponent,
        &Parent,
    )>,
    ra: Res<RenderAssets>,
) {
    for (mut material, ge, parent) in &mut q_garbage_elements.iter_mut() {
        let gr = q_root.get(parent.get()).unwrap();
        let count_value = gr.active_game.game.get_garbage_element_countdown(ge.index);

        material.0 = if let Some(count) = count_value {
            ra.garbage_counter_materials[count - 1].clone()
        } else {
            ra.empty_material.clone()
        };
    }
}
