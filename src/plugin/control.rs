use super::{
    input::{create_input_manager_bundle, PlyAction},
    pause::PauseState,
};
use bevy::{prelude::*, window::CursorGrabMode};
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

pub struct PlyControlPlugin;

impl Plugin for PlyControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PauseState>()
            .add_systems(Update, toggle_pause_system)
            .add_systems(
                Update,
                (rotate_ply_system, move_ply_system).run_if(in_state(PauseState::Playing)),
            )
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
            fast_speed: 40.0,
            rot_speed: 0.2,
        }
    }
}

#[derive(Default, Component, Debug)]
pub struct PlyCamRot(pub Vec2);

fn move_ply_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PlyCamControl, &ActionState<PlyAction>)>,
) {
    for (mut transform, ply_cam, ctrl) in query.iter_mut() {
        // Move camera
        let key_motion = ctrl.axis_pair(PlyAction::LateralMove).unwrap();
        let mut velocity = Vec3::new(key_motion.x(), 0.0, -key_motion.y()).normalize_or_zero();

        velocity = (transform.rotation * velocity) + (Vec3::Y * ctrl.value(PlyAction::UpDown));
        transform.translation += velocity
            * time.delta_seconds()
            * if ctrl.pressed(PlyAction::Fast) {
                ply_cam.fast_speed
            } else {
                ply_cam.speed
            };
    }
}

fn rotate_ply_system(
    mut query: Query<(
        &mut Transform,
        &mut PlyCamRot,
        &PlyCamControl,
        &ActionState<PlyAction>,
    )>,
) {
    for (mut transform, mut cam_rot, ply_cam, ctrl) in query.iter_mut() {
        let mouse_motion = ctrl.axis_pair(PlyAction::Look).unwrap();
        cam_rot.0 += Vec2::new(1.0, -1.0) * mouse_motion.xy() * ply_cam.rot_speed * PI / 180.0;
        cam_rot.0.y = cam_rot.0.y.max(-PI / 2.0).min(PI / 2.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Y, -cam_rot.0.x)
            * Quat::from_axis_angle(Vec3::X, cam_rot.0.y);
    }
}

fn toggle_pause_system(
    state: ResMut<State<PauseState>>,
    mut next_state: ResMut<NextState<PauseState>>,
    query: Query<&ActionState<PlyAction>>,
) {
    for ctrl in query.iter() {
        if ctrl.just_pressed(PlyAction::Pause) {
            next_state.set(state.toggled())
        }
    }
}

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

#[derive(Bundle)]
pub struct PlyCamBundle {
    pub camera: Camera3dBundle,
    pub input: InputManagerBundle<PlyAction>,

    pub control: PlyCamControl,
    pub rotation: PlyCamRot,
}

impl Default for PlyCamBundle {
    fn default() -> Self {
        Self {
            camera: default(),
            input: create_input_manager_bundle(),
            control: default(),
            rotation: default(),
        }
    }
}
