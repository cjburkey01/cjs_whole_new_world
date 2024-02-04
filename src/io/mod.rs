use crate::voxel::Chunk;
use bevy::prelude::IVec3;
use bincode::config::{Configuration, LittleEndian, NoLimit, Varint};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;

pub const SAVES_DIR_NAME: &str = "saves";
pub const CHUNKS_DIR_NAME: &str = "chunks";

lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("com", "cjburkey", "cjs_whole_new_world").unwrap();
    pub static ref MAIN_DIR: PathBuf = PROJECT_DIRS.config_dir().to_path_buf();
    pub static ref SAVES_DIR: PathBuf = MAIN_DIR.join(SAVES_DIR_NAME);
}

pub fn saves_dir(world_name: &str) -> PathBuf {
    SAVES_DIR.join(world_name)
}

pub fn chunks_dir(world_name: &str) -> PathBuf {
    saves_dir(world_name).join(CHUNKS_DIR_NAME)
}

pub fn chunk_file(world_name: &str, IVec3 { x, y, z }: IVec3) -> PathBuf {
    chunks_dir(world_name).join(format!("{x}_{y}_{z}.chunk"))
}

pub fn write_chunk(world_name: &str, chunk_pos: IVec3, chunk: &Chunk) {
    std::fs::create_dir_all(&chunks_dir(world_name)).unwrap();
    let file_path = chunk_file(world_name, chunk_pos);

    std::fs::write(
        &file_path,
        bincode::serde::encode_to_vec(
            &chunk.voxels,
            Configuration::<LittleEndian, Varint, NoLimit>::default(),
        )
        .unwrap(),
    )
    .unwrap();
}
