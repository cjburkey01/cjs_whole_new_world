use crate::{oct_tree::LodPos, voxel_world::world_state::WorldState};
use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashSet};
use std::time::Duration;

pub struct RegionMaintainerPlugin;

impl Plugin for RegionMaintainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_loading_or_generating_needed_system
                .run_if(not(in_state(WorldState::NotInWorld)))
                .run_if(on_timer(Duration::from_millis(200))),
        );
    }
}

#[derive(Default, Resource)]
pub struct NeededLods(HashSet<LodPos>);

impl NeededLods {
    pub fn mark_needed(&mut self, pos: LodPos) {
        self.0.insert(pos);
    }

    pub fn mark_unneeded(&mut self, pos: LodPos) {
        self.0.remove(&pos);
    }

    pub fn needed(&self, pos: LodPos) -> bool {
        self.0.contains(&pos)
    }
}

fn start_loading_or_generating_needed_system(mut commands: Commands, needed: Res<NeededLods>) {
    // TODO: USE ECS TO MANAGE TASKS?
    //       IDK WHAT IM DOING
}
