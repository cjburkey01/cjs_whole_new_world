use super::{
    chunk_pos::ChunkPos,
    control::{
        input::{create_input_manager_bundle, PlyAction},
        pause::PauseState,
        PlyCamRot, PrimaryCamera,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::{action_state::ActionState, InputManagerBundle};
use std::f32::consts::PI;

pub struct Controller2ElectricBoogalooPlugin;

impl Plugin for Controller2ElectricBoogalooPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_character_controller_rotations,
                update_character_controller_position,
            )
                .chain()
                .run_if(in_state(PauseState::Playing)),
        );
    }
}

#[allow(clippy::type_complexity)]
fn update_character_controller_position(
    time: Res<Time>,
    phys: Res<RapierConfiguration>,
    mut query: Query<(
        &Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &CharControl2,
        &mut CharControl2Velocity,
        &ActionState<PlyAction>,
    )>,
) {
    for (transform, mut controller, controller_output, ply_cam, mut vel, ctrl) in query.iter_mut() {
        // Move camera
        let key_motion = ctrl.axis_pair(PlyAction::LateralMove).unwrap();
        let y_velocity = vel.0.y;
        vel.0 = (transform.rotation * Vec3::new(key_motion.x(), 0.0, -key_motion.y()))
            .normalize_or_zero();
        vel.0 *= if ctrl.pressed(PlyAction::Fast) {
            ply_cam.run_speed
        } else {
            ply_cam.walk_speed
        };

        vel.0.y = y_velocity;
        if let Some(controller_output) = controller_output {
            if controller_output.grounded {
                if ctrl.just_pressed(PlyAction::Jump) {
                    vel.0.y = ply_cam.jump_velocity;
                } else {
                    vel.0.y = 0.0;
                }
            }
        }
        vel.0 += phys.gravity * time.delta_seconds();
        controller.translation = Some(vel.0 * time.delta_seconds());
    }
}

#[allow(clippy::type_complexity)]
fn update_character_controller_rotations(
    mut controller: Query<
        (
            Entity,
            &mut Transform,
            &CharControl2,
            &mut PlyCamRot,
            &ActionState<PlyAction>,
        ),
        (With<KinematicCharacterController>, Without<PrimaryCamera>),
    >,
    mut camera: Query<(&Parent, &mut Transform), With<PrimaryCamera>>,
) {
    let (
        Ok((entity, mut controller_transform, controller, mut cam_rot, action)),
        Ok((parent, mut camera_transform)),
    ) = (controller.get_single_mut(), camera.get_single_mut())
    else {
        return;
    };

    // idk check this just in case
    if parent.get() == entity {
        let mouse_motion = action.axis_pair(PlyAction::Look).unwrap();
        cam_rot.0 += Vec2::new(1.0, -1.0) * mouse_motion.xy() * controller.rot_speed * PI / 180.0;
        cam_rot.0.y = cam_rot.0.y.max(-PI / 2.0).min(PI / 2.0);
        controller_transform.rotation = Quat::from_axis_angle(Vec3::Y, -cam_rot.0.x);
        camera_transform.rotation = Quat::from_axis_angle(Vec3::X, cam_rot.0.y);
    }
}

#[derive(Component)]
pub struct CharControl2 {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_velocity: f32,
    pub rot_speed: f32,
}

impl Default for CharControl2 {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            run_speed: 10.0,
            jump_velocity: 7.0,
            rot_speed: 0.2,
        }
    }
}

#[derive(Default, Component)]
pub struct CharControl2Velocity(pub Vec3);

#[derive(Bundle)]
pub struct CharacterControllerParentBundle {
    transform: TransformBundle,
    control_settings: CharControl2,
    velocity: CharControl2Velocity,
    rotation: PlyCamRot,
    input_manager: InputManagerBundle<PlyAction>,
    collider: Collider,
    controller: KinematicCharacterController,
    rigidbody: RigidBody,
    chunk_pos: ChunkPos,
}

impl Default for CharacterControllerParentBundle {
    fn default() -> Self {
        Self {
            transform: TransformBundle::from_transform(Transform::from_xyz(15.5, 145.0, 15.5)),
            control_settings: CharControl2::default(),
            velocity: CharControl2Velocity::default(),
            rotation: PlyCamRot::default(),
            input_manager: create_input_manager_bundle(),
            collider: Collider::cuboid(0.5, 1.0, 0.5),
            controller: KinematicCharacterController {
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: false,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(1.1),
                    min_width: CharacterLength::Absolute(0.15),
                    include_dynamic_bodies: false,
                }),
                max_slope_climb_angle: 46.0f32.to_radians() as bevy_rapier3d::prelude::Real,
                min_slope_slide_angle: 30.0f32.to_radians() as bevy_rapier3d::prelude::Real,
                apply_impulse_to_dynamic_bodies: true,
                ..default()
            },
            rigidbody: RigidBody::KinematicPositionBased,
            chunk_pos: ChunkPos::default(),
        }
    }
}
