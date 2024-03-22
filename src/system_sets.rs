use bevy::prelude::*;

/// Configure the system sets' run ordering
pub fn system_sets_plugin(app: &mut App) {
    app.configure_sets(
        Startup,
        (StartupSystems::Root, StartupSystems::AfterRoot).chain(),
    )
    .configure_sets(
        Update,
        (
            UpdateSystems::Input,
            UpdateSystems::RootTick,
            UpdateSystems::Render,
        )
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSystems {
    Root,
    AfterRoot,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSystems {
    Input,
    RootTick,
    Render,
}
