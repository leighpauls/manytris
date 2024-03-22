use crate::entities::FieldComponent;
use crate::shapes::{Rot, Shift};
use bevy::prelude::*;
use bevy::utils::Duration;

const INITIAL_REPEAT: Duration = Duration::from_millis(200);
const REPEAT: Duration = Duration::from_millis(100);

#[derive(Resource, Default)]
pub struct RepeatTimes {
    left: RepeatingInput,
    right: RepeatingInput,
    down: RepeatingInput,
    ccw: RepeatingInput,
    cw: RepeatingInput,
}

#[derive(Default)]
struct RepeatingInput {
    next_time: Option<Duration>,
}

impl RepeatingInput {
    fn apply<F: FnOnce()>(
        &mut self,
        now: Duration,
        keys: &ButtonInput<KeyCode>,
        key: KeyCode,
        action: F,
    ) {
        match (keys.pressed(key), self.next_time) {
            (false, _) => {
                self.next_time = None;
            }
            (true, None) => {
                self.next_time = Some(now + INITIAL_REPEAT);
                action();
            }
            (true, Some(ref mut target)) => {
                if *target <= now {
                    *target += REPEAT;
                    action();
                }
            }
        }
    }
}

pub fn update_for_input(
    mut q_field: Query<&mut FieldComponent>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    mut repeat_times: ResMut<RepeatTimes>,
) {
    let gs = &mut q_field.single_mut().game;
    let now = time.elapsed();

    repeat_times.left.apply(now, &keys, KeyCode::ArrowLeft, || {
        gs.shift(Shift::Left);
    });
    repeat_times
        .right
        .apply(now, &keys, KeyCode::ArrowRight, || {
            gs.shift(Shift::Right);
        });
    repeat_times.down.apply(now, &keys, KeyCode::ArrowDown, || {
        gs.down();
    });
    repeat_times.ccw.apply(now, &keys, KeyCode::KeyZ, || {
        gs.rotate(Rot::Ccw);
    });
    repeat_times.cw.apply(now, &keys, KeyCode::KeyX, || {
        gs.rotate(Rot::Cw);
    });

    if keys.just_pressed(KeyCode::Space) {
        gs.drop();
    }
}
