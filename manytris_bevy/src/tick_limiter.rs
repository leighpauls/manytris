use std::time::{Duration, Instant};

use bevy::prelude::*;

#[derive(Debug, Clone, Resource)]
pub struct TickLimiterPlugin {
    pub target_fps: u32,
}

impl Plugin for TickLimiterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, do_sleep)
            .insert_resource(self.clone())
            .insert_resource(TickTarget::default());
    }
}

#[derive(Debug, Clone, Resource, Default)]
struct TickTarget {
    next_frame_end_target: Option<Instant>,
}

fn do_sleep(mut target: ResMut<TickTarget>, config: Res<TickLimiterPlugin>) {
    let desired_time = Duration::from_secs(1) / config.target_fps;
    if let Some(t) = target.next_frame_end_target {
        let elapsed_time = t.elapsed();
        if desired_time > elapsed_time {
            spin_sleep::sleep(desired_time - elapsed_time);
        }
    }
    target.next_frame_end_target = Some(Instant::now() + desired_time);
}
