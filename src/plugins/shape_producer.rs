use crate::consts;
use crate::game_state::{LockResult, TickMutation};
use crate::plugins::root::{LockEvent, TickEvent};
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use crate::shapes::Shape;
use bevy::prelude::*;
use rand::{thread_rng, RngCore};

#[derive(Component)]
pub struct ShapeProducer {
    upcoming_blocks: Vec<Shape>,
    bag_remaining: Vec<Shape>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup.in_set(StartupSystems::AfterRoot))
        .add_systems(Update, update.in_set(UpdateSystems::LocalEventProducers));
}

fn setup(mut commands: Commands) {
    commands.spawn(ShapeProducer::new());
}

fn update(
    mut sp_q: Query<&mut ShapeProducer>,
    mut reader: EventReader<LockEvent>,
    mut writer: EventWriter<TickEvent>,
) {
    let mut sp = sp_q.single_mut();

    for event in reader.read() {
        if let LockEvent(LockResult::Ok { lines_cleared: _ }) = event {
            println!("Producing new tetromino");
            writer.send(TickEvent::new_local(TickMutation::EnqueueTetromino(
                sp.take(),
            )));
        }
    }
}

impl ShapeProducer {
    fn new() -> Self {
        let mut res = Self {
            upcoming_blocks: vec![],
            bag_remaining: vec![],
        };
        res.refill();
        res
    }

    fn refill(&mut self) {
        while self.upcoming_blocks.len() < consts::NUM_PREVIEWS * 2 {
            if self.bag_remaining.is_empty() {
                self.bag_remaining = enum_iterator::all::<Shape>().collect();
            }
            let next_idx = thread_rng().next_u32() as usize % self.bag_remaining.len();
            self.upcoming_blocks
                .push(self.bag_remaining.remove(next_idx));
        }
    }

    fn take(&mut self) -> Shape {
        let res = self.upcoming_blocks.remove(0);
        self.refill();
        res
    }
}
