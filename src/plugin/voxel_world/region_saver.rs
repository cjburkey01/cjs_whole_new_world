use crate::{
    io::write_regions_to_file,
    plugin::voxel_world::{beef::FixedChunkWorld, world_info::WorldInfo},
    voxel::{RegionHandler, RegionMaintainerPlugin},
};
use bevy::{
    app::AppExit, prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct RegionSaverPlugin;

impl Plugin for RegionSaverPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RegionMaintainerPlugin)
            // .add_systems(
            //     Update,
            //     async_ish_save_regions_system
            //         .run_if(resource_exists::<FixedChunkWorld>())
            //         .run_if(resource_exists::<RegionHandlerRes>())
            //         .run_if(on_timer(Duration::from_secs(300))),
            // )
            .add_systems(Last, save_regions_on_exit_system);
    }
}

#[derive(Default, Resource)]
pub struct RegionHandlerRes(pub Arc<RwLock<RegionHandler>>);

fn save_regions_on_exit_system(
    exit_reader: EventReader<AppExit>,
    world_info: Option<Res<WorldInfo>>,
    region_handler: Option<Res<RegionHandlerRes>>,
    chunk_world: Option<Res<FixedChunkWorld>>,
) {
    if !exit_reader.is_empty() {
        debug!("exiting game, checking if we need to save regions");
        if let (Some(world_info), Some(region_handler), Some(chunk_world)) =
            (world_info, region_handler, chunk_world)
        {
            force_sync_regions_save(&world_info, &region_handler, &chunk_world);
        }
    }
}

fn async_ish_save_regions_system(
    world_info: Res<WorldInfo>,
    region_handler: Res<RegionHandlerRes>,
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
    let world_name = world_info.name().to_string();
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

pub fn force_sync_regions_save(
    world_info: &WorldInfo,
    region_handler: &RegionHandlerRes,
    chunk_world: &FixedChunkWorld,
) {
    debug!("forcing world save");
    match region_handler.0.write() {
        Ok(mut region_handler) => {
            info!("saving world!");
            region_handler.extract_chunks(chunk_world);
        }
        Err(_) => {
            error!("FAILED TO LOCK REGION HANDLER TO EXTRACT WORLD!!!");
        }
    }
    match region_handler.0.read() {
        Ok(region_handler) => {
            write_regions_to_file(world_info.name(), &region_handler);
            info!("world saved!");
        }
        Err(_) => {
            error!("FAILED TO LOCK REGION HANDLER TO SAVE REGIONS!!!");
        }
    }
}
