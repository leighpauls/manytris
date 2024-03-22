use crate::assets;
use bevy::prelude::*;

pub fn root_plugin(app: &mut App) {
    app.add_systems(Startup, setup_root);
}

#[derive(Component)]
pub struct RootMarker {}

#[derive(Bundle)]
pub struct RootTransformBundle {
    transform: SpatialBundle,
    marker: RootMarker,
}

pub fn setup_root(mut commands: Commands) {
    commands.spawn(RootTransformBundle {
        transform: SpatialBundle::from_transform(Transform::from_xyz(
            -assets::BLOCK_SIZE * 8.,
            -assets::BLOCK_SIZE * 11.,
            0.,
        )),
        marker: RootMarker {},
    });
}
