use crate::consts;
use crate::field::OccupiedBlock;
use crate::shapes::Shape;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use std::collections::HashMap;

pub const BLOCK_SIZE: f32 = 30.0;
pub const BLOCK_BORDER: f32 = 3.0;

pub fn plugin(app: &mut App) {
    app.init_resource::<RenderAssets>();
}

#[derive(Resource)]
pub struct RenderAssets {
    pub occupied_materials: HashMap<OccupiedBlock, Handle<ColorMaterial>>,
    pub shadow_materials: HashMap<Shape, Handle<ColorMaterial>>,
    pub empty_material: Handle<ColorMaterial>,
    pub invisible_material: Handle<ColorMaterial>,
    pub block_mesh: Mesh2dHandle,
    pub garbage_counter_materials: [Handle<ColorMaterial>; consts::GARBAGE_TURN_COUNT],
}

impl FromWorld for RenderAssets {
    fn from_world(world: &mut World) -> Self {
        let rect = Rectangle::new(BLOCK_SIZE - BLOCK_BORDER, BLOCK_SIZE - BLOCK_BORDER);
        let block_mesh = Mesh2dHandle(world.resource_mut::<Assets<Mesh>>().add(rect));

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();

        let hue_pairs = [
            (Shape::Z, 0.),
            (Shape::L, 30.),
            (Shape::O, 60.),
            (Shape::S, 120.),
            (Shape::I, 180.),
            (Shape::J, 240.),
            (Shape::T, 300.),
        ];

        let mut occupied_materials = hue_pairs
            .iter()
            .map(|(shape, hue)| {
                (
                    OccupiedBlock::FromShape(*shape),
                    materials.add(Color::hsl(*hue, 0.7, 0.7)),
                )
            })
            .collect::<HashMap<OccupiedBlock, Handle<ColorMaterial>>>();
        occupied_materials.insert(
            OccupiedBlock::FromGarbage,
            materials.add(Color::hsl(0., 0., 0.7)),
        );

        let shadow_materials = hue_pairs
            .iter()
            .map(|(shape, hue)| (*shape, materials.add(Color::hsl(*hue, 0.15, 0.7))))
            .collect::<HashMap<Shape, Handle<ColorMaterial>>>();

        let garbage_hue_sat = [(0., 1.), (60., 1.), (0., 0.), (0., 0.)];
        let garbage_counter_materials =
            garbage_hue_sat.map(|(hue, sat)| materials.add(Color::hsl(hue, sat, 0.5)));

        Self {
            occupied_materials,
            shadow_materials,
            empty_material: materials.add(Color::hsl(0., 0., 0.2)),
            invisible_material: materials.add(Color::hsla(0., 0., 0., 0.)),
            block_mesh,
            garbage_counter_materials,
        }
    }
}
