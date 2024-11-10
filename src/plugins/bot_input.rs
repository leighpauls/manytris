use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::system_sets::UpdateSystems;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Clone, Resource)]
pub struct BotInputPlugin {
    pub bot_period_millis: u64,
}

impl Plugin for BotInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_bot_input)
            .add_systems(Update, apply_bot_input.in_set(UpdateSystems::Input))
            .insert_resource(self.clone());
    }
}

#[derive(Component)]
struct BotInputState {
    prev_piece_time: Option<Duration>,
}

#[derive(Bundle)]
struct BotInputBundle {
    state: BotInputState,
}

fn init_bot_input(mut cmds: Commands) {
    cmds.spawn(BotInputBundle {
        state: BotInputState {
            prev_piece_time: None,
        },
    });
}

fn apply_bot_input(
    mut input_state: Query<&mut BotInputState>,
    mut input_writer: EventWriter<InputEvent>,
    time: Res<Time<Fixed>>,
    input_config: Res<BotInputPlugin>,
) {
    let mut is = input_state.single_mut();
    let cur_time = time.elapsed();
    let Some(prev_time) = is.prev_piece_time.as_mut() else {
        is.prev_piece_time = Some(cur_time);
        return;
    };
    let target_time = *prev_time + Duration::from_millis(input_config.bot_period_millis);
    if target_time <= cur_time {
        input_writer.send(InputEvent {
            input_type: InputType::PerformBotMoveEvent,
            is_repeat: false,
        });
        *prev_time = target_time;
    }
}
