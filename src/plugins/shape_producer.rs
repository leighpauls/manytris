use std::collections::BTreeMap;
use std::iter;

use bevy::prelude::*;
use manytris_core::consts;

use crate::plugins::root::{GameId, LockEvent, TickEvent, TickMutationMessage};
use crate::plugins::states;
use crate::plugins::states::PlayingState;
use crate::plugins::system_sets::UpdateSystems;
use manytris_core::game_state::{LockResult, TickMutation};
use manytris_core::shape_bag::ShapeBag;
use manytris_core::shapes::Shape;

#[derive(Component, Default)]
pub struct ShapeProducer {
    history_cursors: BTreeMap<GameId, usize>,
    history: Vec<Shape>,
    shape_bag: ShapeBag,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(PlayingState::Playing),
        setup.run_if(states::produces_shapes),
    )
    .add_systems(
        OnExit(PlayingState::Playing),
        teardown.run_if(states::produces_shapes),
    )
    .add_systems(
        Update,
        update
            .in_set(UpdateSystems::LocalEventProducers)
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::produces_shapes),
    );
}

pub fn setup(mut commands: Commands) {
    commands.spawn(ShapeProducer::default());
}

pub fn teardown(mut commands: Commands, producer_q: Query<Entity, With<ShapeProducer>>) {
    commands.entity(producer_q.single()).despawn();
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
            self.history.push(self.shape_bag.next().unwrap());
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
}
