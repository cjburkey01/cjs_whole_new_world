use crate::{io::write_regions_to_file, plugin::beef::FixedChunkWorld, voxel::RegionHandler};
use bevy::{prelude::*, time::common_conditions::on_timer};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct RegionSaverPlugin;

impl Plugin for RegionSaverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            save_regions_system
                .run_if(resource_exists::<FixedChunkWorld>())
                .run_if(resource_exists::<RegionHandlerRes>())
                .run_if(on_timer(Duration::from_secs(20))),
        );
    }
}

#[derive(Default, Resource)]
pub struct RegionHandlerRes(pub Arc<RwLock<RegionHandler>>);

fn save_regions_system(
    region_handler: ResMut<RegionHandlerRes>,
    chunk_world: Res<FixedChunkWorld>,
) {
    match region_handler.0.write() {
        Ok(mut region_handler) => {
            region_handler.extract_chunks(&chunk_world);
            write_regions_to_file(chunk_world.name(), &region_handler);
        }
        Err(_) => {
            panic!("u done fucked up more");
        }
    }
}
