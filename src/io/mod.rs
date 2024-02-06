use crate::voxel::{Chunk, VoxelRegion};
use bevy::prelude::IVec3;
use bincode::config::Configuration;
use directories::ProjectDirs;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

pub const SAVES_DIR_NAME: &str = "saves";
pub const CHUNKS_DIR_NAME: &str = "chunks";
pub const REGIONS_DIR_NAME: &str = "regions";

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

pub fn save_chunks_dir(world_name: &str) -> PathBuf {
    saves_dir(world_name).join(CHUNKS_DIR_NAME)
}

pub fn save_chunk_file(world_name: &str, IVec3 { x, y, z }: IVec3) -> PathBuf {
    save_chunks_dir(world_name).join(format!("{x}_{y}_{z}.chunk.gz"))
}

pub fn save_regions_dir(world_name: &str) -> PathBuf {
    saves_dir(world_name).join(REGIONS_DIR_NAME)
}

pub fn save_region_file(world_name: &str, IVec3 { x, y, z }: IVec3) -> PathBuf {
    save_chunks_dir(world_name).join(format!("{x}_{y}_{z}.chunk.gz"))
}

pub fn write_chunk_to_file(world_name: &str, chunk_pos: IVec3, chunk: &Chunk) {
    std::fs::create_dir_all(save_chunks_dir(world_name)).unwrap();
    let chunk_file_path = save_chunk_file(world_name, chunk_pos);
    write_to_file(&chunk_file_path, chunk);
}

pub fn read_chunk_from_file(world_name: &str, chunk_pos: IVec3) -> Option<Chunk> {
    read_from_file::<Chunk>(&save_chunk_file(world_name, chunk_pos)).map(|mut chunk| {
        chunk.update_edge_slice_bits();
        chunk
    })
}

pub fn write_region_to_file(world_name: &str, chunk_pos: IVec3, region: &VoxelRegion) {
    std::fs::create_dir_all(save_regions_dir(world_name)).unwrap();
    let region_file_path = save_region_file(world_name, chunk_pos);
    write_to_file(&region_file_path, region);
}

fn write_to_file<Data: serde::Serialize>(path: &Path, input_data: Data) {
    // Serialize chunk
    let serialized_data = bincode::serde::encode_to_vec(input_data, SERIAL_CONFIG).unwrap();
    let mut gzip_encoder = GzEncoder::new(File::create(path).unwrap(), Compression::default());
    gzip_encoder.write_all(&serialized_data[..]).unwrap();
    let mut file_write = gzip_encoder.finish().unwrap();
    file_write.flush().unwrap();
}

fn read_from_file<Data: DeserializeOwned>(path: &Path) -> Option<Data> {
    match path.exists() {
        true => {
            let gzip_decoder = GzDecoder::new(File::open(path).ok()?);
            bincode::serde::decode_from_reader(BufReader::new(gzip_decoder), SERIAL_CONFIG).ok()
        }
        false => None,
    }
}
