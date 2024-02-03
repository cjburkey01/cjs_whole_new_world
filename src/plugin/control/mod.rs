pub mod input;
pub mod pause;

use bevy::{prelude::*, window::CursorGrabMode};
use pause::PauseState;

pub struct PlyControlPlugin;

impl Plugin for PlyControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PauseState>()
            .add_systems(OnEnter(PauseState::Paused), on_pause_system)
            .add_systems(OnExit(PauseState::Paused), on_unpause_system);
    }
}

#[derive(Default, Component, Debug)]
pub struct PrimaryCamera;

#[derive(Component, Debug)]
pub struct PlyCamControl {
    pub speed: f32,
    pub fast_speed: f32,
    pub rot_speed: f32,
}

impl Default for PlyCamControl {
    fn default() -> Self {
        Self {
            speed: 20.0,
            fast_speed: 100.0,
            rot_speed: 0.2,
        }
    }
}

#[derive(Default, Component, Debug)]
pub struct PlyCamRot(pub Vec2);

fn on_pause_system(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
    info!("Paused");
}

fn on_unpause_system(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
    info!("Unpaused");
}
