use bevy::prelude::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum PauseState {
    #[default]
    Paused,
    Playing,
}

impl PauseState {
    pub fn toggled(&self) -> Self {
        match self {
            Self::Paused => Self::Playing,
            Self::Playing => Self::Paused,
        }
    }
}
