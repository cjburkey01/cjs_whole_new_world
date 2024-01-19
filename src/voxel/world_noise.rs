use super::{BiomeTable, Chunk, InChunkPos, Voxel, CHUNK_WIDTH};
use bevy::prelude::*;
use itertools::iproduct;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin,
};

#[derive(Clone)]
pub struct Chunk2dNoiseValues {
    pub chunk_pos: IVec2,
    pub heightmap: Vec<f64>,
    pub temperature: Vec<f64>,
    pub humidity: Vec<f64>,
}

#[derive(Resource, Clone)]
pub struct WorldNoiseSettings {
    heightmap_fbm: Fbm<Perlin>,
    temperature_fbm: Fbm<Perlin>,
    humidity_fbm: Fbm<Perlin>,
    biome_table: BiomeTable,
}

impl WorldNoiseSettings {
    pub fn new(seed: u32, biome_table: BiomeTable) -> Self {
        Self {
            heightmap_fbm: Fbm::<Perlin>::new(seed)
                .set_frequency(0.02)
                .set_persistence(0.45),
            temperature_fbm: Fbm::<Perlin>::new((seed / 3 + 893) + 10)
                .set_octaves(2)
                .set_frequency(0.003),
            humidity_fbm: Fbm::<Perlin>::new(30 * (seed + 4) / 7)
                .set_octaves(2)
                .set_frequency(0.014),
            biome_table,
        }
    }

    pub fn chunk_2d_noise_fn(fbm: &Fbm<Perlin>, chunk_pos: IVec2) -> Vec<f64> {
        PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size(CHUNK_WIDTH as usize, CHUNK_WIDTH as usize)
            .set_x_bounds(chunk_pos.x as f64, chunk_pos.x as f64 + 1.0)
            .set_y_bounds(chunk_pos.y as f64, chunk_pos.y as f64 + 1.0)
            .build()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn generate_chunk_2d_noise(&self, chunk_pos: IVec2) -> Chunk2dNoiseValues {
        Chunk2dNoiseValues {
            chunk_pos,
            heightmap: Self::chunk_2d_noise_fn(&self.heightmap_fbm, chunk_pos),
            temperature: Self::chunk_2d_noise_fn(&self.temperature_fbm, chunk_pos),
            humidity: Self::chunk_2d_noise_fn(&self.humidity_fbm, chunk_pos),
        }
    }

    pub fn generate_chunk_from_noise(&self, y_level: i32, noise: &Chunk2dNoiseValues) -> Chunk {
        let mut chunk = Chunk::default();
        let heightmap = noise.heightmap.as_slice();

        let mut definitely_empty = true;
        for (z, x) in iproduct!(0..CHUNK_WIDTH, 0..CHUNK_WIDTH) {
            let height = heightmap[(z * CHUNK_WIDTH + x) as usize] * 150.0
                - (y_level * CHUNK_WIDTH as i32) as f64
                + 10.0;
            for y in 0..(height.max(0.0) as u32).min(CHUNK_WIDTH) {
                definitely_empty = false;
                chunk.set(
                    InChunkPos::new(UVec3::new(x, y, z)).unwrap(),
                    match y as f64 {
                        y if y < (height - 6.0) => Voxel::Stone,
                        y if y < (height - 3.0) => Voxel::Dirt,
                        _ => Voxel::Grass,
                    },
                );
            }
        }

        Chunk {
            definitely_empty,
            ..chunk
        }
    }
}
