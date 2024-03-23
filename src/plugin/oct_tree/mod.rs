//! Implementation of a voxel oct-tree-esque structure to track which chunks
//! of which LOD level need to be loaded. This is not a real oct tree. It's a
//! farce.

mod lod_chunk;
mod lod_pos;
mod lod_state;
mod lod_task;
mod lod_world;
mod oct_base;
mod oct_loader;

pub use lod_chunk::*;
pub use lod_pos::*;
pub use lod_state::*;
pub use lod_task::*;
pub use lod_world::*;
pub use oct_base::*;
pub use oct_loader::*;

use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

use crate::voxel_world::world_state::WorldState;

pub struct OctLodPlugin;

impl Plugin for OctLodPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OctLoaderPlugin)
            // .add_systems(OnEnter(WorldState::NotInWorld), delete_world_system)
            // .add_systems(
            //     OnEnter(WorldState::LoadingStartArea),
            //     start_initializing_world_system.run_if(not(resource_exists::<LodWorld>())),
            // )
            .add_systems(
                Update,
                wait_for_finish_initializing_system
                    .run_if(on_timer(Duration::from_millis(500)))
                    .run_if(in_state(WorldState::LoadingStartArea)),
            );
    }
}

fn wait_for_finish_initializing_system(mut next_world_state: ResMut<NextState<WorldState>>) {
    // TODO: WAIT
    next_world_state.set(WorldState::WorldLoaded);
}
