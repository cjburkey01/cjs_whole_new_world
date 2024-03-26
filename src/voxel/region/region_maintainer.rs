use super::REGION_LOD_LEVEL;
use crate::{
    oct_tree::LodPos,
    voxel::RegionPos,
    voxel_world::{region_saver::RegionHandlerRes, world_state::WorldState},
};
use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashSet};
use itertools::iproduct;
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
pub struct RegionManager {
    needed: HashSet<RegionPos>,
}

impl RegionManager {
    pub fn add_required_lod_positions(&mut self, required: &[LodPos]) {
        let needed_regions = required.iter().fold(HashSet::new(), |mut set, lod_pos| {
            // At lod level 0, there is 1 chunk per lod chunk, at level 1,
            // there are two chunks, at level 2, there are four chunks, at
            // level 3, there are eight chunks, and at level 4, there are
            // 16 chunks, which is the number of chunks wide a region is.
            // So, we can just get the start and end lod positions at lod-4
            // to determine how many regions need to load for the given lod
            // level.
            let start_pos = lod_pos.to_level(REGION_LOD_LEVEL);
            let end_pos =
                LodPos::new(lod_pos.level, lod_pos.pos + IVec3::ONE).to_level(REGION_LOD_LEVEL);
            for (x, y, z) in iproduct!(
                start_pos.pos.x..end_pos.pos.x,
                start_pos.pos.y..end_pos.pos.y,
                start_pos.pos.z..end_pos.pos.z
            ) {
                set.insert(RegionPos(IVec3 { x, y, z }));
            }
            set
        });

        for needed_region in needed_regions {
            self.needed.insert(needed_region);
        }
    }

    pub fn needed(&self) -> &HashSet<RegionPos> {
        &self.needed
    }
}

fn start_loading_or_generating_needed_system(
    mut commands: Commands,
    region_handler: Res<RegionHandlerRes>,
    needed: Res<RegionManager>,
) {
    // TODO: USE ECS TO MANAGE TASKS?
    //       IDK WHAT IM DOING
    let Ok(region_handler) = region_handler.0.read() else {
        return;
    };

    for region_pos in needed.needed.iter() {
        match region_handler.region(*region_pos) {
            // Don't gotta do nothing
            Some(_region) => {
                todo!("region loaded");
            }
            // Gotta spawn the task to check whether this chunk exists already or needs to be generated.
            // That whole system needs to be rewritten too, it is heavily reliant upon beef.
            None => {
                todo!("region not loaded");
            }
        }
    }
}
