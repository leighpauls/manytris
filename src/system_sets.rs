use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StartupSystemSets {
    Root,
    AfterRoot,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSystemSets {
    Input,
    RootTick,
    Render,
}
