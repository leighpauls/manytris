use crate::shapes::Shape;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use enum_iterator::all;
use std::collections::HashMap;

pub const BLOCK_SIZE: f32 = 30.0;
pub const BLOCK_BORDER: f32 = 3.0;

pub fn assets_plugin(app: &mut App) {
    app.init_resource::<RenderAssets>();
}

#[derive(Resource)]
pub struct RenderAssets {
    pub occupied_materials: HashMap<Shape, Handle<ColorMaterial>>,
    pub shadow_materials: HashMap<Shape, Handle<ColorMaterial>>,
    pub empty_material: Handle<ColorMaterial>,
    pub invisible_material: Handle<ColorMaterial>,
    pub block_mesh: Mesh2dHandle,
}

impl FromWorld for RenderAssets {
    fn from_world(world: &mut World) -> Self {
        let rect = Rectangle::new(BLOCK_SIZE - BLOCK_BORDER, BLOCK_SIZE - BLOCK_BORDER);
        let block_mesh = Mesh2dHandle(world.resource_mut::<Assets<Mesh>>().add(rect));

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();

        let all_shapes: Vec<Shape> = all::<Shape>().collect();
        let num_shapes = all_shapes.len() as f32;

        let hues_iter = all_shapes
            .iter()
            .enumerate()
            .map(|(idx, shape)| (*shape, 360. * idx as f32 / num_shapes));

        let occupied_materials = hues_iter
            .clone()
            .map(|(shape, hue)| (shape, materials.add(Color::hsl(hue, 0.7, 0.7))))
            .collect::<HashMap<Shape, Handle<ColorMaterial>>>();
        let shadow_materials = hues_iter
            .map(|(shape, hue)| (shape, materials.add(Color::hsl(hue, 0.15, 0.7))))
            .collect::<HashMap<Shape, Handle<ColorMaterial>>>();

        Self {
            occupied_materials,
            shadow_materials,
            empty_material: materials.add(Color::hsl(0., 0., 0.2)),
            invisible_material: materials.add(Color::hsla(0., 0., 0., 0.)),
            block_mesh,
        }
    }
}
