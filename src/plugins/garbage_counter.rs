use manytris_core::consts;
use crate::plugins::assets::RenderAssets;
use crate::plugins::root::GameRoot;
use crate::plugins::states::PlayingState;
use crate::plugins::system_sets::UpdateSystems;
use crate::plugins::{assets, states};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

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
pub struct GarbageCountElementComponent {
    index: usize,
}

#[derive(Bundle)]
pub struct GarbageCountElementBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    element: GarbageCountElementComponent,
}

fn add_garbage_counters_to_root(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    root_ent_q: Query<Entity, Added<GameRoot>>,
) {
    for root_entity in &root_ent_q {
        for i in 0..(consts::H as usize - 2) {
            commands
                .spawn(GarbageCountElementBundle {
                    element: GarbageCountElementComponent { index: i },
                    mesh: MaterialMesh2dBundle {
                        mesh: ra.block_mesh.clone(),
                        transform: Transform::from_xyz(
                            -assets::BLOCK_SIZE,
                            assets::BLOCK_SIZE * (i as f32 + 0.5),
                            0.,
                        ),
                        material: ra.empty_material.clone(),
                        ..default()
                    },
                })
                .set_parent(root_entity);
        }
    }
}

fn render_garbage_counter(
    q_root: Query<&GameRoot>,
    mut q_garbage_elements: Query<(
        &mut Handle<ColorMaterial>,
        &GarbageCountElementComponent,
        &Parent,
    )>,
    ra: Res<RenderAssets>,
) {
    for (mut material, ge, parent) in &mut q_garbage_elements.iter_mut() {
        let gr = q_root.get(parent.get()).unwrap();
        let count_value = gr.active_game.game.get_garbage_element_countdown(ge.index);

        *material = if let Some(count) = count_value {
            ra.garbage_counter_materials[count - 1].clone()
        } else {
            ra.empty_material.clone()
        };
    }
}
