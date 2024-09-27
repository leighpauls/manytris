use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game_state::GameState;
use crate::plugins::assets::RenderAssets;
use crate::plugins::root;
use crate::plugins::system_sets::UpdateSystems;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ControlEvent {
    JoinRequest,
    SnapshotResponse(GameState, Uuid),
}

#[derive(Event)]
pub struct SendControlEvent(pub ControlEvent);

#[derive(Event)]
pub struct ReceiveControlEvent(pub ControlEvent);

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_for_control_events.in_set(UpdateSystems::LocalEventProducers),
    )
    .add_event::<SendControlEvent>()
    .add_event::<ReceiveControlEvent>();
}

pub fn update_for_control_events(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    mut control_event_reader: EventReader<ReceiveControlEvent>,
    mut control_event_writer: EventWriter<SendControlEvent>,
    time: Res<Time<Fixed>>,
) {
    let cur_time = time.elapsed();

    for rce in control_event_reader.read() {
        let ReceiveControlEvent(ce) = rce;
        match ce {
            ControlEvent::JoinRequest => {
                let (game_state, game_id) =
                    root::create_new_root(&mut commands, &ra, &asset_server, cur_time);

                control_event_writer.send(SendControlEvent(ControlEvent::SnapshotResponse(
                    game_state, game_id,
                )));
            }
            ControlEvent::SnapshotResponse(gs, game_id) => {
                root::create_root_from_snapshot(
                    &mut commands,
                    &ra,
                    &asset_server,
                    gs.clone(),
                    cur_time,
                    game_id.clone(),
                );
            }
        }
    }
}
