use crate::input::InputEvent::{DownEvent, RotateEvent, ShiftEvent};
use crate::shapes::{Rot, Shift};
use bevy::prelude::*;
use bevy::utils::Duration;

const INITIAL_REPEAT: Duration = Duration::from_millis(200);
const REPEAT: Duration = Duration::from_millis(100);

#[derive(Event, Copy, Clone)]
pub enum InputEvent {
    ShiftEvent(Shift),
    RotateEvent(Rot),
    DownEvent,
    DropEvent,
}

#[derive(Resource)]
pub struct RepeatTimes {
    repeating_inputs: Vec<RepeatingInput>,
}

struct RepeatingInput {
    next_time: Option<Duration>,
    event: InputEvent,
    key: KeyCode,
}

impl Default for RepeatTimes {
    fn default() -> Self {
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
    fn new(event: InputEvent, key: KeyCode) -> Self {
        Self {
            next_time: None,
            event,
            key,
        }
    }

    fn get_event(&mut self, now: Duration, keys: &ButtonInput<KeyCode>) -> Option<InputEvent> {
        match (keys.pressed(self.key), self.next_time) {
            (false, _) => {
                self.next_time = None;
                None
            }
            (true, None) => {
                self.next_time = Some(now + INITIAL_REPEAT);
                Some(self.event)
            }
            (true, Some(ref mut target)) => {
                if *target <= now {
                    *target += REPEAT;
                    Some(self.event)
                } else {
                    None
                }
            }
        }
    }
}

pub fn update_for_input(
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
        input_event_writer.send(InputEvent::DropEvent);
    }
}
