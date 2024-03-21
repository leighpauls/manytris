use crate::assets;
use bevy::prelude::*;

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
            -assets::BLOCK_SIZE * 8 as f32,
            -assets::BLOCK_SIZE * 11 as f32,
            0.,
        )),
        marker: RootMarker {},
    });
}
