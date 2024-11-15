use bevy::prelude::*;

pub struct StatesPlugin {
    pub initial_state: PlayingState,
}

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(self.initial_state);
    }
}

#[derive(States, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PlayingState {
    MainMenu,
    Playing,
}
