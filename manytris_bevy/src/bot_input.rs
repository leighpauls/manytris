#![cfg(feature = "bot")]

use crate::game_container::LocalGameRoot;
use crate::input::{InputEvent, InputType};
use crate::root::{GameRoot, TickEvent, TickMutationMessage};
use crate::states;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use bevy::prelude::*;
use manytris_bot::bot_start_positions::START_POSITIONS;
use manytris_bot::{bot_player, BotContext};
use manytris_core::game_state::TickMutation::JumpToBotStartPosition;
use manytris_core::game_state::{GameState, TickMutation};
use std::time::Duration;

#[derive(Clone, Resource)]
pub struct BotInputPlugin {
    pub bot_period_millis: u64,
}

impl Plugin for BotInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayingState::Playing),
            init_bot_input.run_if(states::is_bot),
        )
        .add_systems(
            OnExit(PlayingState::Playing),
            teardown_bot_input.run_if(states::is_bot),
        )
        .add_systems(
            Update,
            (
                apply_bot_input.in_set(UpdateSystems::Input),
                apply_bot_tick_events.in_set(UpdateSystems::LocalEventProducers),
            )
                .run_if(in_state(PlayingState::Playing))
                .run_if(states::is_bot),
        )
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
    cmds.spawn(BotInputState {
        prev_piece_time: None,
    });
}

fn teardown_bot_input(mut cmds: Commands, bot_input_q: Query<Entity, With<BotInputState>>) {
    cmds.entity(bot_input_q.single()).despawn();
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

fn apply_bot_tick_events(
    mut input_events: EventReader<InputEvent>,
    mut tick_event_writer: EventWriter<TickEvent>,
    q_root: Query<&GameRoot>,
    local_game_root_res: Option<Res<LocalGameRoot>>,
) {
    let Some(local_game_root) = local_game_root_res else {
        return;
    };
    let game_id = local_game_root.game_id;
    let Some(game_root) = q_root.iter().filter(|gr| gr.game_id == game_id).next() else {
        return;
    };

    let game = &game_root.active_game.game;

    input_events
        .read()
        .map(|e| match e.input_type {
            InputType::JumpToBotStartPositionEvent => {
                vec![JumpToBotStartPosition(
                    (*START_POSITIONS)
                        .bot_start_position(game.active_shape(), 0)
                        .clone(),
                )]
            }
            InputType::PerformBotMoveEvent => make_bot_move_events(game),
            _ => vec![],
        })
        .flatten()
        .for_each(|mutation| {
            tick_event_writer.send(TickEvent::new_local(TickMutationMessage {
                mutation,
                game_id,
            }));
        });
}

fn make_bot_move_events(game: &GameState) -> Vec<TickMutation> {
    let bot_context = make_context();
    let mr = bot_player::select_next_move(game, &bot_context, &bot_player::BEST_BOT_KS, 3).unwrap();
    mr.moves[0].as_tick_mutations()
}

fn make_context() -> impl BotContext {
    #[cfg(feature = "bot_vulkan")]
    {
        use manytris_bot_vulkan::VulkanBotContext;
        VulkanBotContext::init().unwrap()
    }
}
