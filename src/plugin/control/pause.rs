use bevy::prelude::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum PauseState {
    #[default]
    Paused,
    Playing,
}
