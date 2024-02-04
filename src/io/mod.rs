use crate::voxel::VoxelContainer;
use bevy::prelude::IVec3;
use bincode::config::Configuration;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::{fs::File, io::BufReader, path::PathBuf};

pub const SAVES_DIR_NAME: &str = "saves";
pub const CHUNKS_DIR_NAME: &str = "chunks";

lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs =
        ProjectDirs::from("com", "cjburkey", "cjs_whole_new_world").unwrap();
    pub static ref MAIN_DIR: PathBuf = PROJECT_DIRS.config_dir().to_path_buf();
    pub static ref SAVES_DIR: PathBuf = MAIN_DIR.join(SAVES_DIR_NAME);
}

const SERIAL_CONFIG: Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding()
    .with_no_limit();

pub fn saves_dir(world_name: &str) -> PathBuf {
    SAVES_DIR.join(world_name)
}

pub fn chunks_dir(world_name: &str) -> PathBuf {
    saves_dir(world_name).join(CHUNKS_DIR_NAME)
}

pub fn chunk_file(world_name: &str, IVec3 { x, y, z }: IVec3) -> PathBuf {
    chunks_dir(world_name).join(format!("{x}_{y}_{z}.chunk"))
}

pub fn write_chunk_to_file(world_name: &str, chunk_pos: IVec3, chunk_container: &VoxelContainer) {
    std::fs::create_dir_all(&chunks_dir(world_name)).unwrap();
    let chunk_file_path = chunk_file(world_name, chunk_pos);

    std::fs::write(
        &chunk_file_path,
        bincode::serde::encode_to_vec(chunk_container, SERIAL_CONFIG).unwrap(),
    )
    .unwrap();
}

pub fn read_chunk_from_file(world_name: &str, chunk_pos: IVec3) -> Option<VoxelContainer> {
    let chunk_file_path = chunk_file(world_name, chunk_pos);
    match chunk_file_path.exists() {
        true => bincode::serde::decode_from_reader(
            BufReader::new(File::open(&chunk_file_path).ok()?),
            SERIAL_CONFIG,
        )
        .ok(),
        false => None,
    }
}
