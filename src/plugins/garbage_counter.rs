use crate::consts;
use crate::plugins::assets;
use crate::plugins::assets::RenderAssets;
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::UpdateSystems;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, render_garbage_counter.in_set(UpdateSystems::Render));
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

pub fn spawn_garbage_counters(cmd: &mut Commands, ra: &Res<RenderAssets>, root_entity: Entity) {
    for i in 0..(consts::H as usize - 2) {
        cmd.spawn(GarbageCountElementBundle {
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
