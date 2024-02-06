use super::{
    beef::{ChunkEntity, ChunkState, FixedChunkWorld, LoadedChunk},
    chunk_pos::ChunkPos,
    control::{
        input::{create_input_manager_bundle, PlyAction},
        pause::PauseState,
        PlyCamRot, PrimaryCamera,
    },
};
use crate::{
    plugin::beef::DirtyChunk,
    voxel::{InChunkPos, Voxel, CHUNK_WIDTH},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::{action_state::ActionState, InputManagerBundle};
use std::{f32::consts::PI, time::Duration};

pub struct Controller2ElectricBoogalooPlugin;

impl Plugin for Controller2ElectricBoogalooPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLookAtRes>().add_systems(
            Update,
            (
                (look_at_voxels, modify_block_system).chain(),
                (
                    update_character_controller_rotations,
                    update_character_controller_position,
                )
                    .chain(),
                unfreeze_after_time_system.run_if(on_timer(Duration::from_secs(5))),
            )
                .run_if(in_state(PauseState::Playing)),
        );
    }
}

fn unfreeze_after_time_system(mut commands: Commands, ply: Query<Entity, With<PlayerStartFrozen>>) {
    if let Ok(ply) = ply.get_single() {
        commands.entity(ply).remove::<PlayerStartFrozen>();
    }
}

fn modify_block_system(
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkWorld>,
    look_at: Res<PlayerLookAtRes>,
    camera: Query<&ActionState<PlyAction>>,
) {
    if let Ok(ctrl) = camera.get_single() {
        if ctrl.just_pressed(PlyAction::Fire) {
            if let Some(look_at) = &look_at.0 {
                let ent = chunks.chunks.get(&look_at.chunk_pos).map(|lc| lc.entity);
                if let (Some(entity), Some(chunk)) =
                    (ent, chunks.regions.get_chunk_mut(look_at.chunk_pos))
                {
                    chunk.set(look_at.voxel_pos_in_chunk, Voxel::Air);
                    commands.entity(entity).insert(DirtyChunk);
                }
            }
        }
    }
}

fn look_at_voxels(
    chunks: Res<FixedChunkWorld>,
    rapier_context: Res<RapierContext>,
    mut look_at: ResMut<PlayerLookAtRes>,
    mut gizmos: Gizmos,
    camera: Query<&GlobalTransform, With<PrimaryCamera>>,
    chunk_query: Query<&ChunkEntity>,
) {
    let Ok(camera) = camera.get_single() else {
        return;
    };

    look_at.0 = None;

    if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
        camera.translation(),
        camera.forward(),
        30.0,
        false,
        QueryFilter::only_fixed(),
    ) {
        if let Ok(ChunkEntity(chunk_pos)) = chunk_query.get(entity) {
            if let (
                Some(LoadedChunk {
                    state: ChunkState::Rendered,
                    ..
                }),
                Some(chunk),
            ) = (
                chunks.chunks.get(chunk_pos),
                chunks.regions.chunk(*chunk_pos),
            ) {
                let global_voxel_pos = (intersection.point - intersection.normal.normalize() * 0.3)
                    .floor()
                    .as_ivec3();
                let voxel_pos_in_chunk = InChunkPos::new(
                    global_voxel_pos
                        .rem_euclid(UVec3::splat(CHUNK_WIDTH).as_ivec3())
                        .as_uvec3(),
                )
                .unwrap();

                look_at.0 = Some(PlayerLookAt {
                    normal: intersection.normal,
                    chunk_pos: *chunk_pos,
                    global_voxel_pos,
                    voxel_pos_in_chunk,
                    voxel: chunk.at(voxel_pos_in_chunk),
                });

                gizmos.cuboid(
                    Transform::from_translation(
                        (*chunk_pos * CHUNK_WIDTH as i32 + voxel_pos_in_chunk.pos().as_ivec3())
                            .as_vec3()
                            + Vec3::splat(0.5),
                    ),
                    Color::WHITE,
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_character_controller_position(
    time: Res<Time>,
    phys: Res<RapierConfiguration>,
    mut query: Query<
        (
            &Transform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
            &CharControl2,
            &mut Velocity,
            &ActionState<PlyAction>,
        ),
        Without<PlayerStartFrozen>,
    >,
) {
    for (transform, mut controller, controller_output, ply_cam, mut vel, ctrl) in query.iter_mut() {
        // Move camera
        let key_motion = ctrl.axis_pair(PlyAction::LateralMove).unwrap();
        let y_velocity = vel.linvel.y;
        vel.linvel = (transform.rotation * Vec3::new(key_motion.x(), 0.0, -key_motion.y()))
            .normalize_or_zero();
        vel.linvel *= if ctrl.pressed(PlyAction::Fast) {
            ply_cam.run_speed
        } else {
            ply_cam.walk_speed
        };

        vel.linvel.y = y_velocity.min(ply_cam.jump_velocity);
        if let Some(controller_output) = controller_output {
            if controller_output.grounded {
                if ctrl.just_pressed(PlyAction::Jump) {
                    vel.linvel.y = ply_cam.jump_velocity;
                } else {
                    vel.linvel.y = 0.0;
                }
            }
        }
        vel.linvel += phys.gravity * time.delta_seconds();
        controller.translation = Some(vel.linvel * time.delta_seconds());
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

#[derive(Default, Resource)]
pub struct PlayerLookAtRes(pub Option<PlayerLookAt>);

#[derive(Clone)]
pub struct PlayerLookAt {
    pub normal: Vec3,
    pub chunk_pos: IVec3,
    pub global_voxel_pos: IVec3,
    pub voxel_pos_in_chunk: InChunkPos,
    pub voxel: Voxel,
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

#[derive(Component)]
pub struct PlayerStartFrozen;

#[derive(Bundle)]
pub struct CharacterControllerParentBundle {
    transform: TransformBundle,
    control_settings: CharControl2,
    velocity: Velocity,
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
            transform: default(),
            control_settings: default(),
            velocity: default(),
            rotation: default(),
            input_manager: create_input_manager_bundle(),
            collider: Collider::cylinder(0.8, 0.3),
            controller: KinematicCharacterController {
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.03),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(1.1),
                    min_width: CharacterLength::Absolute(0.25),
                    include_dynamic_bodies: false,
                }),
                // max_slope_climb_angle: 46.0f32.to_radians() as bevy_rapier3d::prelude::Real,
                // min_slope_slide_angle: 30.0f32.to_radians() as bevy_rapier3d::prelude::Real,
                apply_impulse_to_dynamic_bodies: true,
                ..default()
            },
            rigidbody: RigidBody::KinematicPositionBased,
            chunk_pos: default(),
        }
    }
}
