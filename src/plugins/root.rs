use crate::game_state::GameState;
use crate::plugins::assets;
use crate::plugins::input::InputEvent;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_root.in_set(StartupSystems::Root))
        .add_systems(Update, update_root_tick.in_set(UpdateSystems::RootTick));
}

#[derive(Component)]
pub struct GameRoot {
    pub game: GameState,
}

#[derive(Bundle)]
pub struct RootTransformBundle {
    transform: SpatialBundle,
    marker: GameRoot,
}

fn setup_root(mut commands: Commands) {
    commands.spawn(RootTransformBundle {
        transform: SpatialBundle::from_transform(Transform::from_xyz(
            -assets::BLOCK_SIZE * 8.,
            -assets::BLOCK_SIZE * 11.,
            0.,
        )),
        marker: GameRoot {
            game: GameState::new(),
        },
    });
}

fn update_root_tick(mut q_root: Query<&mut GameRoot>, mut input_events: EventReader<InputEvent>) {
    let gs = &mut q_root.single_mut().game;

    for event in input_events.read() {
        use InputEvent::*;
        match event {
            ShiftEvent(s) => {
                gs.shift(*s);
            }
            RotateEvent(d) => {
                gs.rotate(*d);
            }
            DownEvent => {
                gs.down();
            }
            DropEvent => {
                gs.drop();
            }
            HoldEvent => {
                gs.hold();
            }
        }
    }
}
