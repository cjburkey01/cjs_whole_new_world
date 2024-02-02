use bevy::prelude::*;

pub struct GameSettingsPlugin;

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>();
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub load_radius: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { load_radius: 2 }
    }
}
