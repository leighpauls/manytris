use std::collections::BTreeMap;
use std::iter;

use crate::consts;
use bevy::prelude::*;
use rand::{thread_rng, RngCore};

use crate::game_state::{LockResult, TickMutation};
use crate::plugins::root::{GameId, LockEvent, TickEvent, TickMutationMessage};
use crate::plugins::system_sets::UpdateSystems;
use crate::shapes::Shape;

#[derive(Component, Default)]
pub struct ShapeProducer {
    history_cursors: BTreeMap<GameId, usize>,
    history: Vec<Shape>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, update.in_set(UpdateSystems::LocalEventProducers));
}

pub fn setup(mut commands: Commands) {
    commands.spawn(ShapeProducer::default());
}

fn update(
    mut sp_q: Query<&mut ShapeProducer>,
    mut reader: EventReader<LockEvent>,
    mut writer: EventWriter<TickEvent>,
) {
    let mut sp = sp_q.single_mut();

    for event in reader.read() {
        if let LockEvent {
            lock_result: LockResult::Ok { lines_cleared: _ },
            game_id,
        } = event
        {
            writer.send(TickEvent::new_local(TickMutationMessage {
                mutation: TickMutation::EnqueueTetromino(sp.take(game_id)),
                game_id: game_id.clone(),
            }));
        }
    }
}

impl ShapeProducer {
    pub fn take(&mut self, game_id: &GameId) -> Shape {
        let cursor = self.history_cursors.entry(game_id.clone()).or_insert(0);
        while *cursor >= self.history.len() {
            Self::refill(&mut self.history);
        }
        let res = self.history[*cursor];
        *cursor += 1;
        res
    }

    pub fn take_initial_state(&mut self, game_id: &GameId) -> Vec<Shape> {
        iter::repeat_with(|| self.take(&game_id))
            .take(consts::NUM_PREVIEWS * 2)
            .collect()
    }

    fn refill(history: &mut Vec<Shape>) {
        let mut bag: Vec<_> = enum_iterator::all::<Shape>().collect();
        while !bag.is_empty() {
            let next_idx = thread_rng().next_u32() as usize % bag.len();
            history.push(bag.remove(next_idx))
        }
    }
}
