use crate::plugins::system_sets::UpdateSystems;
use crate::shapes::{Rot, Shift};
use bevy::prelude::*;
use bevy::utils::Duration;

const INITIAL_REPEAT: Duration = Duration::from_millis(160);
const REPEAT: Duration = Duration::from_millis(30);

pub fn plugin(app: &mut App) {
    app.add_event::<InputEvent>()
        .init_resource::<RepeatTimes>()
        .add_systems(Update, update_for_input.in_set(UpdateSystems::Input));
}

#[derive(Event, Copy, Clone)]
pub struct InputEvent {
    pub input_type: InputType,
    pub is_repeat: bool,
}

#[derive(Copy, Clone)]
pub enum InputType {
    ShiftEvent(Shift),
    RotateEvent(Rot),
    DownEvent,
    DropEvent,
    HoldEvent,
    JumpToBotStartPositionEvent,
    PerformBotMoveEvent,
    EnqueueGarbageEvent(usize),
}

#[derive(Resource)]
pub struct RepeatTimes {
    repeating_inputs: Vec<RepeatingInput>,
}

struct RepeatingInput {
    next_time: Option<Duration>,
    input_type: InputType,
    key: KeyCode,
}

impl Default for RepeatTimes {
    fn default() -> Self {
        use InputType::*;
        Self {
            repeating_inputs: vec![
                RepeatingInput::new(ShiftEvent(Shift::Left), KeyCode::ArrowLeft),
                RepeatingInput::new(ShiftEvent(Shift::Right), KeyCode::ArrowRight),
                RepeatingInput::new(RotateEvent(Rot::Ccw), KeyCode::KeyZ),
                RepeatingInput::new(RotateEvent(Rot::Cw), KeyCode::KeyX),
                RepeatingInput::new(DownEvent, KeyCode::ArrowDown),
            ],
        }
    }
}

impl RepeatingInput {
    fn new(input_type: InputType, key: KeyCode) -> Self {
        Self {
            next_time: None,
            input_type,
            key,
        }
    }

    fn get_event(&mut self, now: Duration, keys: &ButtonInput<KeyCode>) -> Option<InputEvent> {
        match (
            keys.pressed(self.key) || keys.just_pressed(self.key),
            self.next_time,
        ) {
            // Not pressed, reset
            (false, _) => {
                self.next_time = None;
                None
            }
            // Pressed for the first time
            (true, None) => {
                self.next_time = Some(now + INITIAL_REPEAT);
                Some(InputEvent {
                    input_type: self.input_type,
                    is_repeat: false,
                })
            }
            // Button is being Held
            (true, Some(ref mut target)) => {
                if *target <= now {
                    *target += REPEAT;
                    Some(InputEvent {
                        input_type: self.input_type,
                        is_repeat: true,
                    })
                } else {
                    None
                }
            }
        }
    }
}

fn update_for_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut repeat_times: ResMut<RepeatTimes>,
    mut input_event_writer: EventWriter<InputEvent>,
) {
    let now = time.elapsed();

    for repeating in &mut repeat_times.repeating_inputs {
        if let Some(event) = repeating.get_event(now, &keys) {
            input_event_writer.send(event);
        }
    }

    // Non-repeating events
    if keys.just_pressed(KeyCode::Space) {
        input_event_writer.send(InputEvent {
            input_type: InputType::DropEvent,
            is_repeat: false,
        });
    }

    if keys.just_pressed(KeyCode::KeyC) {
        input_event_writer.send(InputEvent {
            input_type: InputType::HoldEvent,
            is_repeat: false,
        });
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        input_event_writer.send(InputEvent {
            input_type: InputType::JumpToBotStartPositionEvent,
            is_repeat: false,
        });
    }

    if keys.just_pressed(KeyCode::KeyW) {
        input_event_writer.send(InputEvent {
            input_type: InputType::PerformBotMoveEvent,
            is_repeat: false,
        });
    }

    if keys.just_pressed(KeyCode::KeyG) {
        input_event_writer.send(InputEvent {
            input_type: InputType::EnqueueGarbageEvent(1),
            is_repeat: false,
        });
    }
}
