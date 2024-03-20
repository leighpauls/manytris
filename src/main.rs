use crate::game_state::{GameState, Shift, Tetromino};
use bevy::prelude::*;

mod game_state;
mod shapes;
mod field_rendering;

fn main() {
    println!("Hello, world!");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, field_rendering::setup)
        .add_systems(Update, field_rendering::update_field)
        .run();
}
