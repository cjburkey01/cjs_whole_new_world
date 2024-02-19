//! Implementation of a voxel oct-tree-esque structure to track which chunks
//! of which LOD level need to be loaded. This is not a real oct tree. It's a
//! farce.

mod lod_chunk;
mod lod_pos;
mod lod_state;
mod lod_world;
mod oct_base;

use std::time::Duration;

pub use lod_chunk::*;
pub use lod_pos::*;
pub use lod_state::*;
pub use lod_world::*;
pub use oct_base::*;

use bevy::{prelude::*, time::common_conditions::on_timer};

pub struct OctLodPlugin;

impl Plugin for OctLodPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<WorldState>()
            .add_systems(OnEnter(WorldState::NoExist), delete_world_system)
            .add_systems(
                OnEnter(WorldState::Initializing),
                start_initializing_world_system.run_if(not(resource_exists::<LodWorld>())),
            )
            .add_systems(
                Update,
                wait_for_finish_initializing_system
                    .run_if(on_timer(Duration::from_millis(500)))
                    .run_if(in_state(WorldState::Initializing)),
            );
    }
}

fn start_initializing_world_system(mut commands: Commands) {
    commands.init_resource::<LodWorld>();
}

fn delete_world_system(mut commands: Commands) {
    commands.remove_resource::<LodWorld>();
}

fn wait_for_finish_initializing_system(mut next_world_state: ResMut<NextState<WorldState>>) {
    // TODO: WAIT
    next_world_state.set(WorldState::Ready);
}
