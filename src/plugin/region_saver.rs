use crate::{io::write_regions_to_file, plugin::beef::FixedChunkWorld, voxel::RegionHandler};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer};
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
            info!("saving world!");
            debug!("extracting chunks into region handler");
            region_handler.extract_chunks(&chunk_world);
        }
        Err(_) => {
            error!("FAILED TO LOCK REGION HANDLER TO EXTRACT WORLD!!!");
        }
    }

    let region_handler_inner = Arc::clone(&region_handler.0);
    let world_name = chunk_world.name().to_string();
    AsyncComputeTaskPool::get()
        .spawn(async move {
            match region_handler_inner.read() {
                Ok(region_handler) => {
                    debug!("saving regions to disk");
                    write_regions_to_file(&world_name, &region_handler);
                    info!("world saved!");
                }
                Err(_) => {
                    error!("FAILED TO LOCK REGION HANDLER TO SAVE REGIONS!!!");
                }
            }
        })
        .detach();
}
