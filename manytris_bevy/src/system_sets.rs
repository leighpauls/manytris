use bevy::prelude::*;

/// Configure the system sets' run ordering
pub fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            UpdateSystems::Input,
            UpdateSystems::LocalEventProducers,
            UpdateSystems::EventSenders,
            UpdateSystems::RootTick,
            UpdateSystems::PreRender,
            UpdateSystems::Render,
        )
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSystems {
    Input,
    LocalEventProducers,
    EventSenders,
    RootTick,
    PreRender,
    Render,
}
