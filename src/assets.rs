use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

pub const BLOCK_SIZE: f32 = 30.0;
pub const BLOCK_BORDER: f32 = 3.0;

#[derive(Resource)]
pub struct RenderAssets {
    pub empty_material: Handle<ColorMaterial>,
    pub occupied_material: Handle<ColorMaterial>,
    pub active_material: Handle<ColorMaterial>,
    pub invisible_material: Handle<ColorMaterial>,
    pub block_mesh: Mesh2dHandle,
}

impl FromWorld for RenderAssets {
    fn from_world(world: &mut World) -> Self {
        let rect = Rectangle::new(BLOCK_SIZE - BLOCK_BORDER, BLOCK_SIZE - BLOCK_BORDER);
        let block_mesh = Mesh2dHandle(world.resource_mut::<Assets<Mesh>>().add(rect));

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();

        Self {
            empty_material: materials.add(Color::hsl(0., 0., 0.2)),
            occupied_material: materials.add(Color::hsl(0., 0.7, 0.7)),
            active_material: materials.add(Color::hsl(180., 0.7, 0.7)),
            invisible_material: materials.add(Color::hsla(0., 0., 0., 0.)),
            block_mesh,
        }
    }
}
